use async_trait::async_trait;
use nix::sys::signal::Signal;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};

use crate::{prelude::*, process::VashProcess};

#[derive(Debug)]
pub enum DelegateMessage {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Exit(Option<i32>),
    Error(String),
}

#[derive(Debug)]
pub enum DelegateCommand {
    Stdin(Vec<u8>),
    Signal(Signal),
}

pub struct ExecutionDelegate {
    pub tx: Sender<DelegateCommand>,
    pub rx: Receiver<DelegateMessage>,
}

#[async_trait]
pub trait Delegate {
    fn send(&self, cmd: DelegateCommand);
    async fn recv(&mut self) -> Option<DelegateMessage>;
}

#[async_trait]
impl Delegate for ExecutionDelegate {
    fn send(&self, cmd: DelegateCommand) {
        self.tx.send(cmd).unwrap();
    }

    async fn recv(&mut self) -> Option<DelegateMessage> {
        self.rx.recv().await
    }
}

#[async_trait]
impl Delegate for Option<ExecutionDelegate> {
    fn send(&self, cmd: DelegateCommand) {
        if let Some(delegate) = self {
            delegate.send(cmd);
        }
    }

    async fn recv(&mut self) -> Option<DelegateMessage> {
        if let Some(delegate) = self {
            delegate.recv().await
        } else {
            None
        }
    }
}

impl ExecutionDelegate {
    pub async fn spawn(mut exec: VashProcess) -> Self {
        let (mtx, mrx) = unbounded_channel();
        let (ctx, mut crx) = unbounded_channel();

        tokio::task::spawn(async move {
            let mut stdout_buf = Vec::new();
            let mut stderr_buf = Vec::new();

            loop {
                select! {
                    Some(cmd) = crx.recv() => {
                        match cmd {
                            DelegateCommand::Stdin(data) => {
                                exec.stdin.write_all(&data).await.unwrap();
                                exec.stdin.flush().await.unwrap();
                            }
                            DelegateCommand::Signal(sig) => {
                                exec.child.signal(sig).await.unwrap();
                            }
                        }
                    }
                    Ok(stdout_len) = exec.stdout.read_buf(&mut stdout_buf) => {
                        if stdout_len == 0 {
                            continue;
                        }

                        mtx.send(DelegateMessage::Stdout(std::mem::take(&mut stdout_buf))).unwrap();
                    }
                    Ok(stderr_len) = exec.stderr.read_buf(&mut stderr_buf) => {
                        if stderr_len == 0 {
                            continue;
                        }

                        mtx.send(DelegateMessage::Stderr(std::mem::take(&mut stderr_buf))).unwrap();
                    }
                    output = exec.child.wait() => {
                        // drain the remaining stdout/stderr
                        if let Ok(len) = exec.stdout.read_to_end(&mut stdout_buf).await {
                            if len > 0 {
                                mtx.send(DelegateMessage::Stdout(std::mem::take(&mut stdout_buf))).unwrap();
                            }
                        }

                        if let Ok(len) = exec.stderr.read_to_end(&mut stderr_buf).await {
                            if len > 0 {
                                mtx.send(DelegateMessage::Stderr(std::mem::take(&mut stderr_buf))).unwrap();
                            }
                        }

                        match output {
                            Ok(exit) => {
                                mtx.send(DelegateMessage::Exit(exit.code())).unwrap();
                                break;
                            }
                            Err(err) => {
                                mtx.send(DelegateMessage::Error(err.to_string())).unwrap();
                                break;
                            }
                        }
                    }
                }
            }
        });

        Self { tx: ctx, rx: mrx }
    }
}
