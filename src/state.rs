use std::{io::Write, path::PathBuf};

use color_eyre::Result;
use itertools::Itertools;
use nix::sys::signal::Signal;
use termion::cursor::Goto;

use crate::{
    cmd::delegate::{Delegate, DelegateCommand, DelegateMessage, ExecutionDelegate},
    parse::parse_command,
};

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
    pub fn render<W: Write>(&self, stdout: &mut W) -> Result<()> {
        write!(stdout, "vash> ")?;

        Ok(())
    }

    pub async fn execute(&mut self) -> Result<()> {
        let res = parse_command(&self.input);

        self.push_history();

        let plan = res?;

        trace!("parsed command: {:?}", plan);

        let exec = plan.execute().await;

        self.running = Some(ExecutionDelegate::spawn(exec).await);

        Ok(())
    }

    pub fn push_history(&mut self) {
        if !matches!(self.history.last(), Some(last) if last == &self.input) {
            self.history.push(std::mem::take(&mut self.input));
        }
        self.history_pos = 0;
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    pub fn mutate_history_pos(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                if !self.history.is_empty() {
                    self.input = self.history[self.history.len() - 1 - self.history_pos].clone();
                    self.history_pos = usize::min(self.history_pos + 1, self.history.len() - 1);
                }
            }
            Direction::Down => {
                if !self.history.is_empty() {
                    self.input = self.history[self.history.len() - 1 - self.history_pos].clone();
                    let before = self.history_pos;
                    self.history_pos = self.history_pos.saturating_sub(1);
                    if self.history_pos == before {
                        self.input.clear();
                    }
                }
            }
        }
    }

    pub fn terminate(&mut self) -> bool {
        if let Some(running) = self.running.take() {
            running.send(DelegateCommand::Signal(Signal::SIGTERM));
            true
        } else {
            false
        }
    }

    pub fn mutate_scroll_pos(&mut self, direction: Direction) -> Result<()> {
        match direction {
            Direction::Up => {
                let lines = self.output.lines().count();
                let new_scroll_y = usize::min(self.scroll_y + 1, lines);
                if new_scroll_y != 0 {
                    self.scrolled_when_len = Some(lines);
                }
                let (_width, height) = termion::terminal_size()?;
                let available = height as usize - 2;
                self.scroll_y = new_scroll_y.min(lines.saturating_sub(available));
            }
            Direction::Down => {
                self.scroll_y = usize::max(self.scroll_y.saturating_sub(1), 0);
                if self.scroll_y == 0 {
                    self.scrolled_when_len = None;
                }
            }
        }

        Ok(())
    }

    pub fn push_output(&mut self, output: &str) {
        self.output.push_str(output);
    }

    pub async fn poll(&mut self) {
        if let Some(msg) = self.running.recv().await {
            match msg {
                DelegateMessage::Stdout(data) => {
                    self.push_output(&String::from_utf8_lossy(&data));
                }
                DelegateMessage::Stderr(data) => {
                    self.push_output(&String::from_utf8_lossy(&data));
                }
                DelegateMessage::Exit(code) => {
                    self.push_output(&format!("exit: {:#?}\n", code));
                    self.running = None;
                }
                DelegateMessage::Error(err) => {
                    self.push_output(&format!("error: {:#?}\n", err));
                    self.running = None;
                }
            }
        }
    }
}

pub enum Direction {
    Up,
    Down,
}
