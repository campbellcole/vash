use async_trait::async_trait;

use crate::process::VashProcess;

use super::BuiltinCommand;

#[derive(Default)]
pub struct Cd;

#[async_trait]
impl BuiltinCommand for Cd {
    fn name(&self) -> &'static str {
        "cd"
    }

    async fn execute(&self, args: &[&str]) -> VashProcess {
        trace!("executing cd builtin: {args:?}");

        let path = args.first().map(|s| s.to_string()).unwrap_or_default();

        let full_path = std::env::current_dir()
            .unwrap()
            .join(path)
            .canonicalize()
            .unwrap();

        trace!("cd: {:?}", full_path);

        match std::env::set_current_dir(full_path) {
            Ok(_) => VashProcess::sink(),
            Err(err) => {
                error!("failed to cd: {}", err);
                VashProcess::sink_failure()
            }
        }
    }
}
