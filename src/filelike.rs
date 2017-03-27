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

use super::fusefl::ResultOpenObj;

pub trait ReadUnixExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize>;
}

pub trait WriteUnixExt {
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize>;
}

impl<T> ReadUnixExt for T where T: FileExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        FileExt::read_at(self, buf, offset)
    }
}
    
impl<T> WriteUnixExt for T where T: FileExt {
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        FileExt::write_at(self, buf, offset)
    }
}


pub struct ReadWriteAdaptor<R, W> {
    reader: R,
    writer: W,
}

impl<R, _> ReadUnixExt for ReadWriteAdaptor<R, _> where R: ReadUnixExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        self.reader.read_at(buf, offset)
    }
}

impl<_, W> WriteUnixExt for ReadWriteAdaptor<_, W> where W: WriteUnixExt {
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
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
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        match self {
            ReadOnly(r) => r.read_at(buf, offset),
            WriteOnly(_) => io::Error::new(io::ErrorKind::Other, "Reading from write-only file."),
            ReadWrite(rw) => rw.read_at(buf, offset),
        }
    }
}

impl<_, W, RW> WriteUnixExt for ModalFileLike<_, W, RW> where 
    W: WriteUnixExt, RW: WriteUnixExt {
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        match self {
            ReadOnly(_) => io::Error::new(io::ErrorKind::Other, "Writing to read-only file."),
            WriteOnly(w) => w.write_at(buf, offset),
            ReadWrite(rw) => rw.write_at(buf, offset),
        }
    }
}
