// Copyright 2017 Amos Onn.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

extern crate fuse_mt;
extern crate libc;
extern crate time;

mod fusefl;
mod handler_table;
mod filelike;

pub use fusefl::*;

use std::result;
pub type Result<T> = result::Result<T, libc::c_int>;

#[test]
fn it_works() {
}
