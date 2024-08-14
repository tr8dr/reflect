use pest::{Parser};
use pest_derive::Parser;
use std::any::Any;
use std::vec::Vec;


// Define the parser struct using the grammar file
#[derive(Parser)]
#[grammar = "src/parser/grammar.pest"] // Path to the grammar file
pub struct CTorParser;


/// Parser for ctor expressions
impl CTorParser {

    /// Create type based on ctor expression
    /// ```
    ///    // create object based on ctor expression
    ///    let obj = CTorParser::create ("Resample(Momentum(SMA,[200,560,10],0.9), 300)");
    /// ```
    ///
    /// # Parameters
    /// - `expr`: ctor expression
    ///
    /// # Returns
    /// - create object instance or None
    pub fn create (expr: &str) -> Result<Box<dyn Any>,String> {
        todo!()
    }

    // private implementation

    /// Parse ctor
    /// - parse each argument recursively
    /// - create object
    ///
    /// # Arguments
    /// - `tree`: AST at current level
    fn parse_ctor (tree: &pest::iterators::Pair<Rule>) -> Result<Box<dyn Any>,String> {
        let mut argv_opt: Option<Vec::<Box<dyn Any>>> = None;
        let mut ctor_opt: Option<&str> = None;

        for subtree in tree.clone().into_inner() {
            match subtree.as_rule() {
                Rule::identifier => {
                    ctor_opt = Some(subtree.as_str());
                }
                Rule::argument_list => {
                    argv_opt = Self::parse_arguments (subtree.clone().into_inner());
                }
                _ => ()
            }
        }

        match (ctor_opt, argv_opt) {
            (None, _) =>
                Err(format!("failed to parse ctor for: {}", tree.to_string())),
            (_, None) =>
                Err(format!("failed to parse arguments for: {}", tree.to_string())),
            (Some(ctor), Some(argv)) => {
                todo!()
            }
        }
    }


    /// Parse arguments
    /// - parse each argument recursively
    ///
    /// # Arguments
    /// - `tree`: AST at current level
    fn parse_arguments (tree: pest::iterators::Pairs<Rule>) -> Option<Vec<Box<dyn Any>>> {
        let mut argv = Vec::<Box<dyn Any>>::new();

        for subtree in tree {
            match subtree.as_rule() {
                Rule::ctor_expression => {
                    match Self::parse_ctor (&subtree) {
                        Ok(obj) => argv.push(obj),
                        Err(_) => return None
                    }
                }
                Rule::identifier => {
                    argv.push(Box::new(subtree.to_string()) as Box<dyn Any>);
                }
                Rule::integer => {
                    let s = subtree.as_str();
                    let v: i64 = str::parse::<i64>(s).unwrap();
                    argv.push (Box::new(v));
                }
                Rule::float => {
                    let s = subtree.as_str();
                    let v = str::parse::<f64>(s).unwrap();
                    argv.push (Box::new(v));
                }
                Rule::list => {
                    argv.push (Self::parse_list (&subtree.into_inner()));
                }
                _ => ()
            }
        }

        Some(argv)
    }


    /// Parse arguments
    /// - parse each argument recursively
    ///
    /// # Arguments
    /// - `tree`: AST at current level
    fn parse_list (tree: &pest::iterators::Pairs<Rule>) -> Box<dyn Any> {
        let mut fvec = Vec::<f64>::new();
        let mut ivec = Vec::<i32>::new();

        for subtree in tree.clone() {
            match subtree.as_rule() {
                Rule::integer => {
                    let s = subtree.as_str();
                    let v = str::parse::<i32>(s).unwrap();
                    ivec.push (v);
                    fvec.push (v as f64);
                }
                Rule::float => {
                    let s = subtree.as_str();
                    let v = str::parse::<f64>(s).unwrap();
                    fvec.push (v);
                    ivec.clear();
                }
                _ => ()
            }
        }

        if fvec.len() > ivec.len() {
            Box::new(fvec) as Box<dyn Any>
        } else{
            Box::new(ivec) as Box<dyn Any>
        }
    }

}

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