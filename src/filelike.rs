// Copyright 2017 Amos Onn.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use std::io;
use std::os::unix::fs::FileExt;
use std::result;
use libc;

type Result<T> = Result<T, libc::c_int>;

use super::fusefl::ResultOpenObj;

pub trait ReadUnixExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;
}

pub trait WriteUnixExt {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;
}

impl<T> ReadUnixExt for T where T: FileExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        FileExt::read_at(self, buf, offset).map_err(|e| e.raw_os_error().map_or_else(|| libc::EIO, |x| x as libc::c_int))
    }
}
    
impl<T> WriteUnixExt for T where T: FileExt {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        FileExt::write_at(self, buf, offset).map_err(|e| e.raw_os_error().map_or_else(|| libc::EIO, |x| x as libc::c_int))
    }
}


pub struct ReadWriteAdaptor<R, W> {
    reader: R,
    writer: W,
}

impl<R, _> ReadUnixExt for ReadWriteAdaptor<R, _> where R: ReadUnixExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        self.reader.read_at(buf, offset)
    }
}

impl<_, W> WriteUnixExt for ReadWriteAdaptor<_, W> where W: WriteUnixExt {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        self.reader.write_at(buf, offset)
    }
}


pub enum ModalFileLike<R, W, RW> {
    ReadOnly(R),
    WriteOnly(W),
    ReadWrite(RW),
}


impl<R, _, RW> ReadUnixExt for ModalFileLike<R, _, RW> where 
    R: ReadUnixExt, RW: ReadUnixExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        match self {
            ReadOnly(r) => r.read_at(buf, offset),
            WriteOnly(_) => Err(libc::EBADF),
            ReadWrite(rw) => rw.read_at(buf, offset),
        }
    }
}

impl<_, W, RW> WriteUnixExt for ModalFileLike<_, W, RW> where 
    W: WriteUnixExt, RW: WriteUnixExt {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        match self {
            ReadOnly(_) => Err(libc::EBADF),
            WriteOnly(w) => w.write_at(buf, offset),
            ReadWrite(rw) => rw.write_at(buf, offset),
        }
    }
}
