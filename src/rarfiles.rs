use crate::output::Output;
use lazy_static::lazy_static;
use log::*;
use regex::Regex;
use std::fs::{read_dir, remove_file};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

lazy_static! {
    static ref RAR_REGEXP: Regex = { Regex::new(r"r\d\d").unwrap() };
    static ref PERCENT: Regex = { Regex::new(r"\d{1,2}%").unwrap() };
}

#[derive(Debug)]
pub struct RarFiles {
    main_rar: Option<PathBuf>,
    other_rars: Vec<PathBuf>,
    sender: Sender<Output>,
}

impl RarFiles {
    pub fn new(base: PathBuf, sender: Sender<Output>) -> RarFiles {
        let mut main_rar = None;
        let mut other_rars = vec![];
        let entries = read_dir(base).expect("Unable to read dir");
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "rar" {
                                if main_rar == None {
                                    main_rar = Some(entry.path());
                                } else {
                                    warn!("two rar files in same directory");
                                }
                            } else if RAR_REGEXP.is_match(ext.to_str().unwrap()) {
                                other_rars.push(entry.path())
                            }
                        }
                    }
                }
            }
        }
        RarFiles {
            main_rar,
            other_rars,
            sender,
        }
    }

    fn get_main_rar(&self) -> PathBuf {
        self.main_rar.clone().unwrap()
    }

    pub fn get_main_rar_opt(&self) -> Option<PathBuf> {
        self.main_rar.clone()
    }

    pub fn unrar(&self) -> Result<(), std::io::Error> {
        self.sender.send(Output::New(self.get_main_rar()));
        let mut process = Command::new("unrar")
            .arg("x")
            .arg(self.get_main_rar())
            .arg("-y") // unpack even if the result is already existing
            .current_dir(self.get_main_rar().parent().unwrap())
            .stdout(Stdio::piped())
            .spawn()?;

        let reader = BufReader::new(process.stdout.take().unwrap());
        for line in reader.lines() {
            let line = line.unwrap();
            if let Some(m) = PERCENT.find(line.as_str()) {
                self.sender.send(Output::Progress(
                    line[m.start()..m.end() - 1].parse().unwrap(),
                ));
            }
        }

        let status = process.wait()?;

        if status.success() {
            self.sender.send(Output::Done(self.get_main_rar()));
            Ok(())
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::Other))
        }
    }

    pub fn remove_rars(self) -> Result<(), std::io::Error> {
        if let Some(f) = self.main_rar {
            trace!("removing {:?}", f);
            remove_file(f)?;
        }
        for f in self.other_rars {
            trace!("removing {:?}", f);
            remove_file(f)?;
        }
        Ok(())
    }
}
