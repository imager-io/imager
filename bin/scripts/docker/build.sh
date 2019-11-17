set -e

# BUILD
docker build --tag=imager-bins:latest .

# COPY - SETUP
mkdir -p release/linux/bin
DOCKER_CP_CODE='tar --to-stdout -C /bin --create imager imager-server imager.sha1 imager-server.sha1'

# COPY TO DIST & LIB
docker run --rm imager-bins:latest /bin/sh -c "$DOCKER_CP_CODE" | tar -x -C release/linux/bin


