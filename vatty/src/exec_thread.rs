// use std::{any::TypeId, io::Write, process::Child};

// use iced::{
//     futures::{channel::mpsc, sink::SinkExt, Stream},
//     subscription, Subscription,
// };
// use once_cell::sync::OnceCell;

// #[derive(Debug, Clone)]
// pub enum ChildCommand {
//     Start(String),
//     Kill,
//     Input(String),
//     IsRunning,
// }

// #[derive(Debug, Clone)]
// pub enum ChildMessage {
//     Ready(Sender),
//     Spawned,
//     ProcessExited(Option<i32>),
//     Error(String),
//     Stdout(String),
//     Stderr(String),
// }

// enum State {
//     Starting,
//     Ready(Rx),
// }

// pub type Sender = mpsc::UnboundedSender<ChildCommand>;
// pub type Receiver = mpsc::UnboundedReceiver<ChildMessage>;

// type Rx = mpsc::UnboundedReceiver<ChildCommand>;
// type Tx = mpsc::UnboundedSender<ChildMessage>;

// pub fn exec_thread() -> Subscription<ChildMessage> {
//     struct ExecThread;

//     subscription::channel(TypeId::of::<ExecThread>(), 100, |mut output| async move {
//         let mut state = State::Starting;
//         let mut child = None::<Child>;
//         loop {
//             match &mut state {
//                 State::Starting => {
//                     let (tx, rx) = mpsc::unbounded();
//                     output.send(ChildMessage::Ready(tx)).await;
//                     state = State::Ready(rx);
//                 }
//                 State::Ready(rx) => {
//                     use iced::futures::StreamExt;

//                     let input = rx.select_next_some().await;

//                     match input {
//                         ChildCommand::Input(input) => {
//                             let Some(child) = child else {
//                                 continue;
//                             };

//                             child.stdin.as_mut().map(|stdin| {
//                                 stdin.write_all(input.as_bytes()).unwrap();
//                             });
//                         }
//                         ChildCommand::Start(cmd) => {
//                             let mut split = cmd.split_whitespace();
//                             let cmd = split.next().unwrap();
//                             let args = split.collect::<Vec<&str>>();

//                             let mut cmd = std::process::Command::new(cmd);
//                             cmd.args(args);

//                             match cmd.spawn() {
//                                 Ok(c) => {
//                                     child = Some(c);
//                                     output.send(ChildMessage::Spawned).await;
//                                 }
//                                 Err(e) => {
//                                     output.send(ChildMessage::Error(e.to_string())).await;
//                                 }
//                             }
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//         }
//     })
// }

// /*
// pub enum Event {
//     Ready(mpsc::Sender<Input>),
//     WorkFinished,
//     // ...
// }

// enum Input {
//     DoSomeWork,
//     // ...
// }

// enum State {
//     Starting,
//     Ready(mpsc::Receiver<Input>),
// }

// fn some_worker() -> Subscription<Event> {
//     struct SomeWorker;

//     subscription::channel(std::any::TypeId::of::<SomeWorker>(), 100, |mut output| async move {
//         let mut state = State::Starting;

//         loop {
//             match &mut state {
//                 State::Starting => {
//                     // Create channel
//                     let (sender, receiver) = mpsc::channel(100);

//                     // Send the sender back to the application
//                     output.send(Event::Ready(sender)).await;

//                     // We are ready to receive messages
//                     state = State::Ready(receiver);
//                 }
//                 State::Ready(receiver) => {
//                     use iced_futures::futures::StreamExt;

//                     // Read next input sent from `Application`
//                     let input = receiver.select_next_some().await;

//                     match input {
//                         Input::DoSomeWork => {
//                             // Do some async work...

//                             // Finally, we can optionally produce a message to tell the
//                             // `Application` the work is done
//                             output.send(Event::WorkFinished).await;
//                         }
//                     }
//                 }
//             }
//         }
//     })
// }
//  */
