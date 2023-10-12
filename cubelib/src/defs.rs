use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StepKind {
    EO = 0,
    RZP = 1,
    DR = 2,
    HTR = 3,
    FR = 4,
    FRLS = 5,
    FIN = 6
}

impl Display for StepKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<String>::into(self.clone()))
    }
}

impl FromStr for StepKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eo" => Ok(Self::EO),
            "dr" => Ok(Self::DR),
            "rzp" => Ok(Self::RZP),
            "htr" => Ok(Self::HTR),
            "fr" => Ok(Self::FR),
            "frls" => Ok(Self::FRLS),
            "finish" | "fin" => Ok(Self::FIN),
            x=> Err(format!("Unknown step '{x}'"))
        }
    }
}

impl Into<String> for StepKind {
    fn into(self) -> String {
        match self {
            Self::EO => "eo",
            Self::RZP => "rzp",
            Self::DR => "dr",
            Self::HTR => "htr",
            Self::FR => "fr",
            Self::FRLS => "frls",
            Self::FIN => "finish",
        }.to_string()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NissSwitchType {
    Never = 0,
    Before = 1,
    Always = 2,
}