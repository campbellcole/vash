use std::{
    io::{self, Stdout, Write},
    panic::PanicInfo,
};

use cmd::execute::ExecutionReceiver;
use input::InputSender;
use once_cell::sync::OnceCell;
use termion::{
    cursor::{DetectCursorPos, Goto},
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen, IntoAlternateScreen},
};
use tokio::{select, sync::mpsc::unbounded_channel};

use crate::cmd::execute::ExecutionMessage;

pub mod cmd;
pub mod input;
pub mod prelude;

pub struct State {
    pub prompt: String,
    pub input: String,
    pub output: String,
    pub running: ExecutionReceiver,
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
async fn main() {
    std::panic::set_hook(Box::new(panic));

    let stdout = std::io::stdout()
        .into_raw_mode()
        .unwrap()
        .into_alternate_screen()
        .unwrap();

    #[allow(unused_must_use)]
    unsafe {
        TERMINAL.set(Some(stdout));
    }

    let mut rx = input::spawn_input_thread().await;

    let (etx, erx) = unbounded_channel();

    let mut state = State {
        prompt: "vash> ".into(),
        input: String::new(),
        output: String::new(),
        running: erx,
    };

    state.render(&mut term().lock()).unwrap();

    loop {
        select! {
            Some(msg) = rx.recv() => match msg {
                input::InputMessage::Char(c) => {
                    state.input.push(c);
                }
                input::InputMessage::Enter => {
                    state.output.push_str(&format!("input: {:#?}\n", state.input));

                    let plan = state
                        .input
                        .parse::<cmd::execution_plan::ExecutionPlan>()
                        .unwrap();

                    state.output.push_str(&format!("plan: {:#?}\n", plan));

                    cmd::execute::execute(plan, etx.clone());

                    state.input.clear();
                }
                input::InputMessage::Backspace => {
                    state.input.pop();
                }
                input::InputMessage::CtrlC => {
                    break;
                }
                input::InputMessage::Error(err) => {
                    state.output.push_str(&format!("error: {err}\n"));
                    break;
                }
            },
            Some(msg) = state.running.recv() => match msg {
                ExecutionMessage::Stdout(line) => {
                    state.output.push_str(&format!("{}\n", String::from_utf8_lossy(&line)));
                }
                ExecutionMessage::Stderr(line) => {
                    state.output.push_str(&format!("{}\n", String::from_utf8_lossy(&line)));
                }
                ExecutionMessage::Error(err) => {
                    state.output.push_str(&format!("error: {err}\n"));
                    break;
                }
            }
        }

        state.render(&mut term().lock()).unwrap();
    }
}
