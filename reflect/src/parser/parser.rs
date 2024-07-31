use pest::Parser;
use pest_derive::Parser;
use std::fs;


// Define the parser struct using the grammar file
#[derive(Parser)]
#[grammar = "src/parser/grammar.pest"] // Path to the grammar file
struct CTorParser;



#[cfg(test)]
mod tests {
    // Import the parent module's items for testing
    use super::*;

    #[test]
    fn test_parse1() {
        let input = "Filter(Momentum(SMA,[100,200,300],[0.6, 0.3, 0.1], 3), 900)";
        match MyParser::parse(Rule::expression, input) {
            Ok(pairs) => {
                for pair in pairs {
                    println!("{:?}", pair.as_str()); // Process the parsed result
                }
            }
            Err(e) => eprintln!("Parsing error: {:?}", e),
        }
    }
}