use std::{
    io::{BufRead, Write},
    process::Stdio,
    thread,
};

use async_recursion::async_recursion;
use tokio::{
    io::{AsyncRead, AsyncWrite, BufReader, BufWriter},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
};

use super::execution_plan::{ExecutionPlan, PipeType};
use crate::{
    builtins::{BuiltinCommand, BuiltinCommands},
    prelude::*,
    process::VashProcess,
};

impl ExecutionPlan {
    #[async_recursion(?Send)]
    pub async fn execute(&self) -> VashProcess {
        match self {
            Self::Execute(cmd) => {
                let mut cmd = cmd.trim().split_ascii_whitespace();
                // SAFETY: empty commands will be Self::NoOp
                let bin = cmd.next().unwrap();
                let args = cmd.collect::<Vec<_>>();

                if let Some(builtin) = BuiltinCommands::from_name(bin) {
                    return builtin.execute(&args).await;
                }

                let mut cmd = Command::new(bin);
                cmd.args(args);

                cmd.stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                trace!("spawning command: {:?}", cmd);

                let mut child = cmd.spawn().unwrap();

                let stdin = child.stdin.take().unwrap().into();
                let stdout = child.stdout.take().unwrap().into();
                let stderr = child.stderr.take().unwrap().into();

                VashProcess {
                    stdin,
                    stdout,
                    stderr,
                    child: child.into(),
                }
            }
            Self::And(left, right) => {
                trace!("AND: executing left");
                let mut left = left.execute().await;

                trace!("AND: waiting for left to finish");
                let res = left.child.wait().await;

                trace!("AND: left finished, checking exit status");
                match res {
                    Ok(exit) if exit.success() => right.execute().await,
                    Ok(_) => left,
                    Err(_) => left,
                }
            }
            Self::Or(left, right) => {
                trace!("OR: executing left");
                let mut left = left.execute().await;

                trace!("OR: waiting for left to finish");
                let res = left.child.wait().await;

                trace!("OR: left finished, checking exit status");
                match res {
                    Ok(exit) if exit.success() => left,
                    Ok(_) => right.execute().await,
                    Err(_) => left,
                }
            }
            Self::Pipe(left, right) => {
                trace!("spawning right side of pipe");
                let mut right = right.execute().await;
                trace!("spawning left side of pipe");
                let mut left = left.execute().await;

                trace!("spawning pipe thread");
                tokio::task::spawn(async move {
                    tokio::io::copy(&mut left.stdout, &mut right.stdin)
                        .await
                        .unwrap();
                    trace!("pipe thread finished");
                });

                VashProcess {
                    stdin: left.stdin,
                    stdout: right.stdout,
                    stderr: right.stderr,
                    child: right.child,
                }
            }
            Self::RedirectPipe(left, dest) => {
                let mut left = left.execute().await;

                let from: Box<dyn AsyncRead> = match &dest.from {
                    PipeType::Stdout => Box::new(left.stdout),
                    PipeType::Stderr => Box::new(left.stderr),
                    PipeType::File(path) => {
                        let file = tokio::fs::File::create(path).await.unwrap();
                        Box::new(file)
                    }
                    _ => unreachable!("cannot pipe from null or stdin"),
                };

                let to: Box<dyn AsyncWrite> = match &dest.to {
                    PipeType::Null => Box::new(tokio::io::sink()),
                    PipeType::Stdin => Box::new(left.stdin),
                    PipeType::File(path) => {
                        let file = tokio::fs::File::create(path).await.unwrap();
                        Box::new(file)
                    }
                    _ => unreachable!("cannot pipe to stdout or stderr"),
                };

                // need to update execution context to use an abstraction over
                // these streams so I can replace some with a sink, union, etc.
                unimplemented!("redirect pipe");
            }
            _ => unimplemented!(),
        }
    }
}
