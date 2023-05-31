use std::{
    io::{self, Stdout, Write},
    panic::PanicInfo,
    path::PathBuf,
};

use cmd::delegate::ExecutionDelegate;
use color_eyre::Result;
use itertools::Itertools;
use nix::sys::signal::Signal;
use once_cell::sync::OnceCell;
use termion::{
    cursor::Goto,
    event::{Key, MouseButton, MouseEvent},
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen, IntoAlternateScreen},
};
use tokio::select;
use tracing_subscriber::prelude::*;

use crate::{
    cmd::delegate::{Delegate, DelegateMessage},
    parse::parse_command,
};

#[macro_use]
extern crate tracing;

pub mod builtins;
pub mod cmd;
pub mod input;
pub mod parse;
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
    pub scroll_y: usize,
    pub scrolled_when_len: Option<usize>,
}

impl State {
    pub fn render<W: Write>(&self, stdout: &mut W) -> io::Result<()> {
        let (_width, height) = termion::terminal_size()?;

        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )?;

        write!(stdout, "{}", self.prompt)?;

        write!(stdout, "{}", self.input)?;

        let output = self.output.lines().collect_vec();

        let len = self.scrolled_when_len.unwrap_or(output.len());

        let available = height as usize - 2;
        let start = len.saturating_sub(available).saturating_sub(self.scroll_y);
        let end = len.saturating_sub(self.scroll_y);

        let lines = &output[start..end];

        for (x, line) in lines.iter().enumerate() {
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

type Term = MouseTerminal<AlternateScreen<RawTerminal<Stdout>>>;

static mut TERMINAL: OnceCell<Option<Term>> = OnceCell::new();

fn term() -> &'static mut Term {
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
    let stdout = std::io::stdout()
        .into_raw_mode()?
        .into_alternate_screen()?
        .into();

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
        scroll_y: 0,
        scrolled_when_len: None,
    };

    trace!("rendering initial state");
    state.render(&mut term().lock())?;

    loop {
        select! {
            Some(msg) = rx.recv() => match msg {
                input::InputMessage::Key(key) => match key {
                    Key::Char('\n') => {
                        let res = parse_command(&state.input);

                        trace!("parsed command: {:?}", res);

                        match res {
                            Ok(plan) => {
                                let exec = plan.execute().await;

                                state.running = Some(ExecutionDelegate::spawn(exec).await);
                            }
                            Err(err) => {
                                state.output.push_str(&format!("error: {err}\n"));
                            }
                        }

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
                    _ => {
                        trace!("unhandled key: {:?}", key);
                    }
                }
                input::InputMessage::Mouse(event) => {
                    match event {
                        MouseEvent::Press(MouseButton::WheelUp, _, _) => {
                            let lines = state.output.lines().count();
                            let new_scroll_y = usize::min(state.scroll_y + 1, lines);
                            if new_scroll_y != 0 {
                                state.scrolled_when_len = Some(lines);
                            }
                            let (_width, height) = termion::terminal_size()?;
                            let available = height as usize - 2;
                            state.scroll_y = new_scroll_y.min(lines.saturating_sub(available));
                        }
                        MouseEvent::Press(MouseButton::WheelDown, _, _) => {
                            state.scroll_y = usize::max(state.scroll_y.saturating_sub(1), 0);
                            if state.scroll_y == 0 {
                                state.scrolled_when_len = None;
                            }
                        }
                        _ => {
                            trace!("unhandled mouse event: {:?}", event);
                        }
                    }
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
