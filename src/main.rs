use std::{
    io::{Stdout, Write},
    panic::PanicInfo,
};

use color_eyre::Result;
use once_cell::sync::OnceCell;
use termion::{
    event::{Key, MouseButton, MouseEvent},
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen, IntoAlternateScreen},
};
use tokio::select;
use tracing_subscriber::prelude::*;

use crate::state::{Direction, State};

#[macro_use]
extern crate tracing;

pub mod builtins;
pub mod cmd;
pub mod input;
pub mod parse;
pub mod prelude;
pub mod process;
pub mod state;

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
    let mut stdout = std::io::stdout();

    write!(stdout, "vash> ")?;
    stdout.flush()?;

    loop {
        select! {
            Some(msg) = rx.recv() => match msg {
                input::InputMessage::Key(key) => match key {
                    Key::Char('\n') => {
                        state.execute().await?;
                    }
                    Key::Char(c) => {
                        state.push_char(c);
                    }
                    Key::Backspace => {
                        state.pop_char();
                    }
                    Key::Up => {
                        state.mutate_history_pos(Direction::Up);
                    }
                    Key::Down => {
                        state.mutate_history_pos(Direction::Down);
                    }
                    Key::Ctrl('c') => {
                        if !state.terminate() {
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
                            state.mutate_scroll_pos(Direction::Up)?;
                        }
                        MouseEvent::Press(MouseButton::WheelDown, _, _) => {
                            state.mutate_scroll_pos(Direction::Down)?;
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
            _ = state.poll() => {}
        }

        // state.render(&mut term().lock())?;
    }

    Ok(())
}
