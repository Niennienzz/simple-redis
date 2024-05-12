use std::ops::Deref;
use std::sync::Arc;

use dashmap::{DashMap, DashSet};

use crate::RespFrame;

#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug)]
pub struct BackendInner {
    pub(crate) string: DashMap<String, RespFrame>,
    pub(crate) hmap: DashMap<String, DashMap<String, RespFrame>>,
    pub(crate) set: DashMap<String, DashSet<String>>,
}

impl Deref for Backend {
    type Target = BackendInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self(Arc::new(BackendInner::default()))
    }
}

impl Default for BackendInner {
    fn default() -> Self {
        Self {
            string: DashMap::new(),
            hmap: DashMap::new(),
            set: DashMap::new(),
        }
    }
}

impl Backend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn string_get(&self, key: &str) -> Option<RespFrame> {
        self.string.get(key).map(|v| v.value().clone())
    }

    pub fn string_set(&self, key: String, value: RespFrame) {
        self.string.insert(key, value);
    }

    pub fn hash_get(&self, key: &str, field: &str) -> Option<RespFrame> {
        self.hmap
            .get(key)
            .and_then(|v| v.get(field).map(|v| v.value().clone()))
    }

    pub fn hash_set(&self, key: String, field: String, value: RespFrame) {
        let hmap = self.hmap.entry(key).or_default();
        hmap.insert(field, value);
    }

    pub fn hash_get_all(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.clone())
    }

    pub fn set_add(&self, key: String, members: Vec<String>) -> i64 {
        let mut count = 0;
        let set = self.set.entry(key).or_default();
        for member in members {
            if set.insert(member.clone()) {
                count += 1;
            }
        }
        count
    }

    // Check if members are in the set
    pub fn set_is_member(&self, key: &str, member: &str) -> i64 {
        match self.set.get(key) {
            Some(set) => if set.contains(member) { 1 } else { 0 },
            None => 0,
        }
    }

    pub fn set_members(&self, key: &str) -> Option<DashSet<String>> {
        self.set.get(key).map(|v| v.clone())
    }
}
