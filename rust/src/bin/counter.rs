// does not work because total order is not maintained

use anyhow::{anyhow, Context};
use crapids::*;

use serde::{Deserialize, Serialize};
use std::{
    io::StdoutLock,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum CounterPayload {
    Add { delta: usize },
    AddOk,
    Read,
    ReadOk { value: usize },
    Error,
}

#[derive(Eq, Hash, PartialEq, Debug, Serialize, Deserialize, Clone)]
struct Counter {
    value: usize,
    generation: u128,
}

struct KV<T>
where
    T: Copy + std::ops::AddAssign,
{
    lock: RwLock<T>,
}

impl<T> KV<T>
where
    T: Copy + std::ops::AddAssign,
{
    fn new(init_val: T) -> Self {
        KV {
            lock: RwLock::new(init_val),
        }
    }

    fn get(&self) -> anyhow::Result<T> {
        self.lock
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock."))
            .map(|gaurd| *gaurd)
    }

    fn set(&self, latest_value: T) -> anyhow::Result<()> {
        self.lock
            .write()
            .map_err(|_| anyhow!("Failed to acquire read lock."))
            .map(|mut gaurd| *gaurd += latest_value)
    }
}

struct CounterNode {
    id: usize,
    store: Arc<KV<usize>>,
}

impl Node<(), CounterPayload> for CounterNode {
    fn from_init(
        _state: (),
        _init: Init,
        _tx: std::sync::mpsc::Sender<Event<CounterPayload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(CounterNode {
            id: 1,
            store: Arc::new(KV::new(0)),
        })
    }

    fn step(
        &mut self,
        input: Event<CounterPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input {
            Event::Message(input) => {
                let mut reply = input.into_reply(Some(&mut self.id));
                match reply.body.payload {
                    CounterPayload::Add { delta } => {
                        self.store.set(delta)?;
                        reply.body.payload = CounterPayload::AddOk {};
                        reply.send(output).context("Send response to Add.")?;
                    }
                    CounterPayload::AddOk => {}
                    CounterPayload::Read => {
                        match self.store.get() {
                            Ok(value) => reply.body.payload = CounterPayload::ReadOk { value },
                            Err(_) => reply.body.payload = CounterPayload::Error,
                        }
                        reply.send(output).context("Send response to Read.")?;
                    }
                    CounterPayload::ReadOk { .. } => {}
                    CounterPayload::Error => {}
                }
            }
            Event::Inject() => {}
            Event::EOF() => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, CounterNode, _>(())
}
