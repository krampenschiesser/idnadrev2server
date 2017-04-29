use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::clone::Clone;
use std::error::Error;
use std::fmt::Debug;

pub struct ActorControl<Command, Response> {
    sender: Sender<(Sender<Result<Response, String>>, Command)>,
    shutdown_command: Command,
}

pub struct Actor<Command, Response, State> {
    shutdown_command: Command,
    handler: fn(Command, &mut State) -> Result<Response, String>,
    state: State,

    receiver: Receiver<(Sender<Result<Response, String>>, Command)>,
    sender: Sender<(Sender<Result<Response, String>>, Command)>,
}

impl<Command, Response> ActorControl<Command, Response>
    where
        Command: Clone + Eq + Debug + PartialEq + Send,
        Response: Clone + Eq + Debug + PartialEq + Send,
{
    pub fn stop(&self) {
        let (s1, r1) = channel();
        self.sender.send((s1, self.shutdown_command.clone()));
        r1.recv();//wait for shutdown
    }

    pub fn get_sender(&mut self) -> Sender<(Sender<Result<Response, String>>, Command)> {
        self.sender.clone()
    }

    pub fn send_sync(&self, cmd: Command) -> Result<Response, String> {
        let (s2, r2) = channel();
        self.sender.send((s2, cmd));

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

impl<Command, Response, State> Actor<Command, Response, State>
    where
        Command: Clone + Eq + Debug,
        Response: Clone + Eq + Debug,
{
    pub fn start(state: State, handler: fn(Command, &mut State) -> Result<Response, String>, shutdown_command: Command) -> (Actor<Command, Response, State>, ActorControl<Command, Response>) {
        let (sender, receiver) = channel();
        let actor = Actor { shutdown_command: shutdown_command.clone(), handler: handler, sender: sender.clone(), receiver: receiver, state: state };
        let actor_control = ActorControl { shutdown_command: shutdown_command, sender: sender };
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
        assert_eq!(TestResponse::World{content: "Count: 1".to_string()}, resp);
        control.stop();
    }
}

