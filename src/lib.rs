// Copyright 2017 Amos Onn.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//! Wrapper around fuse-mt providing extra functionality for ease of implementation: `open` and
//! `opendir` methods now return a `FileLike` and `DirLike` object (respectively), which is then
//! passed to `read`, etc. calls (instead of a `fh: u64`).
#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        unstable_features,
        unused_import_braces, unused_qualifications)]

extern crate fuse_mt;
extern crate libc;
extern crate time;

mod fusefl;
mod handler_table;
mod filelike;

pub use fusefl::{
    CreatedEntryObj,
    ResultOpenObj,
    ResultCreateObj,
    FilesystemFL,
    FuseFL,
};
pub use filelike::{
    ReadFileLike,
    WriteFileLike,
    NoFile,
    ReadWriteAdaptor,
    ModalFileLike,
    FilesystemFLRwOpen,
    FilesystemFLOpen,
};
pub use fuse_mt::{
    RequestInfo,
    DirectoryEntry,
    Statfs,
    Xattr,
    ResultEmpty,
    ResultGetattr,
    ResultEntry,
    ResultReaddir,
    ResultData,
    ResultWrite,
    ResultStatfs,
    ResultXattr,
    FuseMT,
};

use std::result;
/// All results for fuse calls have `libc::c_int` as the Err variant.
pub type Result<T> = result::Result<T, libc::c_int>;

#[test]
fn it_works() {}
