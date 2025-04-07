use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum BlockRegion {
    USEastOhio,
    USWestOregon,
    MexicoCentral,
    USWestNVirginia,
}

impl Display for BlockRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockRegion::USEastOhio => {
                write!(f, "USEastOhio")
            }
            BlockRegion::USWestOregon => {
                write!(f, "USWestOregon")
            }
            BlockRegion::MexicoCentral => {
                write!(f, "MexicoCentral")
            }
            BlockRegion::USWestNVirginia => {
                write!(f, "USWestNVirginia")
            }
        }
    }
}

impl FromStr for BlockRegion {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USEastOhio" => Ok(BlockRegion::USEastOhio),
            "USWestOregon" => Ok(BlockRegion::USEastOhio),
            "MexicoCentral" => Ok(BlockRegion::MexicoCentral),
            "USWestNVirginia" => Ok(BlockRegion::USWestNVirginia),
            _ => Err(DomainError::ParseError("Unknown block region".to_string())),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: u64,
    pub version: u64,
    pub region: BlockRegion,
    pub creation_date: DateTime<Utc>,
}

impl Block {
    pub fn new(id: u64, version: u64, region: BlockRegion) -> Self {
        Block {
            id,
            region,
            version,
            creation_date: Utc::now(),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block :: id={}, region={}", self.id, self.version)
    }
}
