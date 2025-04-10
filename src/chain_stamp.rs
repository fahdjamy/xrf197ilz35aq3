use crate::core::generate_timebase_str_id;
use crate::{CHAIN_STAMP_LEFT_CHAIN, CHAIN_STAMP_PARENT_SEPARATOR, CHAIN_STAMP_RIGHT_CHAIN};
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Debug, Clone)]
enum ChainStampVersion {
    V1,
}

impl Display for ChainStampVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainStampVersion::V1 => {
                write!(f, "v1")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ChainStamp(pub String);

impl ChainStamp {
    pub fn build(root_cs: Option<ChainStamp>) -> ChainStamp {
        if root_cs.is_none() {
            return ChainStamp(format!(
                "{}*{}{}",
                ChainStampVersion::V1,
                generate_timebase_str_id(),
                CHAIN_STAMP_RIGHT_CHAIN
            ));
        }
        ChainStamp(format!(
            "{}*{}{}{}",
            ChainStampVersion::V1,
            CHAIN_STAMP_LEFT_CHAIN,
            generate_timebase_str_id(),
            CHAIN_STAMP_RIGHT_CHAIN
        ))
    }

    pub fn is_root(&self) -> bool {
        if !self.0.contains(CHAIN_STAMP_LEFT_CHAIN) {
            return true;
        }
        false
    }

    fn compare_with_root(&self, rhs: &ChainStamp) -> bool {
        if self.0.is_empty() && rhs.0.is_empty() {
            return true;
        } else if self.0.is_empty() || rhs.0.is_empty() {
            return false;
        }

        let lhs_parts = self
            .0
            .split(CHAIN_STAMP_PARENT_SEPARATOR)
            .collect::<Vec<&str>>();

        let rhs_parts = rhs
            .0
            .split(CHAIN_STAMP_PARENT_SEPARATOR)
            .collect::<Vec<&str>>();

        if lhs_parts.len() != rhs_parts.len() {
            return false;
        }

        let rhs_root_id = lhs_parts[0];
        let lhs_root_id = rhs_parts[0];

        rhs_root_id.len() == lhs_root_id.len()
            && rhs_root_id == lhs_root_id
            && lhs_parts[1..].len() == 1 // make sure remaining parts is one string is lhs
            && rhs_parts[1..].len() == 1 // make sure remaining parts is one string is rhs
            && Self::compare_last_root_parts(lhs_parts[1], rhs_parts[1])
    }

    fn compare_last_root_parts(lhs: &str, rhs: &str) -> bool {
        if lhs.is_empty() || rhs.is_empty() {
            return false;
        }
        lhs.len() == rhs.len() && lhs == rhs
    }

    pub fn inner(&self) -> String {
        self.0.to_string()
    }
}

impl PartialEq for ChainStamp {
    fn eq(&self, other: &ChainStamp) -> bool {
        if self.is_root() && other.is_root() {
            return self.compare_root(other);
        } else if self.is_root() {
            return false;
        } else if other.is_root() {
            return false;
        }

        ChainStamp::compare_with_root(self, other)
    }
}

impl Display for ChainStamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}
