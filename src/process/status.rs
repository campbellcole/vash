use std::process::ExitStatus;

pub enum VashExitStatus {
    Process(ExitStatus),
    Builtin(BuiltinExitStatus),
}

impl From<ExitStatus> for VashExitStatus {
    fn from(value: ExitStatus) -> Self {
        Self::Process(value)
    }
}

impl From<BuiltinExitStatus> for VashExitStatus {
    fn from(value: BuiltinExitStatus) -> Self {
        Self::Builtin(value)
    }
}

impl VashExitStatus {
    pub fn code(&self) -> Option<i32> {
        match self {
            Self::Process(status) => status.code(),
            Self::Builtin(status) => status.code(),
        }
    }

    pub fn success(&self) -> bool {
        match self {
            Self::Process(status) => status.success(),
            Self::Builtin(status) => status.success(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BuiltinExitStatus(Option<i32>);

impl BuiltinExitStatus {
    pub fn new_success() -> Self {
        Self(Some(0))
    }

    pub fn new_failure() -> Self {
        Self(Some(1))
    }

    pub fn success(&self) -> bool {
        self.0 == Some(0)
    }

    pub fn failure(&self) -> bool {
        !self.success()
    }

    pub fn code(&self) -> Option<i32> {
        self.0
    }
}
