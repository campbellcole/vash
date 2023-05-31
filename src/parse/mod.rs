use logos::Logos;
use thiserror::Error;

use crate::cmd::execution_plan::ExecutionPlan;

use self::token::{LexerError, Token};

pub mod token;

#[derive(Debug, Error)]
pub enum CommandParseError {
    #[error("failed to tokenize command")]
    Lexer(Vec<LexerError>),
}

pub fn parse_command(cmd: &str) -> Result<ExecutionPlan, CommandParseError> {
    let tokens = Token::lexer(cmd);

    let tokens = tokens.collect::<Vec<_>>();

    if tokens.iter().any(|r| r.is_err()) {
        return Err(CommandParseError::Lexer(
            tokens.into_iter().filter_map(|r| r.err()).collect(),
        ));
    }

    let tokens = tokens.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();

    let mut current_cmd = Vec::new();
    let mut incomplete = None::<IncompleteOperator>;

    for token in tokens {
        match token {
            Token::Comment(_) => continue,
            Token::Identifier(seg) => {
                current_cmd.push(seg.to_owned());
            }
            Token::DoubleQuotedString(seg) | Token::SingleQuotedString(seg) => {
                current_cmd.push(seg);
            }
            Token::And => {
                incomplete = Some(IncompleteOperator::And(complete(
                    &mut current_cmd,
                    &mut incomplete,
                )))
            }
            Token::Or => {
                incomplete = Some(IncompleteOperator::Or(complete(
                    &mut current_cmd,
                    &mut incomplete,
                )))
            }
            Token::Pipe => {
                incomplete = Some(IncompleteOperator::Pipe(complete(
                    &mut current_cmd,
                    &mut incomplete,
                )))
            }
            _ => todo!("implement token {:?}", token),
        }
    }

    if !current_cmd.is_empty() {
        Ok(complete(&mut current_cmd, &mut incomplete))
    } else {
        Err(CommandParseError::Lexer(Vec::new()))
    }
}

pub enum IncompleteOperator {
    And(ExecutionPlan),
    Or(ExecutionPlan),
    Pipe(ExecutionPlan),
}

fn complete(cmd: &mut Vec<String>, incomplete: &mut Option<IncompleteOperator>) -> ExecutionPlan {
    let binary = cmd.remove(0);
    let args = std::mem::take(cmd);

    let cmd = ExecutionPlan::Execute(binary, args);
    match std::mem::take(incomplete) {
        Some(IncompleteOperator::And(left)) => ExecutionPlan::And(Box::new(left), Box::new(cmd)),
        Some(IncompleteOperator::Or(left)) => ExecutionPlan::Or(Box::new(left), Box::new(cmd)),
        Some(IncompleteOperator::Pipe(left)) => ExecutionPlan::Pipe(Box::new(left), Box::new(cmd)),
        None => cmd,
    }
}
