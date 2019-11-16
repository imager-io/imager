# Run the build phase.
set -e

docker build --tag=imager-rs-build:latest --target build .
docker run -it imager-rs-build:latest
