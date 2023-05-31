use std::{
    io::{self},
    pin::Pin,
    task,
};

use tokio::{
    io::{AsyncRead, BufReader, ReadBuf, Sink},
    process::{ChildStderr, ChildStdout},
    sync::mpsc::error::TryRecvError,
};

use crate::prelude::*;

pub enum VashRead {
    Stdout(BufReader<ChildStdout>),
    Stderr(BufReader<ChildStderr>),
    Delegate(ReadDelegate),
    Sink(ReadSink),
    Canned(Vec<u8>),
}

impl From<ChildStdout> for VashRead {
    fn from(value: ChildStdout) -> Self {
        Self::Stdout(BufReader::new(value))
    }
}

impl From<ChildStderr> for VashRead {
    fn from(value: ChildStderr) -> Self {
        Self::Stderr(BufReader::new(value))
    }
}

impl AsyncRead for VashRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> task::Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Stdout(stdout) => Pin::new(stdout).poll_read(cx, buf),
            Self::Stderr(stderr) => Pin::new(stderr).poll_read(cx, buf),
            Self::Delegate(delegate) => Pin::new(delegate).poll_read(cx, buf),
            Self::Sink(sink) => Pin::new(sink).poll_read(cx, buf),
            Self::Canned(canned) => {
                buf.put_slice(&canned);
                task::Poll::Ready(Ok(()))
            }
        }
    }
}

pub struct ReadSink;

impl AsyncRead for ReadSink {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> task::Poll<io::Result<()>> {
        task::Poll::Ready(Ok(()))
    }
}

pub enum ReadMessage {
    Read(Vec<u8>),
    Closed,
}

pub struct ReadDelegate {
    pub receiver: Receiver<ReadMessage>,
}

impl AsyncRead for ReadDelegate {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> task::Poll<io::Result<()>> {
        match self.as_mut().receiver.try_recv() {
            Ok(ReadMessage::Read(data)) => {
                buf.put_slice(&data);
                task::Poll::Ready(Ok(()))
            }
            Ok(ReadMessage::Closed) => task::Poll::Ready(Ok(())),
            Err(TryRecvError::Empty) => task::Poll::Pending,
            Err(TryRecvError::Disconnected) => task::Poll::Ready(Ok(())),
        }
    }
}
