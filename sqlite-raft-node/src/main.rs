// Copyright 2018 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate bincode;
extern crate crossbeam_channel as channel;
extern crate failure;
#[macro_use]
extern crate log;
extern crate raft;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate serdebug;
extern crate simplelog;
extern crate sqlite_raft_storage;

use channel::{Receiver, Sender};
use channel::select;
use failure::Error;
use log::LevelFilter;
use log_entry::LogEntry;
use log_entry::LogEntryFactory;
use log_entry::LogEntryKey;
use raft::prelude::*;
use raft::storage::MemStorage;
use router::Router;
use simplelog::{Config as LogConfig, TermLogger};
use sqlite_raft_storage::SqliteStorage;
use sqlite_raft_storage::storage_traits::StorageMut;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};


mod log_entry;

type ProposeCallback = Box<Fn() + Send>;

mod router;

// TODO: add API for grpc thread (propose)
// TODO: evaluate channel based callback
// TODO: Cluster/Node struct to define API

#[derive(Serialize, SerDebug)]
pub enum TransportMessage {
    Propose(Propose),
    #[serde(skip_serializing)]
    Raft(Message),
}

#[derive(Serialize)]
pub struct Propose {
    pub log_entry: LogEntry,
    #[serde(skip_serializing)]
    pub cb: ProposeCallback,
}

impl Propose {
    fn new(log_entry_factory: &mut LogEntryFactory, text: String, cb: ProposeCallback) -> Propose {
        Propose {
            log_entry: log_entry_factory.new_log_entry(text),
            cb,
        }
    }

    fn into_msg(self) -> TransportMessage {
        TransportMessage::Propose(self)
    }
}


// A simple example about how to use the Raft library in Rust.
fn main() -> Result<(), Error> {
    init_log();

    launch_cluster(3)?;

    Ok(())
}

fn init_log() {
    TermLogger::init(LevelFilter::Debug, LogConfig::default()).unwrap();
}

fn launch_cluster(node_count: u64) -> Result<(), Error> {
    let mut handles: Vec<JoinHandle<_>> = vec![];

    let node_ids = 1..=node_count;

    let routers = Router::new_mesh(node_count);

    for (node_id, router) in node_ids.clone().zip(routers) {
        let node_ids = node_ids.clone();

        let handle = thread::spawn(move || {
            let peers: Vec<u64> = node_ids.collect();
            let propose = node_id == 1;

            launch_node(NodeConfig { node_id, peers, router, propose })
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}

struct NodeConfig {
    pub node_id: u64,
    pub peers: Vec<u64>,
    pub router: Router,
    pub propose: bool,
}

fn launch_node(node_config: NodeConfig) -> Result<(), Error> {
    // Create a storage for Raft, and here we just use a simple memory storage.
    // You need to build your own persistent storage in your production.
    // Please check the Storage trait in src/storage.rs to see how to implement one.
    // let storage = MemStorage::new();

    let NodeConfig {
        node_id,
        peers,
        router,
        propose,
    } = node_config;

    let storage = SqliteStorage::open(format!("res/debug/raft_storage_{}.sqlite3", node_id))?;

    // Create the configuration for the Raft node.
    let cfg = Config {
        // The unique ID for the Raft node.
        id: node_id,
        // The Raft node list.
        // Mostly, the peers need to be saved in the storage
        // and we can get them from the Storage::initial_state function, so here
        // you need to set it empty.
        peers,
        // Election tick is for how long the follower may campaign again after
        // it doesn't receive any message from the leader.
        election_tick: 10,
        // Heartbeat tick is for how long the leader needs to send
        // a heartbeat to keep alive.
        heartbeat_tick: 3,
        // The max size limits the max size of each appended message. Mostly, 1 MB is enough.
        max_size_per_msg: 1024 * 1024 * 1024,
        // Max inflight msgs that the leader sends messages to follower without
        // receiving ACKs.
        max_inflight_msgs: 256,
        // The Raft applied index.
        // You need to save your applied index when you apply the committed Raft logs.
        applied: 0,
        // Just for log
        tag: format!("[{}]", node_id),
        ..Default::default()
    };

    // Create the Raft node.
    let mut r = RawNode::new(&cfg, storage, vec![]).unwrap();

    if propose {
        // Use another thread to propose a Raft request.

        let log_entry_factory = LogEntryFactory::new(node_id, 0);

        send_propose(router.clone_own_sender(), log_entry_factory);
    }

    // Loop forever to drive the Raft.
    let mut t = Instant::now();
    let mut timeout = Duration::from_millis(100);

    // Use a HashMap to hold the `propose` callbacks.
    let mut cbs = HashMap::new();

    // TODO: external stepping (debug)
    loop {
        fn on_msg<S: StorageMut>(msg: Option<TransportMessage>, r: &mut RawNode<S>, cbs: &mut HashMap<LogEntryKey, ProposeCallback>)
                                 -> Result<bool, Error> {
            match msg {
                Some(msg) => {
                    match msg {
                        TransportMessage::Propose(Propose { log_entry, cb }) => {
                            cbs.insert(log_entry.key(), cb);

                            r.propose(vec![], log_entry.to_vec_u8()).unwrap();
                        }
                        TransportMessage::Raft(m) => r.step(m).unwrap(),
                    };

                    Ok(true)
                }
                // channel closed
                None => Ok(false),
            }
        };

        select! {
            recv(router.receiver(), msg) => if !on_msg(msg, &mut r, &mut cbs)? {
                return Ok(())
            },
            recv(channel::after(timeout)) => (),
        }

        let d = t.elapsed();
        if d >= timeout {
            t = Instant::now();
            timeout = Duration::from_millis(100);
            // We drive Raft every 100ms.
            r.tick();
        } else {
            timeout -= d;
        }

        on_ready(&mut r, &mut cbs, &router)?;
    }
}

fn on_ready<S: StorageMut>(r: &mut RawNode<S>, cbs: &mut HashMap<LogEntryKey, ProposeCallback>, router: &Router)
                           -> Result<(), Error> {
//    debug!("{} RawNode:\n{}", r.raft.tag, serde_json::to_string_pretty(&r).unwrap());

    if !r.has_ready() {
        return Ok(());
    }

    // The Raft is ready, we can do something now.
    let mut ready = r.ready();

//    debug!("{} ready:\n{}", r.raft.tag, serde_json::to_string_pretty(&ready).unwrap());

    let is_leader = r.raft.leader_id == r.raft.id;
    if is_leader {
        // If the peer is leader, the leader can send messages to other followers ASAP.
        let msgs = ready.messages.drain(..);
        for msg in msgs {
            debug!("{} Leader send message: {:?}", r.raft.tag, msg);

            router.send_raft(msg);
        }
    }

    if !raft::is_empty_snap(&ready.snapshot) {
        // This is a snapshot, we need to apply the snapshot at first.
        r.mut_store()
            // TODO: remove clone
            .apply_snapshot(ready.snapshot.clone())
            .unwrap();
    }

    if !ready.entries.is_empty() {
        // Append entries to the Raft log
        r.mut_store().append(&ready.entries).unwrap();
    }

    if let Some(ref hs) = ready.hs {
        // Raft HardState changed, and we need to persist it.
        r.mut_store().set_hardstate(hs.clone())?;
    }

    if !is_leader {
        // If not leader, the follower needs to reply the messages to
        // the leader after appending Raft entries.
        let msgs = ready.messages.drain(..);
        for msg in msgs {
            // Send messages to other peers.

            debug!("{} Follower send message: {:?}", r.raft.tag, msg);

            router.send_raft(msg);
        }
    }

    if let Some(committed_entries) = ready.committed_entries.take() {
        let mut _last_apply_index = 0;
        for entry in committed_entries {
            let entry: Entry = entry;

            // Mostly, you need to save the last apply index to resume applying
            // after restart. Here we just ignore this because we use a Memory storage.
            _last_apply_index = entry.get_index();

            if entry.get_data().is_empty() {
                // Emtpy entry, when the peer becomes Leader it will send an empty entry.
                continue;
            }

            if entry.get_entry_type() == EntryType::EntryNormal {
                // TODO: Callback/Request manager

                let log_entry = LogEntry::try_from(entry.get_data()).unwrap();

                if let Some(cb) = cbs.remove(&log_entry.key()) {
                    debug!("{} found callback for log entry: {}", r.raft.tag, log_entry.text());
                    cb();
                } else {
                    debug!("{} no callback for log entry: {}", r.raft.tag, log_entry.text());
                }
            }

            // TODO: handle EntryConfChange
        }
    }

    // Advance the Raft
    r.advance(ready);

    Ok(())
}

fn send_propose(sender: Sender<TransportMessage>, mut log_entry_factory: LogEntryFactory) {
    thread::spawn(move || {
        // Wait some time and send the request to the Raft.
        thread::sleep(Duration::from_secs(10));

        let propose_count = 1;

        let mut cb_rxs: Vec<Receiver<u64>> = vec![];

        for propose_index in 0..propose_count {
            // Send a command to the Raft, wait for the Raft to apply it
            // and get the result.
            println!("propose request {}", propose_index);

            let (cb_tx, cb_rx) = channel::unbounded::<u64>();

            cb_rxs.push(cb_rx);

            let msg = Propose::new(
                &mut log_entry_factory,
                "Hello World!".to_string(),
                Box::new(move || {
                    cb_tx.send(propose_index);
                }),
            ).into_msg();

            sender
                .send(msg);
        }

        let mut results = cb_rxs.into_iter().map(|cb_rx| {
            let res = cb_rx.recv().unwrap();

            println!("received propose callback {}", res);

            res
        }).collect::<Vec<_>>();

        results.sort();

        assert_eq!(&results, &(0..propose_count).collect::<Vec<_>>())
    });
}
