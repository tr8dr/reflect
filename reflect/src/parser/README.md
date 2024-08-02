# Description
This module contains the CTorParser.  The parser takes expressions from configuration or another source, such as:
- Sample(Momentum(SMA, 0.9, [100,50], [0.3, 0.7]), 300)

and generates a type instance of `Sample` where the first argument to its ctor is an `Arc<Box<dyn Trait>>` to an instance \
of the `Momentum` type, implementing the expected Trait.

# Status
This is a work in progress.  At this point have only created the grammar, and need to complete the parser.

