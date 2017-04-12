// Copyright 2017 Amos Onn.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use fuse_mt::*;
use libc;
use time::Timespec;
  
use std::ffi::OsStr;
use std::path::Path;

use super::handler_table::HandlerTable;
use super::Result;

/// The return value for `create`: contains info on the newly-created file, as well as a FileLike
/// object to handle the opened file.
#[derive(Debug)]
pub struct CreatedEntryObj<T> {
    /// TTL of the created entry ? (TODO: check fuse docs)
    pub ttl: Timespec,
    /// Attributes of the created file
    pub attr: FileAttr,
    /// The handler object to be passed to calls on this file descriptor.
    pub fl: T,
    /// Creation flags, see fuse docs.
    pub flags: u32,
}

/// Result of an `open` call on FilesystemFL.
pub type ResultOpenObj<T> = Result<(T, u32)>;
/// Result of an `create` call on FilesystemFL.
pub type ResultCreateObj<T> = Result<CreatedEntryObj<T>>;

pub fn map_res_open<T, S, F>(this: ResultOpenObj<T>, f: F) -> ResultOpenObj<S>
    where F: FnOnce(T) -> S {
    this.map(|x| (f(x.0), x.1))
}

pub fn map_res_create<T, S, F>(this: ResultCreateObj<T>, f: F) -> ResultCreateObj<S>
    where F: FnOnce(T) -> S {
    match this {
        Ok(CreatedEntryObj {
               ttl,
               attr,
               fl,
               flags,
           }) => {
            Ok(CreatedEntryObj {
                   ttl,
                   attr,
                   fl: f(fl),
                   flags,
               })
        }
        Err(e) => Err(e),
    }
}

pub fn map_res_create2<T, F>(this: ResultCreateObj<T>, f: F) -> ResultCreate
    where F: FnOnce(T) -> u64 {
    match this {
        Ok(CreatedEntryObj {
               ttl,
               attr,
               fl,
               flags,
           }) => {
            Ok(CreatedEntry {
                   ttl,
                   attr,
                   fh: f(fl),
                   flags,
               })
        }
        Err(e) => Err(e),
    }
}


/// This trait must be implemented to implement a filesystem with FuseFL.
pub trait FilesystemFL {
    /// The type for objects returned by open/create and used by read, etc.
    type FileLike;
    /// The type for objects returned by opendir and used by readdir, etc.
    type DirLike;

    /// Called on mount, before any other function.
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        Err(0)
    }

    /// Called on filesystem unmount.
    fn destroy(&self, _req: RequestInfo) {
        // Nothing.
    }

    /// Look up a filesystem entry and get its attributes.
    ///
    /// * `parent`: path to the parent of the entry being looked up
    /// * `name`: the name of the entry (under `parent`) being looked up.
    fn lookup(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr) -> ResultEntry {
        Err(libc::ENOSYS)
    }

    /// Get the attributes of a filesystem entry.
    ///
    /// * `fl`: a FileLike object if this is called on an open file.
    fn getattr(&self,
               _req: RequestInfo,
               _path: &Path,
               _fl: Option<&Self::FileLike>)
               -> ResultGetattr {
        Err(libc::ENOSYS)
    }

    // The following operations in the FUSE C API are all one kernel call: setattr
    // We split them out to match the C API's behavior.

    /// Change the mode of a filesystem entry.
    ///
    /// * `fl`: a FileLike object if this is called on an open file.
    /// * `mode`: the mode to change the file to.
    fn chmod(&self,
             _req: RequestInfo,
             _path: &Path,
             _fl: Option<&Self::FileLike>,
             _mode: u32)
             -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Change the owner UID and/or group GID of a filesystem entry.
    ///
    /// * `fl`: a FileLike object if this is called on an open file.
    /// * `uid`: user ID to change the file's owner to. If `None`, leave the UID unchanged.
    /// * `gid`: group ID to change the file's group to. If `None`, leave the GID unchanged.
    fn chown(&self,
             _req: RequestInfo,
             _path: &Path,
             _fl: Option<&Self::FileLike>,
             _uid: Option<u32>,
             _gid: Option<u32>)
             -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Set the length of a file.
    ///
    /// * `fl`: a FileLike object if this is called on an open file.
    /// * `size`: size in bytes to set as the file's length.
    fn truncate(&self,
                _req: RequestInfo,
                _path: &Path,
                _fl: Option<&Self::FileLike>,
                _size: u64)
                -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Set timestamps of a filesystem entry.
    ///
    /// * `fl`: a FileLike object if this is called on an open file.
    /// * `atime`: the time of last access.
    /// * `mtime`: the time of last modification.
    fn utimens(&self,
               _req: RequestInfo,
               _path: &Path,
               _fl: Option<&Self::FileLike>,
               _atime: Option<Timespec>,
               _mtime: Option<Timespec>)
               -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Set timestamps of a filesystem entry (with extra options only used on MacOS).
    #[allow(unknown_lints, too_many_arguments)]
    fn utimens_macos(&self,
                     _req: RequestInfo,
                     _path: &Path,
                     _fl: Option<&Self::FileLike>,
                     _crtime: Option<Timespec>,
                     _chgtime: Option<Timespec>,
                     _bkuptime: Option<Timespec>,
                     _flags: Option<u32>)
                     -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    // END OF SETATTR FUNCTIONS

    /// Read a symbolic link.
    fn readlink(&self, _req: RequestInfo, _path: &Path) -> ResultData {
        Err(libc::ENOSYS)
    }

    /// Create a special file.
    ///
    /// * `parent`: path to the directory to make the entry under.
    /// * `name`: name of the entry.
    /// * `mode`: mode for the new entry.
    /// * `rdev`: if mode has the bits `S_IFCHR` or `S_IFBLK` set, this is the major and minor
    ///    numbers for the device file. Otherwise it should be ignored.
    fn mknod(&self,
             _req: RequestInfo,
             _parent: &Path,
             _name: &OsStr,
             _mode: u32,
             _rdev: u32)
             -> ResultEntry {
        Err(libc::ENOSYS)
    }

    /// Create a directory.
    ///
    /// * `parent`: path to the directory to make the directory under.
    /// * `name`: name of the directory.
    /// * `mode`: permissions for the new directory.
    fn mkdir(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32) -> ResultEntry {
        Err(libc::ENOSYS)
    }

    /// Remove a file.
    ///
    /// * `parent`: path to the directory containing the file to delete.
    /// * `name`: name of the file to delete.
    fn unlink(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Remove a directory.
    ///
    /// * `parent`: path to the directory containing the directory to delete.
    /// * `name`: name of the directory to delete.
    fn rmdir(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Create a symbolic link.
    ///
    /// * `parent`: path to the directory to make the link in.
    /// * `name`: name of the symbolic link.
    /// * `target`: path (may be relative or absolute) to the target of the link.
    fn symlink(&self,
               _req: RequestInfo,
               _parent: &Path,
               _name: &OsStr,
               _target: &Path)
               -> ResultEntry {
        Err(libc::ENOSYS)
    }

    /// Rename a filesystem entry.
    ///
    /// * `parent`: path to the directory containing the existing entry.
    /// * `name`: name of the existing entry.
    /// * `newparent`: path to the directory it should be renamed into (may be the same as
    ///   `parent`).
    /// * `newname`: name of the new entry.
    fn rename(&self,
              _req: RequestInfo,
              _parent: &Path,
              _name: &OsStr,
              _newparent: &Path,
              _newname: &OsStr)
              -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Create a hard link.
    ///
    /// * `path`: path to an existing file.
    /// * `newparent`: path to the directory for the new link.
    /// * `newname`: name for the new link.
    fn link(&self,
            _req: RequestInfo,
            _path: &Path,
            _newparent: &Path,
            _newname: &OsStr)
            -> ResultEntry {
        Err(libc::ENOSYS)
    }

    /// Open a file.
    ///
    /// * `path`: path to the file.
    /// * `flags`: one of `O_RDONLY`, `O_WRONLY`, or `O_RDWR`, plus maybe additional flags.
    ///
    /// Return a tuple of (file handle, flags). The file handle will be passed to any subsequent
    /// calls that operate on the file, and can be any value you choose, though it should allow
    /// your filesystem to identify the file opened even without any path info.
    fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::FileLike> {
        Err(libc::ENOSYS)
    }

    /// Read from a file.
    ///
    /// Note that it is not an error for this call to request to read past the end of the file, and
    /// you should only return data up to the end of the file (i.e. the number of bytes returned
    /// will be fewer than requested; possibly even zero). Do not extend the file in this case.
    ///
    /// * `path`: path to the file.
    /// * `fl`: FileLike object returned from the `open` call.
    /// * `offset`: offset into the file to start reading.
    /// * `size`: number of bytes to read.
    ///
    /// Return the bytes read.
    fn read(&self,
            _req: RequestInfo,
            _path: &Path,
            _fl: &Self::FileLike,
            _offset: u64,
            _size: u32)
            -> ResultData {
        Err(libc::ENOSYS)
    }

    /// Write to a file.
    ///
    /// * `path`: path to the file.
    /// * `fl`: FileLike object returned from the `open` call.
    /// * `offset`: offset into the file to start writing.
    /// * `data`: the data to write
    /// * `flags`:
    ///
    /// Return the number of bytes written.
    fn write(&self,
             _req: RequestInfo,
             _path: &Path,
             _fl: &Self::FileLike,
             _offset: u64,
             _data: Vec<u8>,
             _flags: u32)
             -> ResultWrite {
        Err(libc::ENOSYS)
    }

    /// Called each time a program calls `close` on an open file.
    ///
    /// Note that because file descriptors can be duplicated (by `dup`, `dup2`, `fork`) this may be
    /// called multiple times for a given file handle. The main use of this function is if the
    /// filesystem would like to return an error to the `close` call. Note that most programs
    /// ignore the return value of `close`, though.
    ///
    /// NOTE: the name of the method is misleading, since (unlike fsync) the filesystem is not
    /// forced to flush pending writes. One reason to flush data, is if the filesystem wants to
    /// return write errors. (Currently unsupported) If the filesystem supports file locking
    /// operations (setlk, getlk) it should remove all locks belonging to 'lock_owner'.
    ///
    /// * `path`: path to the file.
    /// * `fl`: FileLike object returned from the `open` call.
    /// * `lock_owner`: if the filesystem supports locking (`setlk`, `getlk`), remove all locks
    ///   belonging to this lock owner.
    fn flush(&self,
             _req: RequestInfo,
             _path: &Path,
             _fl: &Self::FileLike,
             _lock_owner: u64)
             -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Write out any pending changes of a file.
    ///
    /// When this returns, data should be written to persistent storage.
    ///
    /// * `path`: path to the file.
    /// * `fl`: FileLike object returned from the `open` call.
    /// * `datasync`: if `false`, just write metadata, otherwise also write file data.
    fn fsync(&self,
             _req: RequestInfo,
             _path: &Path,
             _fl: &Self::FileLike,
             _datasync: bool)
             -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Open a directory.
    ///
    /// Analogous to the `opend` call.
    ///
    /// * `path`: path to the directory.
    /// * `flags`: file access flags. Will contain `O_DIRECTORY` at least.
    ///
    /// Return a tuple of (file handle, flags). The file handle will be passed to any subsequent
    /// calls that operate on the directory, and can be any value you choose, though it should
    /// allow your filesystem to identify the directory opened even without any path info.
    fn opendir(&self,
               _req: RequestInfo,
               _path: &Path,
               _flags: u32)
               -> ResultOpenObj<Self::DirLike> {
        Err(libc::ENOSYS)
    }

    /// Get the entries of a directory.
    ///
    /// * `path`: path to the directory.
    /// * `dl`: DirLike object returned from the `opendir` call.
    ///
    /// Return all the entries of the directory.
    fn readdir(&self, _req: RequestInfo, _path: &Path, _dl: &Self::DirLike) -> ResultReaddir {
        Err(libc::ENOSYS)
    }

    /// Write out any pending changes to a directory.
    ///
    /// Analogous to the `fsync` call.
    fn fsyncdir(&self,
                _req: RequestInfo,
                _path: &Path,
                _dl: &Self::DirLike,
                _datasync: bool)
                -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Get filesystem statistics.
    ///
    /// * `path`: path to some folder in the filesystem.
    ///
    /// See the `Statfs` struct for more details.
    fn statfs(&self, _req: RequestInfo, _path: &Path) -> ResultStatfs {
        Err(libc::ENOSYS)
    }

    /// Set a file extended attribute.
    ///
    /// * `path`: path to the file.
    /// * `name`: attribute name.
    /// * `value`: the data to set the value to.
    /// * `flags`: can be either `XATTR_CREATE` or `XATTR_REPLACE`.
    /// * `position`: offset into the attribute value to write data.
    fn setxattr(&self,
                _req: RequestInfo,
                _path: &Path,
                _name: &OsStr,
                _value: &[u8],
                _flags: u32,
                _position: u32)
                -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Get a file extended attribute.
    ///
    /// * `path`: path to the file
    /// * `name`: attribute name.
    /// * `size`: the maximum number of bytes to read.
    ///
    /// If `size` is 0, return `Xattr::Size(n)` where `n` is the size of the attribute data.
    /// Otherwise, return `Xattr::Data(data)` with the requested data.
    fn getxattr(&self, _req: RequestInfo, _path: &Path, _name: &OsStr, _size: u32) -> ResultXattr {
        Err(libc::ENOSYS)
    }

    /// List extended attributes for a file.
    ///
    /// * `path`: path to the file.
    /// * `size`: maximum number of bytes to return.
    ///
    /// If `size` is 0, return `Xattr::Size(n)` where `n` is the size required for the list of
    /// attribute names.
    /// Otherwise, return `Xattr::Data(data)` where `data` is all the null-terminated attribute
    /// names.
    fn listxattr(&self, _req: RequestInfo, _path: &Path, _size: u32) -> ResultXattr {
        Err(libc::ENOSYS)
    }

    /// Remove an extended attribute for a file.
    ///
    /// * `path`: path to the file.
    /// * `name`: name of the attribute to remove.
    fn removexattr(&self, _req: RequestInfo, _path: &Path, _name: &OsStr) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Check for access to a file.
    ///
    /// * `path`: path to the file.
    /// * `mask`: mode bits to check for access to.
    ///
    /// Return `Ok(())` if all requested permissions are allowed, otherwise return `Err(EACCES)`
    /// or other error code as appropriate (e.g. `ENOENT` if the file doesn't exist).
    fn access(&self, _req: RequestInfo, _path: &Path, _mask: u32) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Create and open a new file.
    ///
    /// * `parent`: path to the directory to create the file in.
    /// * `name`: name of the file to be created.
    /// * `mode`: the mode to set on the new file.
    /// * `flags`: flags like would be passed to `open`.
    ///
    /// Return a `CreatedEntry` (which contains the new file's attributes as well as a file handle
    /// -- see documentation on `open` for more info on that).
    fn create(&self,
              _req: RequestInfo,
              _parent: &Path,
              _name: &OsStr,
              _mode: u32,
              _flags: u32)
              -> ResultCreateObj<Self::FileLike> {
        Err(libc::ENOSYS)
    }

    // getlk

    // setlk

    // bmap
}


/// Adaptor struct for using a filesystem - holds a FilesystemFL and implements FilesystemMT.
#[derive(Debug)]
pub struct FuseFL<T> where T: FilesystemFL {
    inner: T,
    files: HandlerTable<T::FileLike>,
    dirs: HandlerTable<T::DirLike>,
}


impl<T> FuseFL<T> where T: FilesystemFL {
    /// Build a new FuseFL from a given FilesystemFL.
    pub fn new(target_fs: T) -> FuseFL<T> {
        FuseFL {
            inner: target_fs,
            files: HandlerTable::new(),
            dirs: HandlerTable::new(),
        }
    }
}


impl<T: FilesystemFL + Sync + Send + 'static> FilesystemMT for FuseFL<T> {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        self.inner.init(_req)
    }

    fn destroy(&self, _req: RequestInfo) {
        self.inner.destroy(_req)
    }

    fn lookup(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr) -> ResultEntry {
        self.inner.lookup(_req, _parent, _name)
    }

    fn getattr(&self, _req: RequestInfo, _path: &Path, _fh: Option<u64>) -> ResultGetattr {
        if let Some(_fh) = _fh {
            self.inner.getattr(_req, _path, Some(self.files.get(_fh).unwrap()))
        } else {
            self.inner.getattr(_req, _path, None)
        }
    }

    // The following operations in the FUSE C API are all one kernel call: setattr
    // We split them out to match the C API's behavior.

    fn chmod(&self, _req: RequestInfo, _path: &Path, _fh: Option<u64>, _mode: u32) -> ResultEmpty {
        if let Some(_fh) = _fh {
            self.inner.chmod(_req, _path, Some(self.files.get(_fh).unwrap()), _mode)
        } else {
            self.inner.chmod(_req, _path, None, _mode)
        }
    }

    fn chown(&self,
             _req: RequestInfo,
             _path: &Path,
             _fh: Option<u64>,
             _uid: Option<u32>,
             _gid: Option<u32>)
             -> ResultEmpty {
        if let Some(_fh) = _fh {
            self.inner.chown(_req, _path, Some(self.files.get(_fh).unwrap()), _uid, _gid)
        } else {
            self.inner.chown(_req, _path, None, _uid, _gid)
        }
    }

    fn truncate(&self,
                _req: RequestInfo,
                _path: &Path,
                _fh: Option<u64>,
                _size: u64)
                -> ResultEmpty {
        if let Some(_fh) = _fh {
            self.inner.truncate(_req, _path, Some(self.files.get(_fh).unwrap()), _size)
        } else {
            self.inner.truncate(_req, _path, None, _size)
        }
    }

    fn utimens(&self,
               _req: RequestInfo,
               _path: &Path,
               _fh: Option<u64>,
               _atime: Option<Timespec>,
               _mtime: Option<Timespec>)
               -> ResultEmpty {
        if let Some(_fh) = _fh {
            self.inner.utimens(_req, _path, Some(self.files.get(_fh).unwrap()), _atime, _mtime)
        } else {
            self.inner.utimens(_req, _path, None, _atime, _mtime)
        }
    }

    #[allow(unknown_lints, too_many_arguments)]
    fn utimens_macos(&self,
                     _req: RequestInfo,
                     _path: &Path,
                     _fh: Option<u64>,
                     _crtime: Option<Timespec>,
                     _chgtime: Option<Timespec>,
                     _bkuptime: Option<Timespec>,
                     _flags: Option<u32>)
                     -> ResultEmpty {
        if let Some(_fh) = _fh {
            self.inner.utimens_macos(_req, _path, Some(self.files.get(_fh).unwrap()), _crtime, _chgtime, _bkuptime, _flags)
        } else {
            self.inner.utimens_macos(_req, _path, None, _crtime, _chgtime, _bkuptime, _flags)
        }
    }

    // END OF SETATTR FUNCTIONS

    fn readlink(&self, _req: RequestInfo, _path: &Path) -> ResultData {
        self.inner.readlink(_req, _path)
    }

    fn mknod(&self,
             _req: RequestInfo,
             _parent: &Path,
             _name: &OsStr,
             _mode: u32,
             _rdev: u32)
             -> ResultEntry {
        self.inner.mknod(_req, _parent, _name, _mode, _rdev)
    }

    fn mkdir(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32) -> ResultEntry {
        self.inner.mkdir(_req, _parent, _name, _mode)
    }

    fn unlink(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr) -> ResultEmpty {
        self.inner.unlink(_req, _parent, _name)
    }

    fn rmdir(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr) -> ResultEmpty {
        self.inner.rmdir(_req, _parent, _name)
    }

    fn symlink(&self,
               _req: RequestInfo,
               _parent: &Path,
               _name: &OsStr,
               _target: &Path)
               -> ResultEntry {
        self.inner.symlink(_req, _parent, _name, _target)
    }

    fn rename(&self,
              _req: RequestInfo,
              _parent: &Path,
              _name: &OsStr,
              _newparent: &Path,
              _newname: &OsStr)
              -> ResultEmpty {
        self.inner.rename(_req, _parent, _name, _newparent, _newname)
    }

    fn link(&self, _req: RequestInfo, _path: &Path, _newparent: &Path, _newname: &OsStr) -> ResultEntry {
        self.inner.link(_req, _path, _newparent, _newname)
    }

    fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpen {
        map_res_open(self.inner.open(_req, _path, _flags), |fl| self.files.insert(fl))
    }

    fn read(&self,
            _req: RequestInfo,
            _path: &Path,
            _fh: u64,
            _offset: u64,
            _size: u32)
            -> ResultData {
        self.inner.read(_req, _path, self.files.get(_fh).unwrap(), _offset, _size)
    }

    fn write(&self,
             _req: RequestInfo,
             _path: &Path,
             _fh: u64,
             _offset: u64,
             _data: Vec<u8>,
             _flags: u32)
             -> ResultWrite {
        self.inner.write(_req, _path, self.files.get(_fh).unwrap(), _offset, _data, _flags)
    }

    fn flush(&self, _req: RequestInfo, _path: &Path, _fh: u64, _lock_owner: u64) -> ResultEmpty {
        self.inner.flush(_req, _path, self.files.get(_fh).unwrap(), _lock_owner)
    }

    fn release(&self,
               _req: RequestInfo,
               _path: &Path,
               _fh: u64,
               _flags: u32,
               _lock_owner: u64,
               _flush: bool)
               -> ResultEmpty {
        let fl = self.files.remove(_fh).unwrap();
        if _flush {
            self.inner.flush(_req, _path, &fl, _lock_owner)
        } else {
            // TODO: handle unlocking anyway.
            Ok(())
        }
    }

    fn fsync(&self, _req: RequestInfo, _path: &Path, _fh: u64, _datasync: bool) -> ResultEmpty {
        self.inner.fsync(_req, _path, self.files.get(_fh).unwrap(), _datasync)
    }

    fn opendir(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpen {
        map_res_open(self.inner.opendir(_req, _path, _flags), |dl| self.dirs.insert(dl))
    }

    fn readdir(&self, _req: RequestInfo, _path: &Path, _fh: u64) -> ResultReaddir {
        self.inner.readdir(_req, _path, self.dirs.get(_fh).unwrap())
    }

    fn releasedir(&self, _req: RequestInfo, _path: &Path, _fh: u64, _flags: u32) -> ResultEmpty {
        self.dirs.remove(_fh).unwrap();
        Ok(())
    }

    fn fsyncdir(&self, _req: RequestInfo, _path: &Path, _fh: u64, _datasync: bool) -> ResultEmpty {
        self.inner.fsyncdir(_req, _path, self.dirs.get(_fh).unwrap(), _datasync)
    }

    fn statfs(&self, _req: RequestInfo, _path: &Path) -> ResultStatfs {
        self.inner.statfs(_req, _path)
    }

    fn setxattr(&self,
                _req: RequestInfo,
                _path: &Path,
                _name: &OsStr,
                _value: &[u8],
                _flags: u32,
                _position: u32)
                -> ResultEmpty {
        self.inner.setxattr(_req, _path, _name, _value, _flags, _position)
    }

    fn getxattr(&self, _req: RequestInfo, _path: &Path, _name: &OsStr, _size: u32) -> ResultXattr {
        self.inner.getxattr(_req, _path, _name, _size)
    }

    fn listxattr(&self, _req: RequestInfo, _path: &Path, _size: u32) -> ResultXattr {
        self.inner.listxattr(_req, _path, _size)
    }

    fn removexattr(&self, _req: RequestInfo, _path: &Path, _name: &OsStr) -> ResultEmpty {
        self.inner.removexattr(_req, _path, _name)
    }

    fn access(&self, _req: RequestInfo, _path: &Path, _mask: u32) -> ResultEmpty {
        self.inner.access(_req, _path, _mask)
    }

    fn create(&self,
              _req: RequestInfo,
              _parent: &Path,
              _name: &OsStr,
              _mode: u32,
              _flags: u32)
              -> ResultCreate {
        map_res_create2(self.inner.create(_req, _parent, _name, _mode, _flags), |fl| self.files.insert(fl))
    }

    // getlk

    // setlk

    // bmap
}
