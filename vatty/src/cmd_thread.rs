use std::{io, process::Stdio};

use iced::futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

pub enum CommandMessage {
    Stdin(Vec<u8>),
    Kill,
}

#[derive(Debug)]
pub enum CommandResponse {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    ExitCode(Option<i32>),
    Error(String),
}

pub fn cmd_thread(
    mut cmd: Command,
) -> (
    mpsc::UnboundedSender<CommandMessage>,
    mpsc::UnboundedReceiver<CommandResponse>,
) {
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let (pub_tx, rx) = mpsc::unbounded();
    let (tx, pub_rx) = mpsc::unbounded();

    tokio::task::spawn(async move {
        if let Err(err) = cmd_thread_internal(cmd, tx.clone(), rx).await {
            tx.unbounded_send(CommandResponse::Error(err.to_string()))
                .unwrap();
        }
    });

    (pub_tx, pub_rx)
}

#[derive(Debug, Error)]
pub enum CmdThreadError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("channel send error: {0}")]
    ChannelSend(#[from] mpsc::TrySendError<CommandResponse>),
}

async fn cmd_thread_internal(
    mut cmd: Command,
    tx: UnboundedSender<CommandResponse>,
    mut rx: UnboundedReceiver<CommandMessage>,
) -> Result<(), CmdThreadError> {
    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();

    let mut stdout_buf = Vec::with_capacity(64);
    let mut stderr_buf = Vec::with_capacity(64);

    loop {
        tokio::select! {
            input = rx.select_next_some() => {
                match input {
                    CommandMessage::Stdin(input) => {
                        child_stdin.write_all(&input).await?;
                    }
                    CommandMessage::Kill => {
                        child.kill().await?;
                        break;
                    }
                }
            }
            len = child_stdout.read_buf(&mut stdout_buf) => {
                if len? > 0 {
                    tx.unbounded_send(CommandResponse::Stdout(std::mem::take(&mut stdout_buf)))
                        ?;
                }
            }
            len = child_stderr.read_buf(&mut stderr_buf) => {
                if len? > 0 {
                    tx.unbounded_send(CommandResponse::Stderr(std::mem::take(&mut stderr_buf)))
                        ?;
                }
            }
            status = child.wait() => {
                match status {
                    Ok(exit) => {
                        tx.unbounded_send(CommandResponse::ExitCode(exit.code()))?;
                    }
                    Err(err) => {
                        tx.unbounded_send(CommandResponse::Error(err.to_string()))?;
                    }
                }
                break;
            }
        }
    }

    Ok(())
}
