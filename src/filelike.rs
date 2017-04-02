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

pub type Result<T> = result::Result<T, libc::c_int>;

use super::fusefl::*;

pub trait ReadFileLike {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;
}

pub trait WriteFileLike {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

pub enum NoFile {};

impl ReadFileLike for NoFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        Err(libc::ENOSYS)
    }
}

impl WriteFileLike for NoFile {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        Err(libc::ENOSYS)
    }

    fn flush(&self) -> Result<()> {
        Err(libc::ENOSYS)
    }
}

pub struct ReadWriteAdaptor<R, W> {
    reader: R,
    writer: W,
}

impl<R, _> ReadFileLike for ReadWriteAdaptor<R, _> where R: ReadFileLike {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        self.reader.read_at(buf, offset)
    }
}

impl<_, W> WriteFileLike for ReadWriteAdaptor<_, W> where W: WriteFileLike {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        self.writer.write_at(buf, offset)
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


impl<R, _, RW> ReadFileLike for ModalFileLike<R, _, RW> where 
    R: ReadFileLike, RW: ReadFileLike {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        match self {
            ReadOnly(r) => r.read_at(buf, offset),
            WriteOnly(_) => Err(libc::EBADF),
            ReadWrite(rw) => rw.read_at(buf, offset),
        }
    }
}

impl<_, W, RW> WriteFileLike for ModalFileLike<_, W, RW> where 
    W: WriteFileLike, RW: WriteFileLike {
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

pub trait FilesystemFLRwOpen {
    type ReadLike: ReadFileLike = NoFile;
    type WriteLike: WriteFileLike = NoFile;
    type ReadWriteLike: ReadFileLike+WriteFileLike = ReadWriteAdaptor<Self::ReadLike, Self::WriteLike>;

    fn open_read(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::ReadLike> {
        Err(libc::ENOSYS)
    }

    fn open_write(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::WriteLike> {
        Err(libc::ENOSYS)
    }

    fn open_readwrite(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::ReadWriteLike> {
        Err(libc::ENOSYS)
    }

    fn create_write(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32, _flags: u32) -> ResultCreateObj<Self::WriteLike> {
        Err(libc::ENOSYS)
    }

    fn create_readwrite(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32, _flags: u32) -> ResultCreateObj<Self::ReadWriteLike> {
        Err(libc::ENOSYS)
    }
}

// Part of this will become a default impl of FilesystemFL when RFC #1210 lands.
pub trait FilesystemFLOpen {
    type FileLike: ReadFileLike+WriteFileLike;

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
