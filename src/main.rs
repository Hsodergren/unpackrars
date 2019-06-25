mod output;
mod rarfiles;
use crate::rarfiles::RarFiles;
use clap::arg_enum;
use log::*;
use output::RealOutput;
use output::{handle_output, FancyHandler, LogHandler, Output, StdoutHandler};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

arg_enum! {
    #[derive(Debug, Clone)]
    enum OutputType {
        Stdout,
        Log,
        Fancy,
    }
}

impl OutputType {
    fn into_output(&self) -> Box<output::HandleOutput + Send> {
        match self {
            OutputType::Stdout => Box::new(StdoutHandler::new()),
            OutputType::Log => Box::new(LogHandler::new()),
            OutputType::Fancy => Box::new(FancyHandler::new()),
        }
    }
}

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "p", long = "path")]
    path: String,

    #[structopt(short = "r", long = "remove")]
    remove: bool,

    #[structopt(
        short = "o",
        long = "output",
        raw(possible_values = "&OutputType::variants()"),
        raw(case_insensitive = "true")
    )]
    output: Option<OutputType>,
}

impl Opt {
    fn get_output(&mut self) -> output::Data {
        output::Data {
            output: self
                .output
                .take()
                .or(Some(OutputType::Stdout))
                .unwrap()
                .into_output(),
        }
    }
}

fn main() {
    let mut opt = Opt::from_args();
    let (sender, handle) = handle_output(opt.get_output());

    let walker = WalkDir::new(opt.path)
        .into_iter()
        // only return unhidden directories
        .filter_entry(|e| !is_hidden(e) && is_dir(e))
        // get the rarfiles, if any
        .filter_map(|entry| match entry {
            Ok(e) => Some((
                e.path().to_path_buf(),
                RarFiles::new(e.path().to_path_buf(), sender.clone()),
            )),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        });
    for (id, (path, rar_files)) in walker.enumerate() {
        std::thread::sleep_ms(100);
        let _ = sender.send(Output::Visit(path.clone()).into());
        if let Some(main) = rar_files.get_main_rar_opt() {
            match rar_files.unrar(id) {
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
    sender.send(RealOutput::Exit);
    handle.join();
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
