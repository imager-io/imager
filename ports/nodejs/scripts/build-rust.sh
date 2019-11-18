set -e

case $PRE_BUILT_LIB_IMAGER_NODEJS in
    1)
        echo "[NOTE] Using prebuilt (native) libimager library."
        exit 0
        break
    ;;
esac

# BUILD
cargo build --release
mkdir -p dist/native

# COPY
if [[ "$OSTYPE" == "linux-gnu" ]]; then
    cp target/release/libimager_nodejs.so dist/native/libimager_nodejs.linux.node
    cp target/release/libimager_nodejs.so lib/native/libimager_nodejs.linux.node
elif [[ "$OSTYPE" == "darwin"* ]]; then
    cp target/release/libimager_nodejs.dylib dist/native/libimager_nodejs.apple.node
    cp target/release/libimager_nodejs.dylib lib/native/libimager_nodejs.apple.node
elif [[ "$OSTYPE" == "cygwin" ]]; then
    echo "Windows not yet supported."
    exit 1
elif [[ "$OSTYPE" == "msys" ]]; then
    echo "Windows not yet supported."
    exit 1
elif [[ "$OSTYPE" == "win32" ]]; then
    echo "Windows not yet supported."
    exit 1
elif [[ "$OSTYPE" == "freebsd"* ]]; then
    echo "FreeBSD not yet supported."
    exit 1
else
    echo "Unknown Platform"
    exit 1
fi

