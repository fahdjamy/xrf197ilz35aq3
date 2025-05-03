use crate::core::generate_timebase_str_id;
use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum BlockVersion {
    V1,
}

impl Display for BlockVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockVersion::V1 => {
                write!(f, "block**V1**")
            }
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum BlockRegion {
    USEastOhio,
    USWestOregon,
    MexicoCentral,
    USWestNVirginia,
}

impl Display for BlockRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

#[derive(Serialize, Debug, Clone, Eq)]
pub struct Block {
    pub id: String,
    pub app_id: String,
    pub sequence_num: u64,
    pub chain_id: String,
    pub region: BlockRegion,
    pub version: BlockVersion,
    pub entry_ids: Vec<String>,
    pub creation_date: DateTime<Utc>,
}

impl Block {
    pub fn build(
        app_id: String,
        region: BlockRegion,
        entry_ids: Vec<String>,
        chain_stamp_id: String,
    ) -> Result<Self, DomainError> {
        if entry_ids.is_empty() {
            return Err(DomainError::InvalidArgument(
                "entry_ids cannot be empty.".to_string(),
            ));
        }
        Ok(Block {
            region,
            app_id,
            entry_ids,
            sequence_num: 0,
            chain_id: chain_stamp_id,
            version: BlockVersion::V1,
            creation_date: Utc::now(),
            id: generate_timebase_str_id(),
        })
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block :: id={}, region={}", self.id, self.version)
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.app_id == other.app_id && self.version == other.version
    }
}
