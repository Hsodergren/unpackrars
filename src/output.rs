use self::Output::{Done, New, Progress, Visit};
use log::*;
use ncurses;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Debug)]
pub enum Output {
    Visit(PathBuf),
    New { path: PathBuf, id: usize },
    Done { id: usize },
    Progress { id: usize, procent: u8 },
}

impl From<Output> for RealOutput {
    fn from(o: Output) -> RealOutput {
        RealOutput::Output(o)
    }
}

pub enum RealOutput {
    Exit,
    Output(Output),
}

pub trait HandleOutput {
    fn handle(&mut self, o: Output);
}

pub struct Data {
    pub output: Box<HandleOutput + Send>,
}

impl HandleOutput for Data {
    fn handle(&mut self, o: Output) {
        self.output.handle(o);
    }
}

pub fn handle_output(mut handler: Data) -> (Sender<RealOutput>, thread::JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        for output in receiver {
            match output {
                RealOutput::Exit => break,
                RealOutput::Output(o) => handler.handle(o),
            }
        }
    });
    (sender, handle)
}

pub struct LogHandler {
    working: HashMap<usize, PathBuf>,
}

impl LogHandler {
    pub(crate) fn new() -> LogHandler {
        env_logger::init();
        LogHandler {
            working: HashMap::new(),
        }
    }
}

impl HandleOutput for LogHandler {
    fn handle(&mut self, o: Output) {
        match o {
            Visit(a) => trace!("visiting {:?}", a),
            New { path, id } => {
                info!("unraring {:?}", path);
                self.working.insert(id, path);
            }
            Done { id } => {
                let path = self.working.remove(&id);
                info!("done with {:?}", path.unwrap());
            }
            Progress { id: _, procent } => info!("progress: {}%", procent),
        }
    }
}

pub struct StdoutHandler {
    working: HashMap<usize, PathBuf>,
}

impl StdoutHandler {
    pub fn new() -> StdoutHandler {
        StdoutHandler {
            working: HashMap::new(),
        }
    }
}

impl HandleOutput for StdoutHandler {
    fn handle(&mut self, o: Output) {
        match o {
            Visit(a) => println!("visiting {:?}", a),
            New { path, id } => {
                println!("unraring {:?}", path);
                self.working.insert(id, path);
            }
            Done { id } => {
                let path = self.working.remove(&id);
                println!("done with {:?}", path.unwrap());
            }
            Progress { id: _, procent } => println!("progress: {}%", procent),
        }
    }
}

pub struct FancyHandler {
    working: HashMap<usize, Info>,
}

struct Info {
    path: PathBuf,
}

impl FancyHandler {
    pub fn new() -> FancyHandler {
        ncurses::initscr();
        FancyHandler {
            working: HashMap::new(),
        }
    }
}

impl HandleOutput for FancyHandler {
    fn handle(&mut self, o: Output) {
        ncurses::clear();
        ncurses::mvaddstr(0, 0, &format!("{:?}", o));
        ncurses::refresh();
    }
}

impl Drop for FancyHandler {
    fn drop(&mut self) {
        ncurses::endwin();
        println!("Fancy handler dropped");
    }
}
