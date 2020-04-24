use std::sync::mpsc::{channel, Receiver, Sender};

use serde::{Serialize, Serializer};

use crate::event::BoxedEvent;

pub struct Trace {
    events: Vec<BoxedEvent>,
    recv: Receiver<BoxedEvent>,
}

impl Trace {
    pub fn new() -> (Sender<BoxedEvent>, Self) {
        let (send, recv) = channel();

        (
            send,
            Self {
                events: vec![],
                recv,
            },
        )
    }

    pub fn collect(&mut self) {
        while let Ok(event) = self.recv.recv() {
            self.events.push(event);
        }
    }

    pub fn serialize<S>(self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.events.serialize(ser)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::event::Event;
    use crate::impl_event;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Foo {
        foo: u32,
    }
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Bar {
        bar: Option<String>,
    }
    impl_event!(Foo);
    impl_event!(Bar);

    #[test]
    fn test_serialize() {
        let (send, mut trace) = Trace::new();

        send.send(Box::new(Foo { foo: 42 })).unwrap();
        send.send(Box::new(Bar {
            bar: Some("test".to_string()),
        }))
        .unwrap();

        std::mem::drop(send);

        trace.collect();

        let mut output = Vec::new();
        let mut ser = serde_json::Serializer::pretty(&mut output);
        trace.serialize(&mut ser).unwrap();

        let as_string = String::from_utf8(output).unwrap();
        println!("{}", as_string);

        let events: Vec<Box<dyn Event>> = serde_json::from_str(&as_string).unwrap();
        for event in events {
            assert_eq!(&event, &event);
            dbg!(&event);
        }
    }
}
