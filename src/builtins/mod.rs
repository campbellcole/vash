use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use strum::{EnumIter, IntoEnumIterator};

use crate::process::VashProcess;

pub mod cd;

#[async_trait]
#[enum_dispatch(BuiltinCommands)]
pub trait BuiltinCommand {
    fn name(&self) -> &'static str;
    async fn execute(&self, args: &[&str]) -> VashProcess;
}

#[enum_dispatch]
#[derive(EnumIter)]
pub enum BuiltinCommands {
    Cd(cd::Cd),
}

impl BuiltinCommands {
    pub fn from_name(name: &str) -> Option<Self> {
        Self::iter().find(|cmd| cmd.name() == name)
    }
}
