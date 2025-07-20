mod rsblob;
pub use rsblob::RsBlob;

mod tables;
pub use tables::TableManager;
pub use tables::WithTableResolving;

pub mod ast;

mod traits;
pub use traits::*;

pub mod prelude {
    pub use super::*;

    pub use ast::*;

    pub use nom::IResult;
    pub use nom::branch::alt;
    pub use nom::bytes::complete::{is_a, is_not, tag, take, take_till1, take_until1, take_while};
    pub use nom::character::complete::{char, crlf};
    pub use nom::combinator::value;
    pub use nom::error::ParseError;
    pub use nom::multi::{many0, separated_list0, separated_list1};
    pub use nom::sequence::{delimited, pair, preceded, terminated};

    pub fn trim(i: &str) -> IResult<&str, &str> {
        take_while(|c| c == ' ' || c == '\n' || c == '\r')(i)
    }
}
