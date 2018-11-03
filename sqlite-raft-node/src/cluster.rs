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


pub struct Cluster {
    handles: Vec<JoinHandle<Result<(), Error>>>,
}

impl Cluster {
    // TODO: split into new/stop?
    pub fn launch_cluster(node_count: u64) -> Result<(), Error> {
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

}

