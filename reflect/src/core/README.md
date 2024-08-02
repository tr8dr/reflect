This module contains core functionality for type reflection.   I would suggest taking a look at code in the following order:

1. types.rs: a core interface to type representation and reflection functionality
2. parts.rs: the reflected representation of ctors, methods, static functions

The other modules: `conversions` and `registration` are very low level and not terribly instructive.  These are used by the
macros for type registration and conversions.
