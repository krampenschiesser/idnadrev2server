use std::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;
use repository::{RepositoryFile};

use actor::{Actor, ActorControl};

#[derive(Debug, Clone)]
pub enum Cmd {
    CreateRepository(String),
    GetFile(Uuid, Uuid),
    UpdateFileHeader(Uuid, Uuid, u32, RepositoryFile),
    UpdateFileContent(Uuid, Uuid, Vec<u8>),
    UpdateFile(Uuid, Uuid, u32, RepositoryFile, Vec<u8>),

    Shutdown,
}

#[derive(Debug, Clone)]
pub enum Response {
    CreatedRepository(Uuid, String),
    UpdatedFile(Uuid, Uuid, u32),
    DeleteFile(Uuid, Uuid, u32),
    FileContents(Uuid, Uuid, u32, Vec<u8>),
    FileHeader(Uuid, Uuid, u32, RepositoryFile),

    NoSuchFile,
    RepositoryClosed,
    ShutdownSuccessful,
}

#[derive(Debug)]
pub struct RepositoryService {
    receiver: Receiver<(Sender<Response>, Cmd)>,
    sender: Sender<(Sender<Response>, Cmd)>,
}

#[derive(Debug)]
pub struct RepositoryAccess {
    sender: Sender<(Sender<Response>, Cmd)>,
}

impl RepositoryAccess {
    pub fn stop(&mut self) {
        let (s1, r1) = channel();
        match self.sender.send((s1, Cmd::Shutdown)) {
            Ok(_) => {
                match r1.recv() {
                    Ok(_) => debug!("Successfully shutdown repository access"),
                    Err(e) => warn!("Could not shutdown repostiroy access, received no ack"),
                };
            },
            Err(e) => {
                warn!("Could not shutdown repostiroy access, could not send stop");
            }
        }
    }

    pub fn get_sender(&mut self) -> Sender<(Sender<Response>, Cmd)> {
        self.sender.clone()
    }
}

impl RepositoryService {
    pub fn new() -> (Self, RepositoryAccess) {
        let (sender, receiver) = channel();
        let service = RepositoryService { sender: sender.clone(), receiver: receiver };
        let stopper = RepositoryAccess { sender: sender.clone() };
        (service, stopper)
    }

    pub fn work_loop(&self) {
        println!("Starting work loop");
        let mut shutdown = false;
        while !shutdown {
            let result = self.receiver.recv();
            if result.is_ok() {
                let (sender, cmd) = result.unwrap();
                println!("Received command {:?}", cmd);
                let id = Uuid::new_v4();
                match cmd {
                    Cmd::CreateRepository(name) => {
                        let id = Uuid::new_v4();
                        sender.send(Response::CreatedRepository(id, name.clone())).unwrap()
                    }
                    Cmd::Shutdown => {
                        shutdown = true;
                        sender.send(Response::ShutdownSuccessful).unwrap()
                    }
                    _ => sender.send(Response::RepositoryClosed).unwrap(),
                };
            } else {
                return;
            }
        }
    }
}