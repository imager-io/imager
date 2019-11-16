set -e

docker build --tag=imager-rs:latest .
docker run -it imager-rs:latest
