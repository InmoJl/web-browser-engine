use crate::parser::{parse, Parser};

mod parser;
mod dom;
mod css;
mod style;
mod layout;

fn main() {
    let mut node = parse(String::from("<html lang='zh'><body>Hello, world!</body></html>"));
    println!("{:#?}", node);
}
