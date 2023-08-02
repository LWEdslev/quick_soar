pub mod parser;
pub mod analysis;
pub mod web_handling;
pub mod excel;

pub enum PathStrategy {
    Linux,
    Windows,
    MacOS,
}

impl PathStrategy {
    pub fn new() -> Self {
        if cfg!(target_os = "windows") {
            PathStrategy::Windows
        } else if cfg!(target_os = "macos") {
            PathStrategy::MacOS
        } else if cfg!(target_os = "linux") {
            PathStrategy::Linux
        } else {
            panic!("unrecognized OS")
        }
    }

    pub fn get_path(&self) -> String {
        match self {
            PathStrategy::Linux => {
                match home::home_dir() {
                    Some(path) => path.to_str().expect("unreachable").to_string() + &*"/.quicksoar/",
                    None => panic!("no home directory found"),
                }
            }
            PathStrategy::Windows => {
                match home::home_dir() {
                    Some(path) => path.to_str().expect("unreachable").to_string() + &*"/.quicksoar/",
                    None => panic!("no home directory found"),
                }
            }
            PathStrategy::MacOS => {
                match home::home_dir() {
                    Some(path) => path.to_str().expect("unreachable").to_string() + &*"/.quicksoar/",
                    None => panic!("no home directory found"),
                }
            }
        }
    }
}