use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::string::ToString;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub enum StepKind {
    EO,
    RZP,
    AR,
    DR,
    HTR,
    FR,
    FRLS,
    FIN,
    FINLS,
    Other(String)
}

impl Display for StepKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<String>::into(self.clone()))
    }
}

impl FromStr for StepKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eo" => Ok(Self::EO),
            "dr" => Ok(Self::DR),
            "rzp" => Ok(Self::RZP),
            "ar" | "jzp" => Ok(Self::AR),
            "htr" => Ok(Self::HTR),
            "fr" => Ok(Self::FR),
            "frls" => Ok(Self::FRLS),
            "finish" | "fin" => Ok(Self::FIN),
            "finls" => Ok(Self::FINLS),
            x=> Ok(Self::Other(x.to_string()))
        }
    }
}

impl Into<String> for StepKind {
    fn into(self) -> String {
        match self {
            StepKind::EO => "eo".to_string(),
            StepKind::RZP => "rzp".to_string(),
            StepKind::AR => "arm".to_string(),
            StepKind::DR => "dr".to_string(),
            StepKind::HTR => "htr".to_string(),
            StepKind::FR => "fr".to_string(),
            StepKind::FRLS => "frls".to_string(),
            StepKind::FIN => "finish".to_string(),
            StepKind::FINLS => "finls".to_string(),
            StepKind::Other(x) => x,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde_support", derive(serde_with::DeserializeFromStr, serde_with::SerializeDisplay))]
pub enum NissSwitchType {
    #[default] Never = 0,
    Before = 1,
    Always = 2,
}

impl FromStr for NissSwitchType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(NissSwitchType::Never),
            "before" => Ok(NissSwitchType::Before),
            "always" => Ok(NissSwitchType::Always),
            _ => Err("Invalid option")
        }
    }
}

impl Display for NissSwitchType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NissSwitchType::Never => write!(f, "never"),
            NissSwitchType::Before => write!(f, "before"),
            NissSwitchType::Always => write!(f, "always"),
        }
    }
}
