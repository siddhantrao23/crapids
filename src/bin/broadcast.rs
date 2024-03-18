use crapids::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
    time::Duration,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Gossip {
        seen: HashSet<usize>,
    },
}

struct BroadcastNode {
    id: usize,
    node: String,
    messages: HashSet<usize>,
    neighbours: Vec<String>,
    known: HashMap<String, HashSet<usize>>,
}

impl Node<(), BroadcastPayload> for BroadcastNode {
    fn from_init(
        _state: (),
        init: Init,
        tx: std::sync::mpsc::Sender<Event<BroadcastPayload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(300));
            if let Err(_) = tx.send(Event::Inject()) {
                break;
            }
        });

        Ok(BroadcastNode {
            id: 1,
            node: init.node_id,
            messages: HashSet::new(),
            neighbours: Vec::new(),
            known: init
                .node_ids
                .into_iter()
                .map(|nid| (nid, HashSet::new()))
                .collect(),
        })
    }

    fn step(
        &mut self,
        input: Event<BroadcastPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input {
            Event::Message(input) => {
                let mut reply = input.into_reply(Some(&mut self.id));
                match reply.body.payload {
                    BroadcastPayload::Broadcast { message } => {
                        self.messages.insert(message);
                        reply.body.payload = BroadcastPayload::BroadcastOk {};
                        reply.send(output).context("Send response to Broadcast.")?;
                    }
                    BroadcastPayload::BroadcastOk { .. } => {}
                    BroadcastPayload::Read => {
                        reply.body.payload = BroadcastPayload::ReadOk {
                            messages: self.messages.clone(),
                        };
                        reply.send(output).context("Send response to Read.")?;
                    }
                    BroadcastPayload::ReadOk { .. } => {}
                    BroadcastPayload::Topology { mut topology } => {
                        self.neighbours = topology.remove(&self.node).unwrap_or_else(|| {
                            panic!("No topology provided for node {}", self.node)
                        });
                        reply.body.payload = BroadcastPayload::TopologyOk {};
                        reply.send(output).context("Send response to Topology.")?;
                    }
                    BroadcastPayload::TopologyOk => {}
                    BroadcastPayload::Gossip { seen } => {
                        self.messages.extend(seen);
                    }
                }
            }
            Event::Inject() => {
                for neighbour in &self.neighbours {
                    let neighbour_knows = &self.known[neighbour];
                    Message {
                        src: self.node.clone(),
                        dst: neighbour.clone(),
                        body: Body {
                            id: None,
                            in_reply_to: None,
                            payload: BroadcastPayload::Gossip {
                                seen: self
                                    .messages
                                    .iter()
                                    .copied()
                                    .filter(|m| !neighbour_knows.contains(m))
                                    .collect(),
                            },
                        },
                    }
                    .send(output)
                    .context(format!("Send response to node {}", neighbour))?;
                }
            }
            Event::EOF() => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
