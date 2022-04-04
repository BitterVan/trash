use std::fs;
use crate::{ TrashParser, Rule };
use crate::pest::Parser;

pub trait Commander {
    fn get_cmd(&mut self) -> Option<String>;
}

pub struct StdCommander {

}

impl StdCommander {
    pub fn new() -> Self {
        StdCommander{

        }
    }
}

impl Commander for StdCommander {
    fn get_cmd(&mut self) -> Option<String> {
        Some(String::from("test"))
    }
}

pub struct FileCommander {
    file_string: String,
}

impl FileCommander {
    pub fn new(file_name: String) -> Self {
        let file_string = fs::read_to_string(file_name).expect("Error: {}, No such file or directory");
        FileCommander{
            file_string: file_string,
        }
    }
}

impl Commander for FileCommander {
    fn get_cmd(&mut self) -> Option<String> {
        let res = TrashParser::parse(Rule::expr, "cd /home/bittervan + 2");
        println!("{:#?}", res);
        Some(String::from("test"))
    }
}

enum CommanderType {
    Std(StdCommander),
    File(FileCommander),    
}

pub struct BundleCommander {
    commander_type: CommanderType
}

impl BundleCommander {
    pub fn from_file(file_name: String) -> Self {
        BundleCommander {
            commander_type: CommanderType::File(FileCommander::new(file_name)),
        }
    }

    pub fn from_std() -> Self {
        BundleCommander {
            commander_type: CommanderType::Std(StdCommander::new()),
        }
    }
}

impl Commander for BundleCommander {
    fn get_cmd(&mut self) -> Option<String> {
        match self.commander_type {
            CommanderType::Std(ref mut commander) => commander.get_cmd(),
            CommanderType::File(ref mut commander) => commander.get_cmd(),
        }
    } 
}