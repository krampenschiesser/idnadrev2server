// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use thread_local::ThreadLocal;
use std::clone::Clone;
use std::error::Error;
use std::fmt::Debug;

pub struct SenderWrapper<Command: Send, Response: Send> {
    sender_share: Mutex<Sender<(Sender<Result<Response, String>>, Command)>>,
    sender_local: ThreadLocal<Sender<(Sender<Result<Response, String>>, Command)>>,
}

pub struct ActorControl<Command: Send, Response: Send> {
    sender_wrapper: SenderWrapper<Command, Response>,
    shutdown_command: Command,
}

pub struct Actor<Command, Response, State> {
    shutdown_command: Command,
    handler: fn(Command, &mut State) -> Result<Response, String>,
    state: State,

    receiver: Receiver<(Sender<Result<Response, String>>, Command)>,
    sender: Sender<(Sender<Result<Response, String>>, Command)>,
}

pub trait SendSync<Command,Response>
    where
        Command: Clone + Eq + Debug + PartialEq + Send,
        Response: Clone + Eq + Debug + PartialEq + Send,
{
    fn send_sync(&self, cmd: Command) -> Result<Response, String>;
}

impl<Command, Response>  SendSync<Command, Response> for ActorControl<Command, Response>
    where
        Command: Clone + Eq + Debug + PartialEq + Send,
        Response: Clone + Eq + Debug + PartialEq + Send,
{
    fn send_sync(&self, cmd: Command) -> Result<Response, String> {
        let (s2, r2) = channel();
        self.get_sender().send((s2, cmd));

        let result = r2.recv();
        let result = result.map_err(|e| e.description().to_string());
        match result {
            Err(string) => Err(string),
            Ok(result) => {
                match result {
                    Err(string) => Err(string),
                    Ok(response) => Ok(response)
                }
            }
        }
    }
}
impl<Command, Response>  SendSync<Command, Response> for SenderWrapper<Command, Response>
    where
        Command: Clone + Eq + Debug + PartialEq + Send,
        Response: Clone + Eq + Debug + PartialEq + Send,
{
    fn send_sync(&self, cmd: Command) -> Result<Response, String> {
        let (s2, r2) = channel();
        self.get_sender().send((s2, cmd));

        let result = r2.recv();
        let result = result.map_err(|e| e.description().to_string());
        match result {
            Err(string) => Err(string),
            Ok(result) => {
                match result {
                    Err(string) => Err(string),
                    Ok(response) => Ok(response)
                }
            }
        }
    }
}

impl<Command, Response> ActorControl<Command, Response>
    where
        Command: Clone + Eq + Debug + PartialEq + Send,
        Response: Clone + Eq + Debug + PartialEq + Send,
{
    pub fn stop(&self) {
        let (s1, r1) = channel();
        self.get_sender().send((s1, self.shutdown_command.clone()));
        r1.recv();//wait for shutdown
    }


    fn get_sender(&self) -> &Sender<(Sender<Result<Response, String>>, Command)> {
        self.sender_wrapper.get_sender()
    }

    pub fn clone_sender(&self) -> SenderWrapper<Command,Response> {
        self.sender_wrapper.clone()
    }
}

impl<Command, Response> SenderWrapper<Command, Response>
    where Command: Clone + Eq + Debug + Send,
          Response: Clone + Eq + Debug + Send
{
    pub fn get_sender(&self) -> &Sender<(Sender<Result<Response, String>>, Command)> {
        self.sender_local.get_or(|| {
            let sender = self.sender_share.lock().unwrap();
            Box::new(sender.clone())
        })
    }
}

impl<Command, Response>  Clone for SenderWrapper<Command, Response>
    where Command: Clone + Eq + Debug + Send,
          Response: Clone + Eq + Debug + Send
{
    fn clone(&self) -> Self {
        let lock = self.sender_share.lock().unwrap();
        let sender = lock.clone();
        SenderWrapper{sender_share: Mutex::new(sender),sender_local: ThreadLocal::new()}
    }
}

impl<Command, Response, State> Actor<Command, Response, State>
    where
        Command: Clone + Eq + Debug + Send,
        Response: Clone + Eq + Debug + Send,
{
    pub fn start(state: State, handler: fn(Command, &mut State) -> Result<Response, String>, shutdown_command: Command) -> (Actor<Command, Response, State>, ActorControl<Command, Response>) {
        let (sender, receiver) = channel();
        let actor = Actor { shutdown_command: shutdown_command.clone(), handler: handler, sender: sender.clone(), receiver: receiver, state: state };
        let actor_control = ActorControl { shutdown_command: shutdown_command, sender_wrapper: SenderWrapper { sender_share: Mutex::new(sender), sender_local: ThreadLocal::new() } };
        (actor, actor_control)
    }

    pub fn run(&mut self) {
        info!("Starting work loop");
        let mut shutdown = false;
        let ref close_cmd = self.shutdown_command;
        while !shutdown {
            let result = self.receiver.recv();
            if result.is_ok() {
                let (sender, cmd) = result.unwrap();

                if cmd == *close_cmd {
                    shutdown = true;
                    sender.send(Err("".to_string()));
                    return
                }

                let handle = self.handler;
                let resp = handle(cmd, &mut self.state);
                sender.send(resp);
            } else {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[derive(Clone, Eq, PartialEq, Debug)]
    enum TestCmd {
        Hello,
        Shutdown,
    }

    #[derive(Clone, Eq, PartialEq, Debug)]
    enum TestResponse {
        World { content: String },
        DidShtdown,
    }

    struct State {
        counter: u8,
    }

    fn handle(cmd: TestCmd, state: &mut State) -> Result<TestResponse, String> {
        info!("Handling {:?}", cmd);
        {
            state.counter = state.counter + 1;
        }
        match cmd {
            TestCmd::Hello => Ok(TestResponse::World { content: format!("Count: {}", state.counter) }),
            _ => Err("No known command!".to_string()),
        }
    }

    #[test]
    fn communicate() {
        let state = State { counter: 0 };
        info!("Communcation test!");
        let (mut actor, control) = Actor::start(state, handle, TestCmd::Shutdown);
        thread::spawn(move || actor.run());
        let resp = control.send_sync(TestCmd::Hello).unwrap();
        assert_eq!(TestResponse::World { content: "Count: 1".to_string() }, resp);
        control.stop();
    }
}

