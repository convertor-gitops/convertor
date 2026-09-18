use crate::error::ParseError;
pub(crate) mod clash_parser;
pub(crate) mod surge_parser;
pub trait Parse<F>: Sized {
    fn parse(content: &str) -> Result<Self, ParseError>;
}
