use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::process::{status::BuiltinExitStatus, VashProcess};

use super::BuiltinCommand;

#[derive(Default)]
pub struct Pwd;

#[async_trait]
impl BuiltinCommand for Pwd {
    fn name(&self) -> &'static str {
        "pwd"
    }

    async fn execute(&self, _args: &[&str]) -> VashProcess {
        VashProcess::adhoc_process(|child| async {
            let mut stdout = child.stdout;

            let cwd = std::env::current_dir().unwrap();

            let output = format!("{}\n", cwd.display());

            stdout.write_all(output.as_bytes()).await.unwrap();

            BuiltinExitStatus::new_success().into()
        })
    }
}
