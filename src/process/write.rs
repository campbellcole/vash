use std::{io, pin::Pin, task};

use tokio::{
    io::{AsyncWrite, BufWriter, DuplexStream, Sink, WriteHalf},
    process::ChildStdin,
};

use crate::prelude::*;

pub enum VashWrite {
    Stdin(BufWriter<ChildStdin>),
    Delegate(WriteDelegate),
    Sink(Sink),
    Duplex(WriteHalf<DuplexStream>),
}

impl From<ChildStdin> for VashWrite {
    fn from(value: ChildStdin) -> Self {
        Self::Stdin(BufWriter::new(value))
    }
}

impl AsyncWrite for VashWrite {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Self::Stdin(stdin) => Pin::new(stdin).poll_write(cx, buf),
            Self::Delegate(delegate) => Pin::new(delegate).poll_write(cx, buf),
            Self::Sink(sink) => Pin::new(sink).poll_write(cx, buf),
            Self::Duplex(duplex) => Pin::new(duplex).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Self::Stdin(stdin) => Pin::new(stdin).poll_flush(cx),
            Self::Delegate(delegate) => Pin::new(delegate).poll_flush(cx),
            Self::Sink(sink) => Pin::new(sink).poll_flush(cx),
            Self::Duplex(duplex) => Pin::new(duplex).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Self::Stdin(stdin) => Pin::new(stdin).poll_shutdown(cx),
            Self::Delegate(delegate) => Pin::new(delegate).poll_shutdown(cx),
            Self::Sink(sink) => Pin::new(sink).poll_shutdown(cx),
            Self::Duplex(duplex) => Pin::new(duplex).poll_shutdown(cx),
        }
    }
}

#[derive(Debug)]
pub enum WriteMessage {
    Write(Vec<u8>),
    Flush,
    Close,
}

pub struct WriteDelegate {
    pub sender: Sender<WriteMessage>,
}

impl AsyncWrite for WriteDelegate {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> task::Poll<io::Result<usize>> {
        let msg = WriteMessage::Write(buf.to_vec());

        self.sender
            .send(msg)
            .map_err(|_| std::io::ErrorKind::BrokenPipe.into())
            .map(|_| buf.len())
            .into()
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut task::Context<'_>) -> task::Poll<io::Result<()>> {
        let msg = WriteMessage::Flush;

        self.sender
            .send(msg)
            .map_err(|_| std::io::ErrorKind::BrokenPipe.into())
            .into()
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> task::Poll<io::Result<()>> {
        let msg = WriteMessage::Close;

        self.sender
            .send(msg)
            .map_err(|_| std::io::ErrorKind::BrokenPipe.into())
            .into()
    }
}
