// Copyright 2017 Amos Onn.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
//! Helpers for common pattern of `FilesystemFL::FileLike`: delegating reading, writing etc to the
//! objects.
//! Handler objects implement either both of `ReadFileLike` and `WriteFileLike`, for
//! general-purpose opening, and used with `FilesystemFLOpen`; or different handlers are used for
//! read-only, write-only and read-write opening, and used with `FilesystemFLRwOpen`.

use std::fs::File;
use std::cmp::min;
use std::cell::RefCell;
use std::sync::{Mutex, RwLock};
use std::os::unix::fs::FileExt;
use std::ffi::OsStr;
use std::path::Path;
use libc;

use super::fusefl::*;
use fuse_mt::*;
use super::Result;

/// Trait to be implemented for providing the "reader" functionality, to be used with
/// FilesystemFLOpen or FilesystemFLRwOpen.
pub trait ReadFileLike {
    /// Read data.
    /// Read should send exactly the number of bytes requested except on EOF or error,
    /// otherwise the rest of the data will be substituted with zeroes. An exception to
    /// this is when the file has been opened in 'direct_io' mode, in which case the
    /// return value of the read system call will reflect the return value of this
    /// operation.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;
}

/// Trait to be implemented for providing the "writer" functionality, to be used with
/// FilesystemFLOpen or FilesystemFLRwOpen.
pub trait WriteFileLike {
    /// Write data.
    /// Write should return exactly the number of bytes requested except on error. An
    /// exception to this is when the file has been opened in 'direct_io' mode, in
    /// which case the return value of the write system call will reflect the return
    /// value of this operation.
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;

    /// Synchronize file contents.
    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

//FIXME: both of the below impl-s don't guarantee filling the buffer!

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

impl ReadFileLike for [u8] {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        (&self).read_at(buf, offset)
    }
}

impl<'a> ReadFileLike for &'a [u8] {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        if offset > usize::max_value() as u64 {
            return Ok(0);
        }
        let offset = offset as usize;
        let len = min(buf.len(), self.len() - offset);
        buf[..len].copy_from_slice(&self[offset..offset + len]);
        Ok(len)
    }
}

fn do_write_at(this: &mut [u8], buf: &[u8], offset: u64) -> usize {
    if offset > usize::max_value() as u64 {
        return 0;
    }
    let offset = offset as usize;
    let len = min(buf.len(), this.len() - offset);
    this[offset..offset + len].copy_from_slice(&buf[..len]);
    len
}

impl WriteFileLike for RefCell<[u8]> {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        Ok(do_write_at(&mut *self.borrow_mut(), buf, offset))
    }
}

impl WriteFileLike for Mutex<[u8]> {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        Ok(do_write_at(&mut *self.lock().unwrap(), buf, offset))
    }
}

impl WriteFileLike for RwLock<[u8]> {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        Ok(do_write_at(&mut *self.write().unwrap(), buf, offset))
    }
}

/// Empty type for using with FilesystemFLRwOpen as the WriteLike and ReadWriteLike for readonly
/// fs-s (or the similar parallel for writeonly ones).
#[derive(Debug, Clone, Copy)]
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

/// Naive implementation of a read-write FileLike, given a read FileLike and
/// a write FileLike implementation.
#[derive(Debug)]
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

/// Implementation of a FileLike which can be either read-only, write-only or read-write.
/// Implementes both `ReadLike` and `WriteLike`, returning EBADF in case of trying to write to a
/// read-only file or vice-versa (just like you'd expect).
#[derive(Debug)]
pub enum ModalFileLike<R, W, RW> {
    /// Read-only file - will EBADF on `write` or `flush`.
    ReadOnly(R),
    /// Write-only file - will EBADF on `read`.
    WriteOnly(W),
    /// Read-write file - both ops are supported.
    ReadWrite(RW),
}

use self::ModalFileLike::*;


impl<R, W, RW> ReadFileLike for ModalFileLike<R, W, RW>
    where R: ReadFileLike,
          RW: ReadFileLike {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        match *self {
            ReadOnly(ref r) => r.read_at(buf, offset),
            WriteOnly(_) => Err(libc::EBADF),
            ReadWrite(ref rw) => rw.read_at(buf, offset),
        }
    }
}

impl<R, W, RW> WriteFileLike for ModalFileLike<R, W, RW>
    where W: WriteFileLike,
          RW: WriteFileLike {
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

/// Trait for using different types for the read, write and read-write files. The read-write type
/// can be a ReadWriteAdaptor over the read and write ones.
/// Everything implementing this implements `FilesystemFLOpen`, dispatching the open and create calls
/// according to the `ACCMODE` to the open or create method for the right type of file.
/// NOTE: fsync_metadata is not dispatched, but rather gets an enum over the possible file types,
/// because persumably the synchronization of metadata is mostly unaffected by how the file was
/// opened.
/// NOTE: under default impl, all create methods return `ENOSYS` (to enable fallback on mknod+open).
/// If you want to impl create_read for a write-only fs (or vice-versa), you MUST reimplement
/// create_write and create_readwrite to return `EROFS` instead, or create_read and
/// create_readwrite to return `EACCES`.
pub trait FilesystemFLRwOpen {
    /// Type for read-only file handlers.
    type ReadLike: ReadFileLike; // = NoFile;
    /// Type for write-only file handlers.
    type WriteLike: WriteFileLike; // = NoFile;
    /// Type for read-write file handlers.
    type ReadWriteLike: ReadFileLike + WriteFileLike;
        // = ReadWriteAdaptor<Self::ReadLike, Self::WriteLike>;

    /// Open a file read-only.
    fn open_read(&self,
                 _req: RequestInfo,
                 _path: &Path,
                 _flags: u32)
                 -> ResultOpenObj<Self::ReadLike> {
        Err(libc::EACCES)
    }

    /// Open a file write-only.
    fn open_write(&self,
                  _req: RequestInfo,
                  _path: &Path,
                  _flags: u32)
                  -> ResultOpenObj<Self::WriteLike> {
        Err(libc::EROFS)
    }

    /// Open a file read-write.
    fn open_readwrite(&self,
                      _req: RequestInfo,
                      _path: &Path,
                      _flags: u32)
                      -> ResultOpenObj<Self::ReadWriteLike> {
        Err(libc::EROFS)
    }

    /// Create a file, open for read-only.
    fn create_read(&self,
                   _req: RequestInfo,
                   _parent: &Path,
                   _name: &OsStr,
                   _mode: u32,
                   _flags: u32)
                   -> ResultCreateObj<Self::ReadLike> {
        Err(libc::ENOSYS)
    }

    /// Create a file, open for write-only.
    fn create_write(&self,
                    _req: RequestInfo,
                    _parent: &Path,
                    _name: &OsStr,
                    _mode: u32,
                    _flags: u32)
                    -> ResultCreateObj<Self::WriteLike> {
        Err(libc::ENOSYS)
    }

    /// Create a file, open for read-write.
    fn create_readwrite(&self,
                        _req: RequestInfo,
                        _parent: &Path,
                        _name: &OsStr,
                        _mode: u32,
                        _flags: u32)
                        -> ResultCreateObj<Self::ReadWriteLike> {
        Err(libc::ENOSYS)
    }

    /// `fsync` (i.e. flush) only the metadata of a file (with given path and handler). For
    /// `fsync`-ing the contents of the file, implement `WriteFileLike::flush` for
    /// `Self::WriteLike` and `Self::ReadWriteLike`.
    /// Note that the file handler may be of any of the types (enum-ed by `ModalFileLike`), since
    /// we assume metadata flushing does not vary greatly depending on the file opening mode.
    fn fsync_metadata(&self,
                      _req: RequestInfo,
                      _path: &Path,
                      _fl: &ModalFileLike<Self::ReadLike, Self::WriteLike, Self::ReadWriteLike>)
                      -> ResultEmpty {
        Err(libc::ENOSYS)
    }
}

/// Trait for standard usecase of FilesystemFL - open and create methods return a FileLike object,
/// which supports reading, writing and flushing of data, and then these calls are passed directly
/// to it. Once specialization (RFC #1210) lands, this will be integrated as a default impl of
/// FilesystemFL for types implementing this; for now, this repeats some methods from FilesystemFL,
/// and these should be manually called in the implementation of FilesystemFL.
// Part of this will become a default impl of FilesystemFL when RFC #1210 lands.
pub trait FilesystemFLOpen {
    /// The type of a file handler used by this FS.
    type FileLike: ReadFileLike + WriteFileLike;

    /// Open a file - matches `FilesystemFL::open` for overriding, see there.
    /// This should be implemented.
    fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::FileLike> {
        Err(libc::ENOSYS)
    }

    /// Create a file - matches `FilesystemFL::create` for overriding, see there.
    /// If this method is not implemented or under Linux kernel versions earlier than 2.6.15, the
    /// mknod() and open() methods will be called instead.
    fn create(&self,
              _req: RequestInfo,
              _parent: &Path,
              _name: &OsStr,
              _mode: u32,
              _flags: u32)
              -> ResultCreateObj<Self::FileLike> {
        Err(libc::ENOSYS)
    }

    /// Read from a file - matches `FilesystemFL::read` for overriding, see there.
    /// This provides the functionality of this trait.
    fn read(&self,
            _req: RequestInfo,
            _path: &Path,
            _fl: &Self::FileLike,
            _offset: u64,
            _size: u32)
            -> ResultData {
        let _size = _size as usize;
        let mut vec = Vec::<u8>::with_capacity(_size);
        unsafe { vec.set_len(_size) };
        let num_read = _fl.read_at(vec.as_mut_slice(), _offset)?;
        assert!(num_read <= _size);
        unsafe { vec.set_len(num_read) };
        Ok(vec)
    }

    /// Write from a file - matches `FilesystemFL::write` for overriding, see there.
    /// This provides the functionality of this trait.
    fn write(&self,
             _req: RequestInfo,
             _path: &Path,
             _fl: &Self::FileLike,
             _offset: u64,
             _data: Vec<u8>,
             _flags: u32)
             -> ResultWrite {
        assert!(_data.len() <= u32::max_value() as usize);
        _fl.write_at(_data.as_slice(), _offset).map(|x| x as u32)
    }

    /// Fsync a file - matches `FilesystemFL::fsync` for overriding, see there.
    /// This provides the functionality of this trait.
    fn fsync(&self,
             _req: RequestInfo,
             _path: &Path,
             _fl: &Self::FileLike,
             _datasync: bool)
             -> ResultEmpty {
        _fl.flush()?;
        if !_datasync {
            self.fsync_metadata(_req, _path, _fl)
        } else {
            Ok(())
        }
    }

    /// `fsync` (i.e. flush) only the metadata of a file (with given path and handler). For
    /// `fsync`-ing the contents of the file, implement `WriteFileLike::flush` for
    /// `Self::FileLike`.
    /// This should be implemented, it is used by `fsync` (in conjuction with `FileLike::flush`).
    fn fsync_metadata(&self, _req: RequestInfo, _path: &Path, _fl: &Self::FileLike) -> ResultEmpty {
        Err(libc::ENOSYS)
    }
}

// Part of this will become a default impl of FilesystemFL when RFC #1210 lands.
impl<T> FilesystemFLOpen for T where T: FilesystemFLRwOpen {
    type FileLike = ModalFileLike<<Self as FilesystemFLRwOpen>::ReadLike,
        <Self as FilesystemFLRwOpen>::WriteLike,
                  <Self as FilesystemFLRwOpen>::ReadWriteLike>;

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
    fn create(&self,
              _req: RequestInfo,
              _parent: &Path,
              _name: &OsStr,
              _mode: u32,
              _flags: u32)
              -> ResultCreateObj<Self::FileLike> {
        match _flags as i32 & libc::O_ACCMODE {
            libc::O_RDONLY => {
                map_res_create(self.create_read(_req, _parent, _name, _mode, _flags),
                               |fl| ReadOnly(fl))
            }
            libc::O_WRONLY => {
                map_res_create(self.create_write(_req, _parent, _name, _mode, _flags),
                               |fl| WriteOnly(fl))
            }
            libc::O_RDWR => {
                map_res_create(self.create_readwrite(_req, _parent, _name, _mode, _flags),
                               |fl| ReadWrite(fl))
            }
            _ => Err(libc::EINVAL),
        }
    }

    fn fsync_metadata(&self, _req: RequestInfo, _path: &Path, _fl: &Self::FileLike) -> ResultEmpty {
        FilesystemFLRwOpen::fsync_metadata(self, _req, _path, _fl)
    }
}
