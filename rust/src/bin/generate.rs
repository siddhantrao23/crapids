use crapids::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::StdoutLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum GeneratePayload {
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

struct GenerateNode {
    node: String,
    id: usize,
}

impl Node<(), GeneratePayload> for GenerateNode {
    fn from_init(
        _state: (),
        init: Init,
        _tx: std::sync::mpsc::Sender<Event<GeneratePayload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(GenerateNode {
            node: init.node_id,
            id: 1,
        })
    }

    fn step(
        &mut self,
        input: Event<GeneratePayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("Got injected event when there should be none.");
        };
        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.payload {
            GeneratePayload::Generate => {
                let guid = format!("{}-{}", self.node, self.id);
                reply.body.payload = GeneratePayload::GenerateOk { guid };
                reply.send(output).context("Send response to generate.")?;
            }
            GeneratePayload::GenerateOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, GenerateNode, _>(())
}
