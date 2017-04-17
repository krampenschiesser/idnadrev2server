use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{RwLock, Arc};
use uuid::Uuid;
use std::thread;
use repository::{Repository, RepositoryFile};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Cmd {
    CreateRepository(String),
    GetFile(Uuid, Uuid),
    UpdateFileHeader(Uuid, Uuid, u32, RepositoryFile),
    UpdateFileContent(Uuid, Uuid, Vec<u8>),
    UpdateFile(Uuid, Uuid, u32, RepositoryFile, Vec<u8>),
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
}

#[derive(Debug)]
pub struct RepositoryService {
    running: Arc<AtomicBool>,

    receiver: Receiver<(Sender<Response>, Cmd)>,
    sender: Sender<(Sender<Response>, Cmd)>,
}

#[derive(Debug)]
pub struct RepositoryAccess {
    running: Arc<AtomicBool>,
    sender: Sender<(Sender<Response>, Cmd)>,
}

impl RepositoryAccess {
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn get_sender(&mut self) -> Sender<(Sender<Response>, Cmd)> {
        self.sender.clone()
    }
}

impl RepositoryService {
    pub fn new() -> (Self, RepositoryAccess) {
        let (sender, receiver) = channel();
        let running = Arc::new(AtomicBool::new(true));
        let service = RepositoryService { sender: sender.clone(), receiver: receiver, running: running.clone() };
        let stopper = RepositoryAccess { running: running, sender: sender.clone() };
        (service, stopper)
    }

    pub fn work_loop(&self) {
        println!("Starting work loop");
        while self.running.load(Ordering::Relaxed) {
            let result = self.receiver.recv_timeout(Duration::from_secs(1));
            if result.is_ok() {
                let (sender, cmd) = result.unwrap();
                println!("Received command {:?}", cmd);
                let id = Uuid::new_v4();
                match cmd {
                    Cmd::CreateRepository(name) => {
                        let id = Uuid::new_v4();
                        sender.send(Response::CreatedRepository(id, name.clone())).unwrap()
                    }
                    _ => sender.send(Response::RepositoryClosed).unwrap(),
                };
            }
        }
    }
}