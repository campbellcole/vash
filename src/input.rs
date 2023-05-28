use std::{
    io::{stdin, stdout},
    time::Duration,
};

use termion::{event::Key, input::TermRead, raw::IntoRawMode};

use crate::prelude::*;

#[derive(Debug)]
pub enum InputMessage {
    Char(char),
    CtrlC,
    Enter,
    Backspace,
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
                Ok(c) => match c {
                    Key::Char('\n') => {
                        sender.send(InputMessage::Enter).unwrap();
                    }
                    Key::Char(c) => {
                        sender.send(InputMessage::Char(c)).unwrap();
                    }
                    Key::Ctrl('c') => {
                        sender.send(InputMessage::CtrlC).unwrap();
                        break;
                    }
                    Key::Backspace => {
                        sender.send(InputMessage::Backspace).unwrap();
                    }
                    _ => {}
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
