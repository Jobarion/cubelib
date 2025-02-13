use std::fmt::{Display, Formatter};
use std::str::FromStr;
use itertools::Itertools;
//This should be in the htr step, but we need it in the wasm version and the HTR step cannot be compiled to wasm right now

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Subset {
    pub discriminator: Option<&'static str>,
    pub generator: &'static str, //We need a const array, and creating an FromStr isn't const
    pub corners: u8,
    pub edges: u8,
    pub qt_corners: u8,
    pub qt: u8,
}

impl FromStr for Subset {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DR_SUBSETS.iter()
            .find(|x|x.to_string().as_str().eq(s))
            .cloned()
            .ok_or(())
    }
}

impl Subset {
    pub(crate) const fn new(name: Option<&'static str>, generator: &'static str, corners: u8, edges: u8, qt_corners: u8, qt: u8) -> Self {
        Self { discriminator: name, generator, corners, edges, qt_corners, qt }
    }
}

impl Display for Subset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{} {}e", self.corners, self.discriminator.unwrap_or("c"), self.qt_corners, self.edges)
    }
}

pub const DR_SUBSETS: [Subset; 48] = [
    Subset::new(None, "", 0, 0, 0, 0),
    Subset::new(None, "U R2 F2 R2 U", 0, 2, 0, 2),
    Subset::new(None, "U R2 L2 D", 0, 4, 0, 2),
    Subset::new(None, "U R2 L2 F2 R2 F2 D", 0, 6, 0, 2),
    Subset::new(None, "U R2 L2 F2 B2 U", 0, 8, 0, 2),
    Subset::new(Some("a"), "U R2 L2 U F2 B2 D", 4, 0, 1, 3),
    Subset::new(Some("a"), "U R2 L2 U R2 U", 4, 2, 1, 3),
    Subset::new(Some("a"), "U", 4, 4, 1, 1),
    Subset::new(Some("b"), "U R2 F2 R2 F2 U", 4, 0, 2, 2),
    Subset::new(Some("b"), "U R2 U", 4, 2, 2, 2),
    Subset::new(Some("a"), "U R2 F2 U", 4, 4, 2, 2),
    Subset::new(Some("a"), "U R2 U R2 B2 R2 U' R2 U", 4, 0, 2, 2),
    Subset::new(Some("a"), "D B2 D' F2 B2 D' F2 D", 4, 2, 2, 2),
    Subset::new(Some("a"), "U R2 U2 F2 U", 4, 4, 2, 2),
    Subset::new(None, "U F2 U2 R2 B2 U' L2 B2 D", 2, 0, 3, 3),
    Subset::new(None, "U R2 U R2 U", 2, 2, 3, 3),
    Subset::new(None, "U L2 U F2 U", 2, 4, 3, 3),
    Subset::new(None, "U L2 D R2 F2 B2 U", 2, 6, 3, 3),
    Subset::new(None, "U B2 L2 U B2 L2 U2 B2 D", 2, 8, 3, 3),
    Subset::new(None, "U R2 F2 U R2 U2 F2 U", 4, 0, 3, 3),
    Subset::new(None, "U B2 U R2 U2 F2 D", 4, 2, 3, 3),
    Subset::new(None, "U B2 U' L2 U2 B2 D", 4, 4, 3, 3),
    Subset::new(None, "U R2 U2 F2 U' R2 U2 R2 F2 U", 0, 0, 3, 3),
    Subset::new(None, "U R2 U2 F2 U R2 U2 F2 U", 0, 2, 3, 3),
    Subset::new(None, "U R2 U2 F2 U R2 U2 R2 B2 D", 0, 4, 3, 3),
    Subset::new(None, "U L2 U2 F2 U B2 U2 R2 U", 0, 6, 3, 3),
    Subset::new(None, "U R2 U2 F2 U' L2 U2 R2 F2 D", 0, 8, 3, 3),
    Subset::new(None, "U L2 D' R2 D L2 U", 2, 0, 4, 4),
    Subset::new(None, "U R2 U' R2 U R2 U", 2, 2, 4, 4),
    Subset::new(None, "U R2 U' L2 D R2 D", 2, 4, 4, 4),
    Subset::new(None, "U' B2 D' L2 B2 U' R2 U", 2, 6, 4, 4),
    Subset::new(None, "U R2 L2 B2 U R2 U' F2 U", 2, 8, 4, 4),
    Subset::new(None, "U R2 U R2 U2 B2 U B2 U", 0, 0, 4, 4),
    Subset::new(None, "U L2 U B2 U2 R2 U L2 U", 0, 2, 4, 4),
    Subset::new(None, "U B2 U F2 U2 R2 U R2 D", 0, 4, 4, 4),
    Subset::new(None, "U' R2 U L2 U2 L2 B2 U' B2 D", 0, 6, 4, 4),
    Subset::new(None, "U' R2 U R2 U2 B2 U F2 R2 L2 U", 0, 8, 4, 4),
    Subset::new(None, "U R2 U B2 U2 R2 F2 B2 U' B2 U", 4, 0, 4, 4),
    Subset::new(None, "U' F2 U F2 U2 R2 U' R2 U", 4, 2, 4, 4),
    Subset::new(None, "U' B2 U F2 U2 R2 U' R2 U", 4, 4, 4, 4),
    Subset::new(None, "U L2 U L2 U' L2 B2 U' B2 U", 2, 0, 5, 5),
    Subset::new(None, "U L2 U R2 U' R2 U R2 U", 2, 2, 5, 5),
    Subset::new(None, "D R2 U L2 D' R2 D L2 U", 2, 4, 5, 5),
    Subset::new(None, "U L2 U F2 U' R2 U B2 U", 2, 6, 5, 5),
    Subset::new(None, "U R2 U L2 F2 U B2 U' L2 U", 2, 8, 5, 5),
    Subset::new(None, "U' L2 U L2 U' R2 U R2 U", 4, 0, 5, 5),
    Subset::new(None, "U' R2 U F2 U' F2 U B2 D", 4, 2, 5, 5),
    Subset::new(None, "U' R2 U F2 U' F2 U F2 U", 4, 4, 5, 5)
];



pub fn expand_subset_name(name: &str) -> Vec<(Subset, u8)> {
    DR_SUBSETS.iter().cloned().zip(0u8..)
        .filter(|(x, _)|{
            if name.len() >= 2 {
                x.to_string().starts_with(name)
            } else if name.len() == 1 {
                if let Ok(qt) = u8::from_str(name) {
                    x.qt_corners == qt
                } else {
                    false
                }
            } else {
                false
            }
        })
        .collect_vec()
}