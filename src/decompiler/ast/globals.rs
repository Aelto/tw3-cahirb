use crate::decompiler::prelude::*;

#[derive(Debug)]
pub enum Globals {
    TheInput,
    TheTimer,
    TheDebug,
    TheSound,
    TheHud,
    TheCamera,
    ThePlayer,
    TheGame,
    TheTelemetry,
}

impl WithDecompiling for Globals {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, instr) = i.expect_any(&[
            "GetGame",
            "GetPlayer",
            "GetCamera",
            "GetHud",
            "GetSound",
            "GetDebug",
            "GetTimer",
            "GetInput",
            "GetTelemetry",
        ])?;

        let out = match instr.mnemo {
            "GetGame" => Self::TheGame,
            "GetPlayer" => Self::ThePlayer,
            "GetCamera" => Self::TheCamera,
            "GetHud" => Self::TheHud,
            "GetSound" => Self::TheSound,
            "GetDebug" => Self::TheDebug,
            "GetTimer" => Self::TheTimer,
            "GetInput" => Self::TheInput,
            "GetTelemetry" => Self::TheTelemetry,
            _ => panic!("unrecognized Globals pattern"),
        };

        Ok((i, out))
    }
}
