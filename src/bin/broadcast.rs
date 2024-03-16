use crapids::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::StdoutLock};

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
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct BroadcastNode {
    id: usize,
    messages: Vec<usize>,
}

impl Node<(), BroadcastPayload> for BroadcastNode {
    fn from_init(_state: (), _init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(BroadcastNode {
            id: 1,
            messages: Vec::new(),
        })
    }

    fn step(
        &mut self,
        input: Message<BroadcastPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.payload {
            BroadcastPayload::Broadcast { message } => {
                self.messages.push(message);
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
            BroadcastPayload::Topology { .. } => {
                reply.body.payload = BroadcastPayload::TopologyOk {};
                reply.send(output).context("Send response to Topology.")?;
            }
            BroadcastPayload::TopologyOk => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
