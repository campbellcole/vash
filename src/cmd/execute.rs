use std::{io::BufRead, process::Stdio, thread};

use super::execution_plan::ExecutionPlan;
use crate::prelude::*;

#[derive(Debug)]
pub enum ExecutionMessage {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Error(String),
}

pub type ExecutionReceiver = Receiver<ExecutionMessage>;
pub type ExecutionSender = Sender<ExecutionMessage>;

pub fn execute(plan: ExecutionPlan, tx: ExecutionSender) {
    match plan {
        ExecutionPlan::Execute(cmd) => {
            let mut split = cmd.split_whitespace();
            let cmd = split.next().unwrap();
            let args = split.collect::<Vec<_>>();
            let child = std::process::Command::new(cmd)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn();

            let mut child = match child {
                Ok(c) => c,
                Err(err) => {
                    tx.send(ExecutionMessage::Error(err.to_string())).unwrap();
                    return;
                }
            };

            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();

            thread::spawn(move || {
                for line in std::io::BufReader::new(stdout).lines() {
                    tx.send(ExecutionMessage::Stdout(line.unwrap().into_bytes()))
                        .unwrap();
                }
            });
        }
        _ => {
            todo!("execute: {:?}", plan);
        }
    }
}
