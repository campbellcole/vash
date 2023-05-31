use std::io::stdin;

use termion::{
    event::{Event, Key, MouseEvent},
    input::TermRead,
};

use crate::prelude::*;

#[derive(Debug)]
pub enum InputMessage {
    Key(Key),
    Mouse(MouseEvent),
    Error(String),
}

pub type InputReceiver = Receiver<InputMessage>;
pub type InputSender = Sender<InputMessage>;

pub async fn spawn_input_thread() -> InputReceiver {
    let (sender, receiver) = unbounded_channel();
    tokio::task::spawn_blocking(move || {
        let stdin = stdin();

        for ev in stdin.events() {
            match ev {
                Ok(ev) => match ev {
                    Event::Key(key) => sender.send(InputMessage::Key(key)).unwrap(),
                    Event::Mouse(mouse) => sender.send(InputMessage::Mouse(mouse)).unwrap(),
                    Event::Unsupported(ev) => {
                        sender
                            .send(InputMessage::Error(format!("Unsupported event: {:?}", ev)))
                            .unwrap();
                    }
                },
                Err(err) => {
                    sender.send(InputMessage::Error(err.to_string())).unwrap();
                    break;
                }
            }
        }
    });
    receiver
}
