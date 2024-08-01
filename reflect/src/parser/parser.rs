use pest::{Parser};
use pest_derive::Parser;


// Define the parser struct using the grammar file
#[derive(Parser)]
#[grammar = "src/parser/grammar.pest"] // Path to the grammar file
pub struct CTorParser;



#[cfg(test)]
mod tests {
    // Import the parent module's items for testing
    use super::*;

    fn print_expression(pair: pest::iterators::Pair<Rule>, indent: usize) {
        let indent_str = " ".repeat(indent * 2);

        match pair.as_rule() {
            Rule::ctor_expression => {
                println!("{}Constructor:", indent_str);
                for inner_pair in pair.into_inner() {
                    print_expression(inner_pair, indent + 1);
                }
            }
            Rule::identifier =>
                println!("{}Identifier: {}", indent_str, pair.as_str()),
            Rule::integer =>
                println!("{}Integer: {}", indent_str, pair.as_str()),
            Rule::float =>
                println!("{}Float: {}", indent_str, pair.as_str()),
            Rule::list => {
                println!("{}List:", indent_str);
                for inner_pair in pair.into_inner() {
                    print_expression(inner_pair, indent + 1);
                }
            }
            Rule::argument_list => {
                println!("{}Arguments:", indent_str);
                for inner_pair in pair.into_inner() {
                    print_expression(inner_pair, indent + 1);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn test_parse1() {
        let input = "Filter(Momentum(SMA,[100,200,300],[0.6, 0.3, 0.1], 3), 900)";
        match CTorParser::parse(Rule::expression, input) {
            Ok(pairs) => {
                for pair in pairs {
                    print_expression(pair, 0);
                }
            }
            Err(e) => eprintln!("Parsing error: {:?}", e),
        }
    }
}