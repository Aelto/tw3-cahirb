use crate::parser::prelude::*;

/// Engine timestamp, stored as two `u32` values.
pub struct CDateTime {
    /// Bits: year `31..20`, month `19..15`, day `14..10`
    pub date: u32,
    /// Bits: hour `31..22`, minute `21..16`, second `15..10`, millisecond `9..0`
    pub time: u32,
}

impl CDateTime {
    pub fn now() -> Self {
        let now = time::OffsetDateTime::now_utc();
        let year = now.year() as u32;
        let month = u32::from(u8::from(now.month()));
        let day = u32::from(now.day());
        let hour = u32::from(now.hour());
        let minute = u32::from(now.minute());
        let second = u32::from(now.second());
        let millisecond = u32::from(now.millisecond());

        Self {
            date: year << 20 | month << 15 | day << 10,
            time: hour << 22 | minute << 16 | second << 10 | millisecond,
        }
    }
}

impl WithParsing for CDateTime {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, date) = parse_u32(i)?;
        let (i, time) = parse_u32(i)?;

        Ok((i, Self { date, time }))
    }
}

impl WithWriting for CDateTime {
    fn write(&self, w: &mut BlobWriter) {
        w.write_u32(self.date);
        w.write_u32(self.time);
    }
}
