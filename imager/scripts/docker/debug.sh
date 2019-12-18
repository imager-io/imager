# Run the build phase.
set -e

docker build --tag=imager-build:latest --target build .
docker run -it imager-build:latest