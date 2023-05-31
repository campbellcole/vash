use std::{
    io::{self, Stdout, Write},
    panic::PanicInfo,
    path::PathBuf,
};

use cmd::delegate::ExecutionDelegate;
use color_eyre::Result;
use input::InputSender;
use nix::sys::signal::Signal;
use once_cell::sync::OnceCell;
use termion::{
    cursor::{DetectCursorPos, Goto},
    event::Key,
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen, IntoAlternateScreen},
};
use tokio::select;
use tracing_subscriber::prelude::*;

use crate::cmd::{
    delegate::{Delegate, DelegateMessage},
    execution_plan::ExecutionPlan,
};

#[macro_use]
extern crate tracing;

pub mod builtins;
pub mod cmd;
pub mod input;
pub mod prelude;
pub mod process;

pub struct State {
    pub prompt: String,
    pub input: String,
    pub history: Vec<String>,
    pub history_pos: usize,
    pub output: String,
    pub running: Option<ExecutionDelegate>,
    pub working_dir: PathBuf,
}

impl State {
    pub fn render<W: Write>(&self, stdout: &mut W) -> io::Result<()> {
        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )?;

        write!(stdout, "{}", self.prompt)?;

        write!(stdout, "{}", self.input)?;

        for (x, line) in self.output.lines().enumerate() {
            write!(stdout, "{}", Goto(1, (x + 2) as u16))?;
            write!(stdout, "{}", line)?;
        }

        write!(
            stdout,
            "{}",
            Goto((self.prompt.len() + self.input.len()) as u16 + 1, 1)
        )?;

        stdout.flush()?;

        Ok(())
    }
}

static mut TERMINAL: OnceCell<Option<AlternateScreen<RawTerminal<Stdout>>>> = OnceCell::new();

fn term() -> &'static mut AlternateScreen<RawTerminal<Stdout>> {
    unsafe { TERMINAL.get_mut().unwrap().as_mut().unwrap() }
}

fn panic(info: &PanicInfo) {
    let stdout = unsafe { std::mem::take(TERMINAL.get_mut().unwrap()) }.unwrap();

    drop(stdout);

    println!("panic: {:?}", info);

    std::process::exit(1);
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let (writer, _guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::never(".", "logs"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(writer))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_error::ErrorLayer::default())
        .init();

    color_eyre::install()?;

    trace!("preparing terminal");
    let stdout = std::io::stdout().into_raw_mode()?.into_alternate_screen()?;

    #[allow(unused_must_use)]
    unsafe {
        TERMINAL.set(Some(stdout));
    }

    std::panic::set_hook(Box::new(panic));

    trace!("spawning input thread");
    let mut rx = input::spawn_input_thread().await;

    let mut state = State {
        prompt: "vash> ".into(),
        input: String::new(),
        history: Vec::new(),
        history_pos: 0,
        output: String::new(),
        running: None,
        working_dir: std::env::current_dir()?,
    };

    trace!("rendering initial state");
    state.render(&mut term().lock())?;

    loop {
        select! {
            Some(msg) = rx.recv() => match msg {
                input::InputMessage::Event(key) => match key {
                    Key::Char('\n') => {
                        let plan = ExecutionPlan::parse(&state.input)?;

                        trace!(?plan, "execution plan");

                        let exec = plan.execute().await;

                        state.running = Some(ExecutionDelegate::spawn(exec).await);

                        state.history.push(std::mem::take(&mut state.input));
                    }
                    Key::Char(c) => {
                        state.input.push(c);
                    }
                    Key::Backspace => {
                        state.input.pop();
                    }
                    Key::Up => {
                        if !state.history.is_empty() {
                            state.input = state.history[state.history.len() - 1 - state.history_pos].clone();
                            state.history_pos = usize::min(state.history_pos + 1, state.history.len() - 1);
                        }
                    }
                    Key::Down => {
                        if !state.history.is_empty() {
                            state.input = state.history[state.history.len() - 1 - state.history_pos].clone();
                            let before = state.history_pos;
                            state.history_pos = state.history_pos.saturating_sub(1);
                            if state.history_pos == before {
                                state.input.clear();
                            }
                        }
                    }
                    Key::Ctrl('c') => {
                        if let Some(running) = state.running.take() {
                            running.send(cmd::delegate::DelegateCommand::Signal(Signal::SIGTERM));
                        } else {
                            break;
                        }
                    }
                    _ => {}
                }
                input::InputMessage::Error(err) => {
                    state.output.push_str(&format!("error: {err}\n"));
                    break;
                }
            },
            Some(msg) = state.running.recv() => match msg {
                DelegateMessage::Stdout(data) => {
                    state.output.push_str(&String::from_utf8_lossy(&data));
                }
                DelegateMessage::Stderr(data) => {
                    state.output.push_str(&String::from_utf8_lossy(&data));
                }
                DelegateMessage::Exit(code) => {
                    state.output.push_str(&format!("exit: {:#?}\n", code));
                    state.running = None;
                }
                DelegateMessage::Error(err) => {
                    state.output.push_str(&format!("error: {:#?}\n", err));
                    state.running = None;
                }
            }
        }

        state.render(&mut term().lock())?;
    }

    let stdout = unsafe { std::mem::take(TERMINAL.get_mut().unwrap()) }.unwrap();

    drop(stdout);

    Ok(())
}
