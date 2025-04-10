use crate::core::generate_timebase_str_id;
use crate::{CHAIN_STAMP_LEFT_CHAIN, CHAIN_STAMP_RIGHT_CHAIN, CHAIN_STAMP_VERSION_SEPARATOR};
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Debug, Clone)]
enum ChainStampVersion {
    V1,
}

impl From<&str> for ChainStampVersion {
    fn from(value: &str) -> Self {
        match value {
            "v1" => ChainStampVersion::V1,
            _ => ChainStampVersion::V1,
        }
    }
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
/// **ChainStamp**: Sample chain stampId should look like => v1*9203923<39203823>390234082
///
/// i.e. ChainStampVersion*parent/root_stamp_id<current_chain_stamp_id>child_stamp_id
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

    pub fn stamp_id(&self) -> &str {
        // sample => v1*9203923<39203823>390234082
        let parts: Vec<&str> = self.0.split(CHAIN_STAMP_VERSION_SEPARATOR).collect(); // split btn version
        let parts: Vec<&str> = parts[1].split(CHAIN_STAMP_LEFT_CHAIN).collect(); // split btn parent/root
        if parts[1].contains(CHAIN_STAMP_RIGHT_CHAIN) {
            // split btn child
            parts[1].split(CHAIN_STAMP_RIGHT_CHAIN)[0]
        }
        // there's no child, remaining is the id
        parts[1]
    }

    pub fn parent_chain_id(&self) -> &str {
        let parts: Vec<&str> = self.0.split(CHAIN_STAMP_VERSION_SEPARATOR).collect(); // split btn version
        let parts: Vec<&str> = parts[1].split(CHAIN_STAMP_LEFT_CHAIN).collect(); // split btn parent/root

        parts[0]
    }

    pub fn is_parent_chain(&self, parent_chain_stamp: &ChainStamp) -> bool {
        if self.parent_chain_id() == parent_chain_stamp.stamp_id() {
            return true;
        }
        false
    }

    pub fn version(&self) -> ChainStampVersion {
        let version_str: &str = self.0.split(CHAIN_STAMP_VERSION_SEPARATOR)[0];
        ChainStampVersion::from(version_str)
    }

    pub fn is_root(&self) -> bool {
        if !self.0.contains(CHAIN_STAMP_LEFT_CHAIN) {
            return true;
        }
        false
    }

    pub fn append_child(&mut self, child_stamp: ChainStamp) -> Result<(), String> {
        if self.inner().contains(CHAIN_STAMP_RIGHT_CHAIN) {
            return Err("can't append a child to chain that already contains child".to_string());
        }
        if !child_stamp.parent_chain_id() {
            return Err("can't append child to chain whose not the parent".to_string());
        }
        let child_stamp_id = child_stamp.stamp_id().to_string();

        self.0.push_str(CHAIN_STAMP_RIGHT_CHAIN);
        self.0.push_str(&child_stamp_id);
        Ok(())
    }

    fn compare_with_root(&self, rhs: &ChainStamp) -> bool {
        if self.0.is_empty() && rhs.0.is_empty() {
            return true;
        } else if self.0.is_empty() || rhs.0.is_empty() {
            return false;
        }

        let chain_value = self.0.split(CHAIN_STAMP_VERSION_SEPARATOR)[0];

        let lhs_parts = chain_value
            .split(CHAIN_STAMP_VERSION_SEPARATOR)
            .collect::<Vec<&str>>();

        let rhs_parts = chain_value
            .split(CHAIN_STAMP_VERSION_SEPARATOR)
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
