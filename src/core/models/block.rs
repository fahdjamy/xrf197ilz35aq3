use crate::core::generate_timebase_str_id;
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
    pub id: String,
    pub version: u64,
    pub app_id: String,
    pub chain_stamp: u64,
    pub region: BlockRegion,
    pub entry_ids: Vec<String>,
    pub child_stamp: Option<u64>,
    pub ancestor_chain_stamp: u64,
    pub creation_date: DateTime<Utc>,
}

impl Block {
    pub fn new(
        app_id: String,
        version: u64,
        region: BlockRegion,
        ancestor_stamp: u64,
        entry_ids: Vec<String>,
    ) -> Result<Self, DomainError> {
        if entry_ids.is_empty() {
            return Err(DomainError::InvalidArgument(
                "entry_ids cannot be empty.".to_string(),
            ));
        }
        Ok(Block {
            region,
            app_id,
            version,
            entry_ids,
            chain_stamp: 0,
            child_stamp: None,
            creation_date: Utc::now(),
            id: generate_timebase_str_id(),
            ancestor_chain_stamp: ancestor_stamp,
        })
    }

    pub fn is_last_child(&self) -> bool {
        self.child_stamp.is_none()
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block :: id={}, region={}", self.id, self.version)
    }
}
