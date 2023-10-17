use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::Value;

#[derive(Debug, Clone)]
pub struct List(Arc<RwLock<Vec<Value>>>);

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        *self.0.read().unwrap() == *other.0.read().unwrap()
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let list = self.0.read().unwrap();
        let mut iter = list.iter();
        if let Some(value) = iter.next() {
            write!(f, "{value}")?;
            for value in iter {
                write!(f, ", {value}")?;
            }
        }
        write!(f, "]")
    }
}

impl List {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Vec::new())))
    }

    pub fn len(&self) -> i64 {
        self.0.read().unwrap().len() as i64
    }

    pub fn push(&self, value: Value) {
        self.0.write().unwrap().push(value);
    }

    pub fn read(&self) -> RwLockReadGuard<Vec<Value>> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<Vec<Value>> {
        self.0.write().unwrap()
    }
}
