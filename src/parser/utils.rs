//! Various small utility functions for parsing

use super::error::ParseResult;

/// Maps a function over `Option<ParseResult<A>>` to produce `Option<ParseResult<B>>`
pub fn fmap<A, B, F>(option_err : Option<ParseResult<A>>, f : F) -> Option<ParseResult<B>> 
where F : FnOnce(A) -> B
{
    option_err.map(|r| r.map(|v| f(v)))
}