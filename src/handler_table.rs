// Copyright 2017 Amos Onn.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
//
use std::collections::HashMap;
use std::sync::RwLock;


/// Table for storing objects for handlers, and issuing handlers for new objects, all available via
/// inner mutability.
pub struct HandlerTable<T> {
    inner: RwLock<(u64, HashMap<u64, T>)>,
}

impl<T> HandlerTable<T> {
    /// Create a new, empty HandlerTable.
    pub fn new() -> HandlerTable<T> {
        HandlerTable{ inner: RwLock::new((0, HashMap::new())) }
    }

    /// Get the object associated with a file handler, if it exists.
    pub fn get(&self, fh: u64) -> Option<T> {
        self.inner.read().unwrap().1.get(fh)
    }

    /// Insert a new object, returning the file handler generated for it.
    pub fn insert(&self, obj: T) -> u64 {
        let (fh, map) = self.inner.write().unwrap();
        assert!(map.insert(fh, obj).is_none());
        fh += 1;
        fh - 1
    }

    /// Remove an object associated with a file handler, if it exists.
    pub fn remove(&self, fh: u64) -> Option<T> {
        self.inner.write().unwrap().1.remove(fh)
    }
}
