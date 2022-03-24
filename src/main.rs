use pest::Parser;

#[derive(Parser)]
#[grammar = "trash.pest"]
pub struct TrashParser;

fn main() {
    let res = TrashParser::parse(Rule::expr, "6^5");
    println!("{:?}", res);
}