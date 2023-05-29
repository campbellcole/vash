use std::io;

use thiserror::Error;
use tokio::io::AsyncWrite;

const TOKEN_PIPE: char = '|';
const TOKEN_OR: &str = "||";
const TOKEN_AND: &str = "&&";
const TOKEN_BACKGROUND: char = '&';
const TOKEN_REDIRECT_OUT: char = '>';
const TOKEN_REDIRECT_IN: char = '<';

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

#[derive(Debug)]
pub struct PipeRedirection {
    pub from: PipeType,
    pub to: PipeType,
    pub append: bool,
}

#[derive(Debug)]
pub enum PipeType {
    Stdout,
    Stderr,
    Stdin,
    Null,
    File(String),
}

#[derive(Debug, Error)]
pub enum CommandSyntaxError {
    #[error("pipe redirection must follow a command")]
    RedirectBeforeCommand,
    #[error("unterminated string")]
    UnterminatedString,
}

impl ExecutionPlan {
    pub fn parse(cmd: &str) -> Result<Self, CommandSyntaxError> {
        let cmd = cmd.trim();

        if cmd.is_empty() {
            return Ok(Self::NoOp);
        }

        if let Some(position) = cmd.find(TOKEN_OR) {
            let left = Self::parse(&cmd[..position])?;
            let right = Self::parse(&cmd[position + 2..])?;
            return Ok(Self::Or(Box::new(left), Box::new(right)));
        }

        if let Some(position) = cmd.find(TOKEN_AND) {
            let left = Self::parse(&cmd[..position])?;
            let right = Self::parse(&cmd[position + 2..])?;
            return Ok(Self::And(Box::new(left), Box::new(right)));
        }

        if let Some(position) = cmd.find(TOKEN_PIPE) {
            let left = Self::parse(&cmd[..position])?;
            let right = Self::parse(&cmd[position + 1..])?;
            return Ok(Self::Pipe(Box::new(left), Box::new(right)));
        }

        if cmd.ends_with(TOKEN_BACKGROUND) {
            let left = Self::parse(&cmd[..cmd.len() - 1])?;
            return Ok(Self::Background(Box::new(left)));
        }

        if let Some(position) = cmd.find(TOKEN_REDIRECT_OUT) {
            if position == 0 {
                // we are going to subtract 1 from this position,
                // so if it is 0, we will underflow
                return Err(CommandSyntaxError::RedirectBeforeCommand);
            }

            // check for another > after this character (find always returns the first instance)
            let append = cmd.chars().nth(position + 1) == Some(TOKEN_REDIRECT_OUT);
            let append_offset = if append { 1 } else { 0 };

            // check for a 2 before this character
            let (from, offset) = if cmd.chars().nth(position - 1) == Some('2') {
                (PipeType::Stderr, 1)
            } else {
                (PipeType::Stdout, 0)
            };
            let left = Self::parse(&cmd[..position - offset])?;
            let right = {
                let right = cmd[position + 1 + append_offset..].trim();
                let end = if right.starts_with('"') {
                    right[1..]
                        .find('"')
                        .ok_or(CommandSyntaxError::UnterminatedString)?
                } else {
                    right.find(' ').unwrap_or(right.len())
                };
                &right[..end]
            };
            return Ok(Self::RedirectPipe(
                Box::new(left),
                PipeRedirection {
                    from,
                    to: PipeType::File(right.to_string()),
                    append,
                },
            ));
        }

        Ok(Self::Execute(cmd.to_string()))
    }
}
