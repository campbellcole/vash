use std::str::FromStr;

use thiserror::Error;

#[derive(Debug)]
pub enum ExecutionPlan {
    Execute(String),
    Pipe(Box<ExecutionPlan>, Box<ExecutionPlan>),
    And(Box<ExecutionPlan>, Box<ExecutionPlan>),
    Or(Box<ExecutionPlan>, Box<ExecutionPlan>),
    Background(Box<ExecutionPlan>),
    RedirectPipe(Box<ExecutionPlan>, PipeRedirection),
    NoOp,
}

#[derive(Debug, Error)]
pub enum CommandSyntaxError {}

impl FromStr for ExecutionPlan {
    type Err = CommandSyntaxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_whitespace();
        let mut current_cmd = String::new();
        let mut execution_plan = ExecutionPlan::NoOp;

        while let Some(token) = tokens.next() {
            match token {
                "|" | "&&" | "&" | "||" | ">" | "2>" => {
                    let exec = ExecutionPlan::Execute(std::mem::take(&mut current_cmd));
                    if matches!(execution_plan, ExecutionPlan::NoOp) {
                        execution_plan = exec;
                    } else {
                        execution_plan = match token {
                            "|" => ExecutionPlan::Pipe(Box::new(execution_plan), Box::new(exec)),
                            "&&" => ExecutionPlan::And(Box::new(execution_plan), Box::new(exec)),
                            "&" => ExecutionPlan::Background(Box::new(exec)),
                            "||" => ExecutionPlan::Or(Box::new(execution_plan), Box::new(exec)),
                            ">" => ExecutionPlan::RedirectPipe(
                                Box::new(execution_plan),
                                PipeRedirection {
                                    from: PipeType::Stdout,
                                    to: PipeType::File(tokens.next().unwrap().to_string()),
                                },
                            ),
                            "2>" => ExecutionPlan::RedirectPipe(
                                Box::new(execution_plan),
                                PipeRedirection {
                                    from: PipeType::Stderr,
                                    to: PipeType::File(tokens.next().unwrap().to_string()),
                                },
                            ),
                            _ => unreachable!(),
                        }
                    }
                }
                _ => {
                    if !current_cmd.is_empty() {
                        current_cmd.push(' ');
                    }
                    current_cmd.push_str(token);
                }
            }
        }

        if !current_cmd.is_empty() {
            execution_plan = ExecutionPlan::Execute(std::mem::take(&mut current_cmd));
        }

        Ok(execution_plan)
    }
}

#[derive(Debug)]
pub struct PipeRedirection {
    pub from: PipeType,
    pub to: PipeType,
}

#[derive(Debug)]
pub enum PipeType {
    Stdout,
    Stderr,
    Stdin,
    Null,
    File(String),
}
