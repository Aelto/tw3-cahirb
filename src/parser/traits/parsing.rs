use crate::parser::ast;

pub trait WithParsing
where
    Self: Sized,
{
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self>;

    fn parse_array(i: &[u8]) -> nom::IResult<&[u8], Vec<Self>> {
        let (mut i, size) = ast::parse_compressed_i32(i)?;
        let mut elements = Vec::new();
        if size > 0 {
            elements.reserve_exact(size as usize);

            for _ in 0..size {
                let (new_i, element) = Self::parse(i)?;

                i = new_i;
                elements.push(element);
            }
        }

        Ok((i, elements))
    }

    fn parse_array_u32(i: &[u8]) -> nom::IResult<&[u8], Vec<Self>> {
        let (mut i, size) = ast::parse_compressed_u32(i)?;
        let mut elements = Vec::new();
        if size > 0 {
            elements.reserve_exact(size as usize);

            for _ in 0..size {
                let (new_i, element) = Self::parse(i)?;

                i = new_i;
                elements.push(element);
            }
        }

        Ok((i, elements))
    }
}
