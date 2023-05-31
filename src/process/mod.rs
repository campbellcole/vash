use std::process::ExitStatus;

use self::{read::ReadSink, status::BuiltinExitStatus};

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
}
