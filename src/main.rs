mod output;
mod rarfiles;
use crate::rarfiles::RarFiles;
use log::*;
use output::{handle_output, LogHandler, Output, StdoutHandler};
use std::sync::mpsc;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "p", long = "path")]
    path: String,

    #[structopt(short = "r", long = "remove")]
    remove: bool,
}

fn main() {
    env_logger::init();
    let (sender, receiver) = mpsc::channel();
    handle_output(receiver, StdoutHandler());

    let opt = Opt::from_args();
    let walker = WalkDir::new(opt.path)
        .into_iter()
        // only return unhidden directories
        .filter_entry(|e| !is_hidden(e) && is_dir(e))
        // get the rarfiles, if any
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                Some((
                    e.path().to_path_buf(),
                    RarFiles::new(e.path().to_path_buf(), sender.clone()),
                ))
            })
        });
    for (path, rar_files) in walker {
        let _ = sender.send(Output::Visit(path.clone()));
        if let Some(main) = rar_files.get_main_rar_opt() {
            match rar_files.unrar() {
                Ok(()) => {
                    if opt.remove {
                        match rar_files.remove_rars() {
                            Ok(()) => {}
                            Err(e) => error!("while removing {:?}: {}", main, e),
                        };
                    }
                }
                Err(e) => error!("while unraring {:?}: {}", main, e),
            };
        }
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
}
