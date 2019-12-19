# NOTE: Assumes the host is a mac (since we need to build mac binaries too).
set -e

# CURRENT VERSION
IMAGER_VERSION=$(cargo metadata --format-version 1 | jq -r '.packages | map(select(.name=="imager")) | .[] | .version')

# CHECK COMMIT
GIT_RESULT=$(git status --porcelain)
if [[ "$GIT_RESULT" != "" ]]
then
    echo "changed files needing to be committed"
    exit 1
fi

# TAG COMMIT
git tag -a auto-release/imager/$IMAGER_VERSION -m "Imager Release"
git push origin auto-release/imager/$IMAGER_VERSION


# BUILD LINUX
./scripts/docker/build.sh

# BUILD APPLE
mkdir -p release/apple
cargo install --force --root release/apple --path .
rm release/apple/.crates.toml

# CHECKS - LINUX
test -f release/linux/bin/imager || (echo "FAILED!"; exit 1)
test -f release/linux/bin/imager.sha1 || (echo "FAILED!"; exit 1)

# CHECKS - APPLE
test -f release/apple/bin/imager || (echo "FAILED!"; exit 1)

# COMPRESS
tar -cvzf release/imager-v$IMAGER_VERSION-linux.tar.gz -C release linux
tar -cvzf release/imager-v$IMAGER_VERSION-apple.tar.gz -C release apple