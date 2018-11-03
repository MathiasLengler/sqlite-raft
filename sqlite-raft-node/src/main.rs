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

use failure::Error;
use log::LevelFilter;
use log_entry::LogEntry;
use log_entry::LogEntryFactory;
use raft::prelude::*;
use router::Router;
use simplelog::{Config as LogConfig, TermLogger};
use sqlite_raft_storage::SqliteStorage;
use std::thread;
use std::thread::JoinHandle;
use node::{Node, NodeConfig};


mod log_entry;

type ProposeCallback = Box<Fn() + Send>;

mod router;
mod node;

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

            let storage = SqliteStorage::open(format!("res/debug/raft_storage_{}.sqlite3", node_id))?;

            let mut node = Node::new(NodeConfig {
                node_id,
                peers,
                router,
                propose,
                storage,
            })?;
            node.run()
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}
