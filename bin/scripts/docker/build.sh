set -e

# BUILD
docker build --tag=imager-bins:latest .

# COPY - SETUP
# DOCKER_CP_CODE='tar --to-stdout -C dist/native --create libimager_nodejs.linux.node libimager_nodejs.linux.node.sha1'

# COPY TO DIST & LIB
# docker run --rm imager-rs:latest /bin/sh -c "$DOCKER_CP_CODE" | tar -x -C dist/native
# docker run --rm imager-rs:latest /bin/sh -c "$DOCKER_CP_CODE" | tar -x -C lib/native

# CLEANUP
# rm lib/native/libimager_nodejs.linux.node.sha1

