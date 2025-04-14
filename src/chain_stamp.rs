use crate::core::generate_timebase_str_id;
use crate::DomainError;
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

#[derive(Debug, Clone, Serialize, Eq)]
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

    /// Binding a chain stamp builds a chain stamp with the same chain id as the binding chain stamp
    /// But they have to have different parents.
    /// This type of build should be used when 2 transactions are happening between users.
    /// For, example, When a user credits a user's wallet, there should be a binding block (chain).
    pub fn build_bind(
        binding_cs: &ChainStamp,
        associate_root_cs: &ChainStamp,
    ) -> Result<ChainStamp, DomainError> {
        if let Some(binding_cs_root_stamp) = binding_cs.parent_chain_id() {
            if binding_cs_root_stamp == associate_root_cs.stamp_id() {
                return Err(DomainError::InvalidArgument(
                    "Can't build bind: Same parent as current chain stamp".to_string(),
                ));
            }
        }

        Ok(ChainStamp {
            child_stamp: None,
            timestamp: binding_cs.timestamp,
            version: binding_cs.version.clone(),
            stamp: binding_cs.stamp_id().to_string(),
            root_stamp: Some(associate_root_cs.stamp_id().to_string()),
        })
    }

    pub fn stamp_id(&self) -> &str {
        &*self.stamp
    }

    pub fn parent_chain_id(&self) -> Option<String> {
        if let Some(parent_stamp) = &self.root_stamp {
            return Some(parent_stamp.to_string());
        };
        None
    }

    pub fn is_parent(&self, parent_chain_stamp: &ChainStamp) -> bool {
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
        self.version.clone()
    }

    pub fn is_root(&self) -> bool {
        self.root_stamp.is_none()
    }

    pub fn append_child(&mut self, child_stamp: ChainStamp) -> Result<(), String> {
        if self.has_child() {
            return Err("can't append a child to chain that already contains child".to_string());
        }
        if !child_stamp.is_parent(&self) {
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
        if (self.is_root() && !rhs.is_root()) || (!self.is_root() && rhs.is_root()) {
            return false;
        }
        if (self.has_child() && !rhs.has_child()) || (!self.has_child() && rhs.has_child()) {
            return false;
        }
        // they are equal if their roots are the same,
        self.compare_roots(rhs)
            && self.stamp_id().to_string() == rhs.stamp_id().to_string()
            && self.compare_children(rhs)
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
        ChainStamp::compare_to(self, other)
    }
}

impl Display for ChainStamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "v={}$[REDACTED]", self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_partial_eq_similar_chain_stamps() {
        let chain_stamp = ChainStamp::build(None);
        let cloned_chain_stamp = chain_stamp.clone();

        assert_eq!(chain_stamp, cloned_chain_stamp);
    }

    #[test]
    pub fn test_partial_eq_un_similar_chain_stamps() {
        let root_stamp = ChainStamp::build(None);
        let chain_stamp = ChainStamp::build(Some(root_stamp.clone()));

        assert_ne!(chain_stamp, root_stamp);

        let childs_parent_stamp_id = chain_stamp
            .parent_chain_id()
            .expect("Failed to get parent chain id");
        assert_eq!(root_stamp.stamp_id(), childs_parent_stamp_id);
    }

    #[test]
    pub fn test_build_bind_between_binding_and_bound_chain() {
        let root_stamp = ChainStamp::build(None);
        let binding_chain_stamp = ChainStamp::build(None);
        let bound_chain_stamp = ChainStamp::build_bind(&binding_chain_stamp, &root_stamp)
            .expect("failed to build bind chain_stamp");

        let bound_cs_parent_id = bound_chain_stamp
            .parent_chain_id()
            .expect("failed to get parent chain id");
        assert_eq!(bound_cs_parent_id, root_stamp.stamp_id());
        assert_eq!(binding_chain_stamp.stamp_id(), bound_chain_stamp.stamp_id());
    }
}
