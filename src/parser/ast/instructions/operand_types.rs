use crate::parser::prelude::*;

pub trait Operand: Sized {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self>;
    fn write_operand(&self, w: &mut BlobWriter);
    fn runtime_size(&self) -> usize;
    fn to_operand_value(&self) -> OperandValue;
}

#[derive(Debug, Clone)]
pub struct VariableRef(pub CName);

#[derive(Debug, Clone)]
pub struct PropertyRef {
    pub name: CName,
    pub scope: ImportTypeRef,
}

#[derive(Debug, Clone)]
pub enum FunctionRef {
    Import(ImportFunctionRef),
    CurrentFunction,
    InternalOperator(InternalOperatorRef),
}

const POINTER_SIZE: usize = 8;

impl Operand for u8 {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        parse_u8(i)
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write_u8(*self);
    }

    fn runtime_size(&self) -> usize {
        1
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::Unsigned8(*self)
    }
}

impl Operand for bool {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        parse_u8(i).map(|(i, value)| (i, value > 0))
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write_u8(*self as u8);
    }

    fn runtime_size(&self) -> usize {
        1
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::Bool(*self)
    }
}

impl Operand for u16 {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        parse_u16(i)
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write_u16(*self);
    }

    fn runtime_size(&self) -> usize {
        2
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::Unsigned16(*self)
    }
}

impl Operand for i16 {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        parse_i16(i)
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write_i16(*self);
    }

    fn runtime_size(&self) -> usize {
        2
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::Integer16(*self)
    }
}

impl Operand for u32 {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        parse_compressed_u32(i)
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write_compressed_u32(*self);
    }

    fn runtime_size(&self) -> usize {
        4
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::Unsigned32(*self)
    }
}

impl Operand for i32 {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        parse_compressed_i32(i)
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write_compressed_i32(*self);
    }

    fn runtime_size(&self) -> usize {
        4
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::Integer32(*self)
    }
}

impl Operand for f32 {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        parse_f32(i)
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write_f32(*self);
    }

    fn runtime_size(&self) -> usize {
        4
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::Float(*self)
    }
}

impl Operand for String {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, len) = parse_compressed_u32(i)?;
        let (i, bytes) = take(len)(i)?;

        Ok((i, String::from_utf8_lossy(bytes).into_owned()))
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write_compressed_u32(self.len() as u32);
        w.write_bytes(self.as_bytes());
    }

    fn runtime_size(&self) -> usize {
        4 + self.len()
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::String(self.clone())
    }
}

impl Operand for CName {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        <CName as WithParsing>::parse(i)
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write(self);
    }

    fn runtime_size(&self) -> usize {
        4
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::Name(self.clone())
    }
}

impl Operand for VariableRef {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        <CName as WithParsing>::parse(i).map(|(i, name)| (i, Self(name)))
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write(&self.0);
    }

    fn runtime_size(&self) -> usize {
        POINTER_SIZE
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::Name(self.0.clone())
    }
}

impl Operand for PropertyRef {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, name) = <CName as WithParsing>::parse(i)?;
        let (i, scope) = <ImportTypeRef as WithParsing>::parse(i)?;

        Ok((i, Self { name, scope }))
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write(&self.name);
        w.write(&self.scope);
    }

    fn runtime_size(&self) -> usize {
        POINTER_SIZE
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::ClassProp {
            name: self.name.clone(),
            type_ref: self.scope.clone(),
        }
    }
}

impl Operand for ImportTypeRef {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        <ImportTypeRef as WithParsing>::parse(i)
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        w.write(self);
    }

    fn runtime_size(&self) -> usize {
        POINTER_SIZE
    }

    fn to_operand_value(&self) -> OperandValue {
        OperandValue::ImportType(self.clone())
    }
}

impl Operand for FunctionRef {
    fn parse_operand(i: &[u8]) -> IResult<&[u8], Self> {
        let (rest, index) = parse_compressed_i32(i)?;

        let function = match index {
            1.. => Self::Import(ImportFunctionRef {
                table_index: (index - 1) as u32,
            }),
            -1 => Self::CurrentFunction,
            ..-1 => Self::InternalOperator(InternalOperatorRef {
                table_index: -(index + 2) as usize,
            }),
            0 => {
                return Err(nom::Err::Failure(nom::error::Error::new(
                    i,
                    nom::error::ErrorKind::Verify,
                )));
            }
        };

        Ok((rest, function))
    }

    fn write_operand(&self, w: &mut BlobWriter) {
        let index = match self {
            Self::Import(import) => import.table_index as i32 + 1,
            Self::CurrentFunction => -1,
            Self::InternalOperator(operator) => -(operator.table_index as i32) - 2,
        };

        w.write_compressed_i32(index);
    }

    fn runtime_size(&self) -> usize {
        POINTER_SIZE
    }

    fn to_operand_value(&self) -> OperandValue {
        match self {
            Self::Import(import) => OperandValue::ImportFunction(import.clone()),
            Self::CurrentFunction => OperandValue::ImportFunctionSelf,
            Self::InternalOperator(operator) => OperandValue::ImportFunctionInternal(operator.clone()),
        }
    }
}
