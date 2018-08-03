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

#[macro_use]
extern crate log;
extern crate raft;
extern crate serde_json;
extern crate simplelog;
extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serdebug;

use log::LevelFilter;
use raft::prelude::*;
use raft::storage::MemStorage;
use simplelog::{Config as LogConfig, TermLogger};
use std::collections::HashMap;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

type ProposeCallback = Box<Fn() + Send>;

#[derive(Serialize, SerDebug)]
enum Msg {
    Propose(Propose),
    Raft(Message),
}

#[derive(Serialize)]
struct Propose {
    id: u64,
    #[serde(skip_serializing)]
    cb: ProposeCallback,
    data: String,
}

impl Propose {
    fn new(cb: ProposeCallback, data: String) -> Propose {
        // TODO: generate global unique id

        unimplemented!()
    }

    fn to_log_data(&self) -> Vec<u8> {
        // TODO: squash together id + data

        unimplemented!()
    }

    fn parse_log_data(data: &[u8]) -> (u64, String) {
        unimplemented!()
    }
}

struct Router {
    senders: Vec<Sender<Msg>>,
    receiver: Receiver<Msg>,
    own_node_id: u64,
}

impl Router {
    pub fn new_mesh(node_count: u64) -> Vec<Router> {
        let node_ids = 1..=node_count;

        let (senders, receivers): (Vec<Sender<Msg>>, Vec<Receiver<Msg>>) =
            node_ids
                .map(|_| mpsc::channel::<Msg>())
                .unzip();

        receivers.into_iter().enumerate().map(|(i, receiver)| {
            let own_node_id = (i + 1) as u64;

            Router {
                senders: senders.clone(),
                receiver,
                own_node_id,
            }
        }).collect()
    }

    pub fn send_raft(&self, msg: Message) {
        if msg.to == self.own_node_id {
            panic!("Tried to send message to own node: {:?} ", msg)
        }

        self.get_sender(msg.to).send(Msg::Raft(msg)).unwrap();
    }

    fn get_sender(&self, node_id: u64) -> &Sender<Msg> {
        &self.senders[(node_id - 1) as usize]
    }

    pub fn clone_own_sender(&self) -> Sender<Msg> {
        self.get_sender(self.own_node_id).clone()
    }
}


// A simple example about how to use the Raft library in Rust.
fn main() {
    init_log();

    launch_cluster(1);
}

fn init_log() {
    TermLogger::init(LevelFilter::Debug, LogConfig::default()).unwrap();
}

fn launch_cluster(node_count: u64) {
    let mut handles: Vec<JoinHandle<_>> = vec![];

    let node_ids = 1..=node_count;

    let routers = Router::new_mesh(node_count);

    for (node_id, router) in node_ids.clone().zip(routers) {
        let node_ids = node_ids.clone();

        let handle = thread::spawn(move || {
            let peers: Vec<u64> = node_ids.collect();
            let propose = node_id == 1;

            launch_node(node_id, peers, router, propose)
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn launch_node(node_id: u64, peers: Vec<u64>, router: Router, propose: bool) {
    // Create a storage for Raft, and here we just use a simple memory storage.
    // You need to build your own persistent storage in your production.
    // Please check the Storage trait in src/storage.rs to see how to implement one.
    let storage = MemStorage::new();

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
        send_propose(router.clone_own_sender());
    }

    // Loop forever to drive the Raft.
    let mut t = Instant::now();
    let mut timeout = Duration::from_millis(100);

    // Use a HashMap to hold the `propose` callbacks.
    let mut cbs = HashMap::new();

    loop {
        match router.receiver.recv_timeout(timeout) {
            Ok(Msg::Propose { id, cb, data }) => {
                cbs.insert(id, cb);
                r.propose(vec![], data).unwrap();
            }
            Ok(Msg::Raft(m)) => r.step(m).unwrap(),
            Err(RecvTimeoutError::Timeout) => (),
            Err(RecvTimeoutError::Disconnected) => return,
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

        on_ready(&mut r, &mut cbs, &router);
    }
}

fn on_ready(r: &mut RawNode<MemStorage>, cbs: &mut HashMap<u8, ProposeCallback>, router: &Router) {
//    debug!("{} RawNode:\n{}", r.raft.tag, serde_json::to_string_pretty(&r).unwrap());

    if !r.has_ready() {
        return;
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
            .wl()
            .apply_snapshot(ready.snapshot.clone())
            .unwrap();
    }

    if !ready.entries.is_empty() {
        // Append entries to the Raft log
        r.mut_store().wl().append(&ready.entries).unwrap();
    }

    if let Some(ref hs) = ready.hs {
        // Raft HardState changed, and we need to persist it.
        r.mut_store().wl().set_hardstate(hs.clone());
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
                // TODO: parse log data

                if let Some(cb) = cbs.remove(entry.get_data().get(0).unwrap()) {
                    cb();
                }
            }

            // TODO: handle EntryConfChange
        }
    }

    // Advance the Raft
    r.advance(ready);
}

fn send_propose(sender: mpsc::Sender<Msg>) {
    thread::spawn(move || {
        // Wait some time and send the request to the Raft.
        thread::sleep(Duration::from_secs(10));

        let (s1, r1) = mpsc::channel::<u8>();

        println!("propose a request");

        // Send a command to the Raft, wait for the Raft to apply it
        // and get the result.
        sender
            .send(Msg::Propose {
                id: 1,
                cb: Box::new(move || {
                    s1.send(0).unwrap();
                }),
                data: "Hello World!".as_bytes().to_vec(),
            })
            .unwrap();

        let n = r1.recv().unwrap();
        assert_eq!(n, 0);

        println!("receive the propose callback");
    });
}
