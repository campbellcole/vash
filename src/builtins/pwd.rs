use async_trait::async_trait;

use crate::process::VashProcess;

use super::BuiltinCommand;

#[derive(Default)]
pub struct Pwd;

#[async_trait]
impl BuiltinCommand for Pwd {
    fn name(&self) -> &'static str {
        "pwd"
    }

    async fn execute(&self, _args: &[&str]) -> VashProcess {
        let cwd = std::env::current_dir().unwrap();

        VashProcess {
            stdout: crate::process::read::VashRead::Canned(
                cwd.to_string_lossy().as_bytes().to_vec(),
            ),
            ..VashProcess::sink()
        }
    }
}
