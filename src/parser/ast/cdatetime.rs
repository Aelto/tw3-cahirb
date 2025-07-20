use crate::parser::prelude::*;

pub struct CDateTime {
    pub date: u32,
    pub time: u32,
}

impl WithParsing for CDateTime {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, date) = parse_u32(i)?;
        let (i, time) = parse_u32(i)?;

        Ok((i, Self { date, time }))
    }
}
