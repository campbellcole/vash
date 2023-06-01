use async_trait::async_trait;

use crate::process::VashProcess;

use super::BuiltinCommand;

#[derive(Default)]
pub struct Exit;

#[async_trait]
impl BuiltinCommand for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }

    async fn execute(&self, _args: &[&str]) -> VashProcess {
        // TODO: exit code
        panic!("goodbye")
    }
}
