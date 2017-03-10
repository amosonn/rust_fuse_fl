# FUSE-FL

FL stands for File-like. This is an additional wrapper above the `fuse-mt` crate, which provides some Rust ergonomics, aiming to provide even more:
* `open` (and optionally `create`) returns a `File`-like struct, meaning it `impl`-s the traits `Read`, `Write` and `Seek`, which is cached with the file handler.
* `read`, `write`, `flush` and `fsync` are implemented using the methods of the above struct. `release` releases this struct.

Yet undecided:
* How to support file-locking (this is also pending on implementation by `fuse-mt`).
* How to support file-attribute management. Currently this is to be left to the "main" fs struct, disabling use of open file-handlers. Alternatively, this could be implemented by another trait on the `File`-like objects.
