# FUSE-FL

FL stands for File-like. This is an additional wrapper above the `fuse-mt` crate, which provides some Rust ergonomics, aiming to provide even more:
* `open` returns a `File`-like struct, meaning it `impl`-s the traits `Read`, `Write` and `Seek`, which is cached with the file handler.
* `read`, `write`, `flush` and `fsync` are implemented using the methods of the above struct.
