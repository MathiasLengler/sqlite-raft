
use raft::prelude::*;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use TransportMessage;

// TODO: trait for Node2Node Communication
// TODO: use crossbeam channels
// TODO: compare with new raft-rs testing harness
// TODO: single thread round robin cluster?

trait SendRaft {
    fn send_raft(&self, msg: Message);
}

pub struct Router {
    senders: Vec<Sender<TransportMessage>>,
    receiver: Receiver<TransportMessage>,
    own_node_id: u64,
}

impl Router {
    pub fn new_mesh(node_count: u64) -> Vec<Router> {
        let node_ids = 1..=node_count;

        let (senders, receivers): (Vec<Sender<TransportMessage>>, Vec<Receiver<TransportMessage>>) =
            node_ids
                .map(|_| mpsc::channel::<TransportMessage>())
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

        self.get_sender(msg.to).send(TransportMessage::Raft(msg)).unwrap();
    }

    pub fn receiver(&self) -> &Receiver<TransportMessage> {
        &self.receiver
    }

    fn get_sender(&self, node_id: u64) -> &Sender<TransportMessage> {
        &self.senders[(node_id - 1) as usize]
    }

    pub fn clone_own_sender(&self) -> Sender<TransportMessage> {
        self.get_sender(self.own_node_id).clone()
    }
}
