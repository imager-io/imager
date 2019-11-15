set -e

# BUILD
docker build --tag=imager-nodejs:latest .

# COPY - SETUP
mkdir -p dist/native lib/native
DOCKER_CP_CODE='tar --to-stdout -C dist/native --create libimager_nodejs.linux.node libimager_nodejs.linux.node.sha1'

# COPY TO DIST & LIB
docker run --rm imager-nodejs:latest /bin/sh -c "$DOCKER_CP_CODE" | tar -x -C dist/native
docker run --rm imager-nodejs:latest /bin/sh -c "$DOCKER_CP_CODE" | tar -x -C lib/native

# CLEANUP
rm lib/native/libimager_nodejs.linux.node.sha1

