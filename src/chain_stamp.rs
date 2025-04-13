use crate::core::generate_timebase_str_id;
use crate::CHAIN_STAMP_RIGHT_CHAIN;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
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
pub struct ChainStamp {
    pub stamp: String,
    pub timestamp: DateTime<Utc>,
    pub version: ChainStampVersion,
    pub root_stamp: Option<String>,
    pub child_stamp: Option<String>,
}

impl ChainStamp {
    pub fn build(root_cs: Option<ChainStamp>) -> ChainStamp {
        if let Some(root_stamp) = root_cs {
            return ChainStamp {
                child_stamp: None,
                timestamp: Utc::now(),
                version: ChainStampVersion::V1,
                stamp: generate_timebase_str_id(),
                root_stamp: Some(root_stamp.stamp_id().to_string()),
            };
        }
        ChainStamp {
            root_stamp: None,
            child_stamp: None,
            timestamp: Utc::now(),
            version: ChainStampVersion::V1,
            stamp: generate_timebase_str_id(),
        }
    }

    pub fn stamp_id(&self) -> &str {
        // sample => v1*9203923<39203823>390234082
        &*self.stamp
    }

    pub fn parent_chain_id(&self) -> Option<String> {
        if let Some(parent_stamp) = &self.root_stamp {
            return Some(parent_stamp.to_string());
        };
        None
    }

    pub fn is_parent_chain(&self, parent_chain_stamp: &ChainStamp) -> bool {
        if let Some(parent_stamp) = &self.root_stamp {
            return parent_stamp == parent_chain_stamp.stamp_id();
        }
        false
    }

    pub fn is_child_chain(&self, child_chain_stamp: &ChainStamp) -> bool {
        if let Some(child_stamp) = &self.child_stamp {
            return child_stamp == child_chain_stamp.stamp_id();
        }
        false
    }

    pub fn has_child(&self) -> bool {
        self.child_stamp.is_some()
    }

    pub fn version(&self) -> ChainStampVersion {
        *self.version
    }

    pub fn is_root(&self) -> bool {
        self.root_stamp.is_none()
    }

    pub fn append_child(&mut self, child_stamp: ChainStamp) -> Result<(), String> {
        if self.inner().contains(CHAIN_STAMP_RIGHT_CHAIN) {
            return Err("can't append a child to chain that already contains child".to_string());
        }
        if !child_stamp.parent_chain_id() {
            return Err("can't append child to chain whose not the parent".to_string());
        }
        let child_stamp_id = child_stamp.stamp_id().to_string();

        self.child_stamp = Some(child_stamp_id);
        Ok(())
    }

    /// ChainStamps are equal if
    ///
    ///     1. Their roots are the same
    ///     2. They have the same stamp_id
    ///     3. And their children are the same
    pub fn compare_to(&self, rhs: &ChainStamp) -> bool {
        if (*self.is_root() && !rhs.is_root()) || (!self.is_root() && rhs.is_root()) {
            return false;
        }
        if (*self.has_child() && !rhs.has_child()) || (!self.has_child() && rhs.has_child()) {
            return false;
        }
        // they are equal if their roots are the same,
        self.compare_roots(rhs)
            && *self.stamp_id().to_string() == rhs.stamp_id().to_string()
            && *self.compare_children(rhs)
    }

    fn compare_children(&self, rhs: &ChainStamp) -> bool {
        // return true if both self and rhs don't have children
        if !self.has_child() && !rhs.has_child() {
            return true;
        }
        // return false if one of them does not have a children
        if !self.has_child() || !rhs.has_child() {
            return false;
        }
        // it is safe to use unwrap here since we know both  lhs and rhs have children
        self.child_stamp.as_ref() == rhs.child_stamp.as_ref()
    }

    fn compare_roots(&self, rhs: &ChainStamp) -> bool {
        match (&self.root_stamp, &rhs.root_stamp) {
            // Both are None. They are equal according to your rule.
            (None, None) => true,
            // One is Some, the other is None. They are not equal.
            (Some(_), None) | (None, Some(_)) => false,
            // Both have values. lhs and rhs strings are bound as &String automatically.
            (Some(lhs), Some(rhs)) => lhs == rhs,
        }
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
        ChainStamp::compare_to(self, other)
    }
}

impl Display for ChainStamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, format!("v={}$[REDACTED]", self.version))
    }
}
