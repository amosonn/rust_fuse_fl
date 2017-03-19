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
    inner: RwLock<InnerTable<T>>,
}

struct InnerTable<T> {
    next_fh: u64,
    map: HashMap<u64, T>,
}

impl<T> HandlerTable<T> {
    /// Create a new, empty HandlerTable.
    pub fn new() -> HandlerTable<T> {
        HandlerTable { inner: RwLock::new(InnerTable { 
            next_fh: 0, 
            map: HashMap::new(),
        } ) }
    }

    /// Get the object associated with a file handler, if it exists.
    pub fn get(&self, fh: u64) -> Option<&T> {
        self.inner.read().unwrap().map.get(&fh)
    }

    /// Insert a new object, returning the file handler generated for it.
    pub fn insert(&self, obj: T) -> u64 {
        let mut inner = self.inner.write().unwrap();
        let InnerTable { next_fh: ref mut fh, map: ref mut map } = *inner;
        assert!(map.insert(*fh, obj).is_none());
        *fh += 1;
        *fh - 1
    }

    /// Remove an object associated with a file handler, if it exists.
    pub fn remove(&self, fh: u64) -> Option<T> {
        self.inner.write().unwrap().map.remove(&fh)
    }
}
