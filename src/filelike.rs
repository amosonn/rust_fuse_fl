// Copyright 2017 Amos Onn.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use std::fs::File;
use std::os::unix::fs::FileExt;
use std::ffi::OsStr;
use std::path::Path;
use libc;

use super::fusefl::*;
use fuse_mt::*;
use super::Result;

pub trait ReadFileLike {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;
}

pub trait WriteFileLike {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

impl ReadFileLike for File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        FileExt::read_at(self, buf, offset).map_err(|x| x.raw_os_error().unwrap())
    }
}

impl WriteFileLike for File {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        FileExt::write_at(self, buf, offset).map_err(|x| x.raw_os_error().unwrap())
    }

    // NOTE: we can't use the flush method from Write, because that wants a &mut. However, for now
    // File's impl of flush is the same as ours, Ok(()), so we can stick with that.
    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

pub enum NoFile {}

#[allow(unused_variables)]
impl ReadFileLike for NoFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        Err(libc::ENOSYS)
    }
}

#[allow(unused_variables)]
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

impl<R, W> ReadFileLike for ReadWriteAdaptor<R, W> where R: ReadFileLike {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        self.reader.read_at(buf, offset)
    }
}

impl<R, W> WriteFileLike for ReadWriteAdaptor<R, W> where W: WriteFileLike {
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

use self::ModalFileLike::*;


impl<R, W, RW> ReadFileLike for ModalFileLike<R, W, RW> where 
    R: ReadFileLike, RW: ReadFileLike {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        match *self {
            ReadOnly(ref r) => r.read_at(buf, offset),
            WriteOnly(_) => Err(libc::EBADF),
            ReadWrite(ref rw) => rw.read_at(buf, offset),
        }
    }
}

impl<R, W, RW> WriteFileLike for ModalFileLike<R, W, RW> where 
    W: WriteFileLike, RW: WriteFileLike {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        match *self {
            ReadOnly(_) => Err(libc::EBADF),
            WriteOnly(ref w) => w.write_at(buf, offset),
            ReadWrite(ref rw) => rw.write_at(buf, offset),
        }
    }

    fn flush(&self) -> Result<()> {
        match *self {
            ReadOnly(_) => Err(libc::EBADF),
            WriteOnly(ref w) => w.flush(),
            ReadWrite(ref rw) => rw.flush(),
        }
    }
}

pub trait FilesystemFLRwOpen {
    type ReadLike: ReadFileLike; // = NoFile;
    type WriteLike: WriteFileLike; // = NoFile;
    type ReadWriteLike: ReadFileLike+WriteFileLike; // = ReadWriteAdaptor<Self::ReadLike, Self::WriteLike>;

    fn open_read(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::ReadLike> {
        Err(libc::ENOSYS)
    }

    fn open_write(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::WriteLike> {
        Err(libc::ENOSYS)
    }

    fn open_readwrite(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::ReadWriteLike> {
        Err(libc::ENOSYS)
    }

    fn create_read(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32, _flags: u32) -> ResultCreateObj<Self::ReadLike> {
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
        let _size = _size as usize;
        let mut vec = Vec::<u8>::with_capacity(_size);
        unsafe { vec.set_len(_size) };
        let num_read = _fl.read_at(vec.as_mut_slice(), _offset)?;
        assert!(num_read <= _size);
        unsafe { vec.set_len(num_read) };
        Ok(vec)
    }

    fn write(&self, _req: RequestInfo, _path: &Path, _fl: &Self::FileLike, _offset: u64, _data: Vec<u8>, _flags: u32) -> ResultWrite {
        assert!(_data.len() <= u32::max_value() as usize);
        _fl.write_at(_data.as_slice(), _offset).map(|x| x as u32)
    }

    fn fsync(&self, _req: RequestInfo, _path: &Path, _fl: &Self::FileLike, _datasync: bool) -> ResultEmpty {
        _fl.flush()
    }
}

// Part of this will become a default impl of FilesystemFL when RFC #1210 lands.
impl<T> FilesystemFLOpen for T where T: FilesystemFLRwOpen {
    type FileLike = ModalFileLike<
        <Self as FilesystemFLRwOpen>::ReadLike,
        <Self as FilesystemFLRwOpen>::WriteLike,
        <Self as FilesystemFLRwOpen>::ReadWriteLike,
    >;

    fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::FileLike> {
        match _flags as i32 & libc::O_ACCMODE {
            libc::O_RDONLY => map_res_open(self.open_read(_req, _path, _flags), |fl| ReadOnly(fl)),
            libc::O_WRONLY => map_res_open(self.open_write(_req, _path, _flags), |fl| WriteOnly(fl)),
            libc::O_RDWR => map_res_open(self.open_readwrite(_req, _path, _flags), |fl| ReadWrite(fl)),
            _ => Err(libc::EINVAL),
        }
    }

    /// If this method is not implemented or under Linux kernel versions earlier than 2.6.15, the
    /// mknod() and open() methods will be called instead.
    fn create(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32, _flags: u32) -> ResultCreateObj<Self::FileLike> {
        match _flags as i32 & libc::O_ACCMODE {
            libc::O_RDONLY => map_res_create(self.create_read(_req, _parent, _name, _mode, _flags), |fl| ReadOnly(fl)),
            libc::O_WRONLY => map_res_create(self.create_write(_req, _parent, _name, _mode, _flags), |fl| WriteOnly(fl)),
            libc::O_RDWR => map_res_create(self.create_readwrite(_req, _parent, _name, _mode, _flags), |fl| ReadWrite(fl)),
            _ => Err(libc::EINVAL),
        }
    }
}

