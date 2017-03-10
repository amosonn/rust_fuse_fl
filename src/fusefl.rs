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

/// The return value for `create`: contains info on the newly-created file, as well as a FileLike
/// object to handle the opened file.
pub struct CreatedEntryObj<T> {
    pub ttl: Timespec,
    pub attr: FileAttr,
    pub fl: T,
    pub flags: u32,
}

pub type ResultOpenObj<T> = Result<(T, u32), libc::c_int>;
pub type ResultCreateObj<T> = Result<CreatedEntryObj<T>, libc::c_int>;


/// This trait must be implemented to implement a filesystem with FuseMT.
pub trait FilesystemFL {

    type FileLike;
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
    fn getattr(&self, _req: RequestInfo, _path: &Path, _fl: Option<&RwLock<Self::FileLike>>) -> ResultGetattr {
        Err(libc::ENOSYS)
    }

    // The following operations in the FUSE C API are all one kernel call: setattr
    // We split them out to match the C API's behavior.

    /// Change the mode of a filesystem entry.
    ///
    /// * `fl`: a FileLike object if this is called on an open file.
    /// * `mode`: the mode to change the file to.
    fn chmod(&self, _req: RequestInfo, _path: &Path, _fl: Option<&RwLock<Self::FileLike>>, _mode: u32) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Change the owner UID and/or group GID of a filesystem entry.
    ///
    /// * `fl`: a FileLike object if this is called on an open file.
    /// * `uid`: user ID to change the file's owner to. If `None`, leave the UID unchanged.
    /// * `gid`: group ID to change the file's group to. If `None`, leave the GID unchanged.
    fn chown(&self, _req: RequestInfo, _path: &Path, _fl: Option<&RwLock<Self::FileLike>>, _uid: Option<u32>, _gid: Option<u32>) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Set the length of a file.
    ///
    /// * `fl`: a FileLike object if this is called on an open file.
    /// * `size`: size in bytes to set as the file's length.
    fn truncate(&self, _req: RequestInfo, _path: &Path, _fl: Option<&RwLock<Self::FileLike>>, _size: u64) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Set timestamps of a filesystem entry.
    ///
    /// * `fl`: a FileLike object if this is called on an open file.
    /// * `atime`: the time of last access.
    /// * `mtime`: the time of last modification.
    fn utimens(&self, _req: RequestInfo, _path: &Path, _fl: Option<&RwLock<Self::FileLike>>, _atime: Option<Timespec>, _mtime: Option<Timespec>) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Set timestamps of a filesystem entry (with extra options only used on MacOS).
    #[allow(unknown_lints, too_many_arguments)]
    fn utimens_macos(&self, _req: RequestInfo, _path: &Path, _fl: Option<&RwLock<Self::FileLike>>, _crtime: Option<Timespec>, _chgtime: Option<Timespec>, _bkuptime: Option<Timespec>, _flags: Option<u32>) -> ResultEmpty {
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
    /// * `rdev`: if mode has the bits `S_IFCHR` or `S_IFBLK` set, this is the major and minor numbers for the device file. Otherwise it should be ignored.
    fn mknod(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32, _rdev: u32) -> ResultEntry {
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
    fn symlink(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _target: &Path) -> ResultEntry {
        Err(libc::ENOSYS)
    }

    /// Rename a filesystem entry.
    ///
    /// * `parent`: path to the directory containing the existing entry.
    /// * `name`: name of the existing entry.
    /// * `newparent`: path to the directory it should be renamed into (may be the same as `parent`).
    /// * `newname`: name of the new entry.
    fn rename(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _newparent: &Path, _newname: &OsStr) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Create a hard link.
    ///
    /// * `path`: path to an existing file.
    /// * `newparent`: path to the directory for the new link.
    /// * `newname`: name for the new link.
    fn link(&self, _req: RequestInfo, _path: &Path, _newparent: &Path, _newname: &OsStr) -> ResultEntry {
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
    fn read(&self, _req: RequestInfo, _path: &Path, _fl: &RwLock<Self::FileLike>, _offset: u64, _size: u32) -> ResultData {
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
    fn write(&self, _req: RequestInfo, _path: &Path, _fl: &RwLock<Self::FileLike>, _offset: u64, _data: Vec<u8>, _flags: u32) -> ResultWrite {
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
    fn flush(&self, _req: RequestInfo, _path: &Path, _fl: &RwLock<Self::FileLike>, _lock_owner: u64) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Write out any pending changes of a file.
    ///
    /// When this returns, data should be written to persistent storage.
    ///
    /// * `path`: path to the file.
    /// * `fl`: FileLike object returned from the `open` call.
    /// * `datasync`: if `false`, just write metadata, otherwise also write file data.
    fn fsync(&self, _req: RequestInfo, _path: &Path, _fl: &RwLock<Self::FileLike>, _datasync: bool) -> ResultEmpty {
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
    fn opendir(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpenObj<Self::DirLike> {
        Err(libc::ENOSYS)
    }

    /// Get the entries of a directory.
    ///
    /// * `path`: path to the directory.
    /// * `dl`: DirLike object returned from the `opendir` call.
    ///
    /// Return all the entries of the directory.
    fn readdir(&self, _req: RequestInfo, _path: &Path, _dl: &RwLock<Self::DirLike>) -> ResultReaddir {
        Err(libc::ENOSYS)
    }

    /// Close an open directory.
    ///
    /// This will be called exactly once for each `opendir` call.
    ///
    /// * `path`: path to the directory.
    /// * `dl`: DirLike object returned from the `opendir` call.
    /// * `flags`: the file access flags passed to the `opendir` call.
    fn releasedir(&self, _req: RequestInfo, _path: &Path, _dl: &RwLock<Self::DirLike>, _flags: u32) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Write out any pending changes to a directory.
    ///
    /// Analogous to the `fsync` call.
    fn fsyncdir(&self, _req: RequestInfo, _path: &Path, _dl: &RwLock<Self::DirLike>, _datasync: bool) -> ResultEmpty {
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
    fn setxattr(&self, _req: RequestInfo, _path: &Path, _name: &OsStr, _value: &[u8], _flags: u32, _position: u32) -> ResultEmpty {
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
    /// Return `Ok(())` if all requested permissions are allowed, otherwise return `Err(EACCESS)`
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
    fn create(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32, _flags: u32) -> ResultCreateObj<Self::FileLike> {
        Err(libc::ENOSYS)
    }

    // getlk

    // setlk

    // bmap
}
