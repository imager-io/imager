# Run the build phase.
set -e

docker build --tag=imager-bins-build:latest --target build .
docker run -it imager-bins-build:latest
