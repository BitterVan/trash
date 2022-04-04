use std::env;
use std::fs;

mod commander;
use commander::*;

extern crate pest;
#[macro_use]
extern crate pest_derive;
use pest::Parser;
#[derive(Parser)]
#[grammar = "trash.pest"]
pub struct TrashParser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut commander = if args.len() > 1 {
        BundleCommander::from_file(args[1].to_owned())
    } else {
        BundleCommander::from_std()
    };
    println!("{:?}", commander.get_cmd());

    let file_string = fs::read_to_string("/home/bittervan/Repos/trash/test/test.sh").unwrap();
    let file_tree = TrashParser::parse(Rule::script, &file_string);
    print!("{}", file_string);
    print!("{:#?}", file_tree);
}