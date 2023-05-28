// use std::process::Child;

// use exec_thread::ChildMessage;
// use iced::keyboard::KeyCode;
// use iced::widget::runtime::command::Action;
// use iced::widget::{column, container, progress_bar, slider, text_input};
// use iced::{
//     executor, font, keyboard, subscription, Application, Command, Element, Event, Font, Length,
//     Sandbox, Settings, Subscription, Theme,
// };

use iced::futures::select;

pub mod cmd;
pub mod cmd_thread;
pub mod exec_thread;

// const MONO_FONT: Font = Font::with_name("Anonymous Pro");

// pub fn main() -> iced::Result {
//     Terminal::run(Settings::default())
// }

// #[derive(Default)]
// struct Terminal {
//     prompt: String,
//     input: String,
//     stdout: String,
//     sender: Option<exec_thread::Sender>,
//     error: Option<String>,
// }

// #[derive(Debug, Clone)]
// pub enum Message {
//     Event(Event),
//     ExecMessage(ChildMessage),
//     FontLoaded(Result<(), font::Error>),
// }

// impl Application for Terminal {
//     type Message = Message;
//     type Theme = Theme;
//     type Executor = executor::Default;
//     type Flags = ();

//     fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
//         (
//             Self::default(),
//             font::load(include_bytes!("../fonts/AnonymousPro-Regular.ttf").as_ref())
//                 .map(Message::FontLoaded),
//         )
//     }

//     fn title(&self) -> String {
//         String::from("VaTTY")
//     }

//     fn subscription(&self) -> Subscription<Self::Message> {
//         Subscription::batch([
//             exec_thread::exec_thread().map(Message::ExecMessage),
//             subscription::events().map(Message::Event),
//         ])
//     }

//     fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
//         match message {
//             Message::ExecMessage(msg) => match msg {
//                 ChildMessage::Ready(sender) => {
//                     self.sender = Some(sender);
//                 }
//                 ChildMessage::ProcessExited(code) => {
//                     self.error = Some(format!("Process exited with code {:?}", code));
//                 }
//                 _ => {}
//             },
//             Message::Event(ev) => match ev {
//                 Event::Keyboard(key) => match key {
//                     keyboard::Event::CharacterReceived(c) => {
//                         self.input.push(c);
//                     }
//                     keyboard::Event::KeyPressed {
//                         key_code,
//                         modifiers,
//                     } => {
//                         if key_code == KeyCode::Enter {
//                             let mut split = self.input.split_whitespace();
//                             let cmd = split.next().unwrap();
//                             let args = split.collect::<Vec<&str>>();

//                             let mut cmd = std::process::Command::new(cmd);
//                             cmd.args(args);

//                             match cmd.spawn() {
//                                 Ok(child) => {
//                                     self.child = Some(child);
//                                 }
//                                 Err(e) => {
//                                     self.error = Some(e.to_string());
//                                 }
//                             }
//                         }
//                     }
//                     _ => {}
//                 },
//                 _ => {}
//             },
//             _ => {}
//         }

//         Command::none()
//     }

//     fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
//         let text = text_input(&self.prompt, &self.input).font(MONO_FONT);

//         let content = column![text];

//         container(content)
//             .width(Length::Fill)
//             .height(Length::Fill)
//             .center_x()
//             .center_y()
//             .into()
//     }
// }

#[tokio::main]
async fn main() {
    use iced::futures::StreamExt;
    use tokio::process::Command;

    use crate::cmd_thread::CommandResponse;

    let mut cmd = Command::new("ls");
    cmd.arg("-l").arg("-a");

    let (mut tx, mut rx) = cmd_thread::cmd_thread(cmd);

    // tx.unbounded_send(cmd_thread::CommandMessage::Kill).unwrap();

    loop {
        select! {
            msg = rx.select_next_some()=> {
                match msg {
                    CommandResponse::ExitCode(code) => {
                        println!("process exited with code: {:?}", code);
                    }
                    CommandResponse::Error(err) => {
                        println!("error: {err}");
                    }
                    CommandResponse::Stdout(buf) => {
                        let stdout = String::from_utf8_lossy(&buf);
                        print!("{}", stdout);
                    }
                    CommandResponse::Stderr(buf) => {
                        let stderr = String::from_utf8_lossy(&buf);
                        print!("{}", stderr);
                    }
                }
            }
            complete => {
                println!("complete");
                break;
            }
        }
    }
}
