# NOTE: Assumes the host is a mac (since we need to build mac binaries too).
set -e

# CHECK COMMIT
GIT_RESULT=$(git status --porcelain)
if [[ "$GIT_RESULT" != "" ]]
then
    echo "changed files needing to be committed"
    exit 1
fi

# BUILD LINUX
./scripts/docker/build.sh

# BUILD MACOS
./scripts/build-rust.sh

# CHECKS - LINUX
test -f dist/native/libimager_nodejs.linux.node || (echo "FAILED!"; exit 1)
test -f dist/native/libimager_nodejs.linux.node.sha1 || (echo "FAILED!"; exit 1)
test -f lib/native/libimager_nodejs.linux.node || (echo "FAILED!"; exit 1)

# CHECKS - MACOS
test -f dist/native/libimager_nodejs.apple.node || (echo "FAILED!"; exit 1)
test -f lib/native/libimager_nodejs.apple.node || (echo "FAILED!"; exit 1)

# TEST
npm test

# PUBLISH
npm publish