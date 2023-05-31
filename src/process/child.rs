use std::{io, process::ExitStatus};

use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use tokio::{process::Child, task::JoinHandle};

use crate::prelude::*;

use super::status::{BuiltinExitStatus, VashExitStatus};

pub enum VashChild {
    Process(Child),
    Delegate(ChildDelegate),
    PreExecuted(BuiltinExitStatus),
    Thread(JoinHandle<VashExitStatus>),
}

impl From<Child> for VashChild {
    fn from(value: Child) -> Self {
        Self::Process(value)
    }
}

impl VashChild {
    pub async fn wait(&mut self) -> io::Result<VashExitStatus> {
        match self {
            Self::Process(process) => process.wait().await.map(Into::into),
            Self::Delegate(delegate) => delegate.wait().await.map(Into::into),
            Self::PreExecuted(status) => Ok(VashExitStatus::from(*status)),
            Self::Thread(handle) => handle.await.map_err(|_| {
                io::Error::new(io::ErrorKind::BrokenPipe, "Child exited unexpectedly")
            }),
        }
    }

    pub async fn kill(&mut self) -> io::Result<()> {
        match self {
            Self::Process(process) => process.kill().await,
            Self::Delegate(delegate) => {
                delegate.sender.send(ChildCommand::Kill).map_err(|_| {
                    io::Error::new(io::ErrorKind::BrokenPipe, "Child exited unexpectedly")
                })?;
                delegate.wait().await?;
                Ok(())
            }
            Self::PreExecuted(_) => Ok(()),
            Self::Thread(handle) => {
                handle.abort();
                Ok(())
            }
        }
    }

    pub async fn signal(&mut self, signal: Signal) -> io::Result<()> {
        match self {
            Self::Process(process) => {
                let id = process.id().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::BrokenPipe, "Child exited unexpectedly")
                })? as i32;
                tokio::task::spawn_blocking(move || kill(Pid::from_raw(id), signal)).await??;
            }
            Self::Delegate(delegate) => {
                delegate
                    .sender
                    .send(ChildCommand::Signal(signal))
                    .map_err(|_| {
                        io::Error::new(io::ErrorKind::BrokenPipe, "Child exited unexpectedly")
                    })?;

                delegate.wait().await?;
            }
            Self::PreExecuted(_) => {}
            Self::Thread(handle) => match signal {
                Signal::SIGABRT | Signal::SIGINT | Signal::SIGTERM => {
                    handle.abort();
                }
                _ => {}
            },
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ChildCommand {
    Signal(Signal),
    Kill,
}

#[derive(Debug)]
pub enum ChildMessage {
    Exit(ExitStatus),
}

pub struct ChildDelegate {
    pub sender: Sender<ChildCommand>,
    pub receiver: Receiver<ChildMessage>,
}

impl ChildDelegate {
    #[allow(unreachable_patterns)]
    pub async fn wait(&mut self) -> io::Result<ExitStatus> {
        loop {
            match self.receiver.recv().await {
                Some(ChildMessage::Exit(status)) => break Ok(status),
                // here in case any new variants are added
                Some(_) => continue,
                None => {
                    break Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "Child exited unexpectedly",
                    ))
                }
            }
        }
    }
}
