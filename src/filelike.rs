// Copyright 2017 Amos Onn.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use std::result;
use libc;

type Result<T> = Result<T, libc::c_int>;

use super::fusefl::*;

pub trait ReadUnixExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;
}

pub trait WriteUnixExt {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;

    fn flush(&self) -> Result<()> {
        Ok(())
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

    fn flush(&self) -> Result<()> {
        self.writer.flush()
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

    fn flush(&self) -> Result<()> {
        match self {
            ReadOnly(_) => Err(libc::EBADF),
            WriteOnly(w) => w.flush(),
            ReadWrite(rw) => rw.flush(),
        }
    }
}

// Part of this will become a default impl of FilesystemFL when RFC #1210 lands.
pub trait FilesystemFLOpen {
    type FileLike: ReadUnixExt+WriteUnixExt;

    fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::FileLike> {
        Err(libc::ENOSYS)
    }

    /// If this method is not implemented or under Linux kernel versions earlier than 2.6.15, the
    /// mknod() and open() methods will be called instead.
    fn create(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32, _flags: u32) -> ResultCreateObj<Self::FileLike> {
        Err(libc::ENOSYS)
    }

    fn read(&self, _req: RequestInfo, _path: &Path, _fl: &Self::FileLike, _offset: u64, _size: u32) -> ResultData {
        let mut vec = Vec<u8>::with_capacity(_size);
        unsafe { vec.set_len(_size) };
        let num_read = _fl.read_at(vec.as_mut_slice(), _offset)?;
        assert!(num_read <= _size);
        unsafe { vec.set_len(num_read) };
        Ok(vec)
    }

    fn write(&self, _req: RequestInfo, _path: &Path, _fl: &Self::FileLike, _offset: u64, _data: Vec<u8>, _flags: u32) -> ResultWrite {
        assert!(_data.len() <= u32::max_value());
        _fl.write_at(vec.as_slice(), _offset).map(|x| x as u32)
    }

    fn fsync(&self, _req: RequestInfo, _path: &Path, _fl: &Self::FileLike, _datasync: bool) -> ResultEmpty {
        Err(libc::ENOSYS)
    }
}
