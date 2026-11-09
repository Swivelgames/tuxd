# Release runtime notices

`musl-COPYRIGHT.txt` is the unmodified [musl 1.2.5 notice](https://git.musl-libc.org/cgit/musl/plain/COPYRIGHT?h=v1.2.5), retained for the Rust 1.95.0 static-musl release build.

The `cargo xtask package` command also copies `share/doc/rust/COPYRIGHT-library.html` from the active Rust toolchain into both release archives. Recheck runtime versions and notices when changing the pinned release compiler; the command refuses an unreviewed compiler version.
