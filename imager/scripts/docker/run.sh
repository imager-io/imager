set -e

docker build --tag=imager-bins:latest .
docker run -it imager-bins:latest