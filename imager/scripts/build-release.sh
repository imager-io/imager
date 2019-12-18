# # NOTE: Assumes the host is a mac (since we need to build mac binaries too).
# set -e

# # CHECK COMMIT
# # GIT_RESULT=$(git status --porcelain)
# # if [[ "$GIT_RESULT" != "" ]]
# # then
# #     echo "changed files needing to be committed"
# #     exit 1
# # fi

# # BUILD LINUX
# # ./scripts/docker/build.sh

# # BUILD MACOS
# mkdir -p release/macos
# cargo install --force --root release/macos --path imager
# rm release/macos/.crates.toml

# # CHECKS - LINUX
# test -f release/linux/bin/imager || (echo "FAILED!"; exit 1)
# test -f release/linux/bin/imager.sha1 || (echo "FAILED!"; exit 1)

# # CHECKS - MACOS
# test -f release/macos/bin/imager || (echo "FAILED!"; exit 1)

# # TEST
# # npm test

# # PUBLISH
# # npm publish