###############################################################################
# BUILD
###############################################################################
cargo build --release --target=x86_64-apple-darwin

###############################################################################
# COPY
###############################################################################
mkdir -p dist/native lib/native
cp  ../../target/x86_64-apple-darwin/release/libimager_nodejs.dylib \
    dist/native/libimager_nodejs.apple.node

cp  ../../target/x86_64-apple-darwin/release/libimager_nodejs.dylib \
    lib/native/libimager_nodejs.apple.node