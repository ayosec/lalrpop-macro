#[macro_use]
extern crate lalrpop_macro;

// lalrpop_mod!(TestParser, "test.lalrpop");

#[derive(LarlpopGenerator)]
#[allow(dead_code)]
#[source = "src/test.lalrpop"]
struct Test;

fn main() {
    let r = TestParser::parse_Expr("1 + 2");
    println!("{:?}", r);
}
