set -e

# BUILD
docker build --tag=imager:latest .

# COPY - SETUP
mkdir -p release/linux/bin
DOCKER_CP_CODE='tar --to-stdout -C /bin --create imager imager.sha1'

# COPY TO DIST & LIB
docker run --rm imager:latest /bin/sh -c "$DOCKER_CP_CODE" | tar -x -C release/linux/bin

