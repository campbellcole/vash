use std::io::stdin;

use termion::{event::Key, input::TermRead};

use crate::prelude::*;

#[derive(Debug)]
pub enum InputMessage {
    Event(Key),
    Error(String),
}

pub type InputReceiver = Receiver<InputMessage>;
pub type InputSender = Sender<InputMessage>;

pub async fn spawn_input_thread() -> InputReceiver {
    let (sender, receiver) = unbounded_channel();
    tokio::task::spawn_blocking(move || {
        let stdin = stdin();

        for c in stdin.keys() {
            match c {
                Ok(c) => sender.send(InputMessage::Event(c)).unwrap(),

                Err(err) => {
                    sender.send(InputMessage::Error(err.to_string())).unwrap();
                    break;
                }
            }
        }
    });
    receiver
}
