use std::{future::Future, process::ExitStatus};

use tokio::io::{duplex, split, AsyncRead, AsyncWrite, DuplexStream, ReadHalf, WriteHalf};

use self::{
    read::ReadSink,
    status::{BuiltinExitStatus, VashExitStatus},
};

pub mod child;
pub mod read;
pub mod status;
pub mod write;

pub struct VashProcess {
    pub stdin: write::VashWrite,
    pub stdout: read::VashRead,
    pub stderr: read::VashRead,
    pub child: child::VashChild,
}

impl VashProcess {
    pub fn sink() -> Self {
        VashProcess {
            stdin: write::VashWrite::Sink(tokio::io::sink()),
            stdout: read::VashRead::Sink(ReadSink),
            stderr: read::VashRead::Sink(ReadSink),
            child: child::VashChild::PreExecuted(BuiltinExitStatus::new_success()),
        }
    }

    pub fn sink_failure() -> Self {
        VashProcess {
            stdin: write::VashWrite::Sink(tokio::io::sink()),
            stdout: read::VashRead::Sink(ReadSink),
            stderr: read::VashRead::Sink(ReadSink),
            child: child::VashChild::PreExecuted(BuiltinExitStatus::new_failure()),
        }
    }

    pub fn adhoc_process<F, A>(f: F) -> Self
    where
        F: FnOnce(PseudoChild) -> A + Send + 'static,
        A: Future<Output = VashExitStatus> + Send + 'static,
    {
        // i have to use this hack because there's no way to construct a single duplex
        let (stream1, stream2) = duplex(1024);
        let (stdin_read, stdout_write) = split(stream1);
        let (stdout_read, stdin_write) = split(stream2);

        let (stream3, stream4) = duplex(1024);
        let (stderr_read, _) = split(stream3);
        let (_, stderr_write) = split(stream4);

        let child = PseudoChild {
            stdin: stdin_read,
            stdout: stdout_write,
            stderr: stderr_write,
        };

        let handle = tokio::task::spawn(f(child));

        VashProcess {
            stdin: write::VashWrite::Duplex(stdin_write),
            stdout: read::VashRead::Duplex(stdout_read),
            stderr: read::VashRead::Duplex(stderr_read),
            child: child::VashChild::Thread(handle),
        }
    }
}

pub struct PseudoChild {
    pub stdin: ReadHalf<DuplexStream>,
    pub stdout: WriteHalf<DuplexStream>,
    pub stderr: WriteHalf<DuplexStream>,
}
