# Warning
Still under initial development. Not yet ready for release!

# Ongoing Development Issues

## Building & Distribution Issues

While the Linux target is building (on my Mac VIA cargo cross compilation), it just occurred to me that Imager depends on lower-level C/C++ dependencies that complicate things (projects that can’t simply be rewritten in Rust)… 

For now, the best solution may be to just build linux binaries VIA Docker. This way we can also run tests for linux builds within a linux instance.

