use log::*;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::thread;

pub enum Output {
    Visit(PathBuf),
    New(PathBuf),
    Done(PathBuf),
    Progress(u8),
}

pub trait HandleOutput {
    fn handle(&self, o: Output);
}

pub fn handle_output<T>(receiver: Receiver<Output>, handler: T)
where
    T: HandleOutput + Send + Sync + 'static,
{
    thread::spawn(move || {
        for output in receiver {
            handler.handle(output);
        }
    });
}

pub struct LogHandler();

impl HandleOutput for LogHandler {
    fn handle(&self, o: Output) {
        use self::Output::{Done, New, Progress, Visit};
        match o {
            Visit(a) => trace!("visiting {:?}", a),
            New(a) => info!("unraring {:?}", a),
            Done(a) => info!("done with {:?}", a),
            Progress(p) => info!("progress: {}%", p),
        }
    }
}

pub struct StdoutHandler();

impl HandleOutput for StdoutHandler {
    fn handle(&self, o: Output) {
        use self::Output::{Done, New, Progress, Visit};
        match o {
            Visit(a) => println!("visiting {:?}", a),
            New(a) => println!("unraring {:?}", a),
            Done(a) => println!("done with {:?}", a),
            Progress(p) => println!("progress: {}%", p),
        }
    }
}
