use logos::{Lexer, Logos};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Default, Error)]
pub enum LexerError {
    #[default]
    #[error("unknown token")]
    UnknownToken,
    #[error("unterminated string")]
    UnterminatedString,
}

#[derive(Debug, PartialEq, Logos)]
#[logos(skip r"[ \t\n\f]+", error = LexerError)]
pub enum Token<'a> {
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token(";")]
    Semi,
    #[token("|")]
    Pipe,
    #[token(">>")]
    Append,
    #[token(">")]
    Write,
    #[token("<<")]
    HereDoc,
    #[token("<")]
    Read,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("fi")]
    Fi,
    #[token("while")]
    While,
    #[token("do")]
    Do,
    #[token("done")]
    Done,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("function")]
    Function,
    #[token("case")]
    Case,
    #[token("esac")]
    Esac,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,

    #[regex(r"[a-zA-Z0-9_\-\./]*", priority = 2)]
    Identifier(&'a str),
    #[regex(r#""([^"\\]|\\.)*""#, quoted_str_callback)]
    DoubleQuotedString(String),
    #[regex(r"'[^']*'", single_quoted_str_callback)]
    SingleQuotedString(String),
    #[regex(r"#.*")]
    Comment(&'a str),
    #[regex(r"\d+", |lex| lex.slice().parse().ok())]
    Number(i64),
}

fn quoted_str_callback<'a>(lex: &mut Lexer<'a, Token<'a>>) -> String {
    let slice = lex.slice();
    // replace escaped quotes with unescaped quotes
    slice[1..slice.len() - 1].replace("\\\"", "\"")
}

fn single_quoted_str_callback<'a>(lex: &mut Lexer<'a, Token<'a>>) -> String {
    let slice = lex.slice();
    // replace escaped quotes with unescaped quotes
    slice[1..slice.len() - 1].replace("\\'", "'")
}
