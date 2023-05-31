use async_trait::async_trait;

use crate::process::{status::BuiltinExitStatus, VashProcess};

use super::BuiltinCommand;

#[derive(Default)]
pub struct Cd;

#[async_trait]
impl BuiltinCommand for Cd {
    fn name(&self) -> &'static str {
        "cd"
    }

    async fn execute(&self, args: &[&str]) -> VashProcess {
        let dir = args.first().unwrap_or(&".").to_string();
        VashProcess::adhoc_process(|_child| async {
            std::env::set_current_dir(dir).unwrap();

            BuiltinExitStatus::new_success().into()
        })
    }
}
