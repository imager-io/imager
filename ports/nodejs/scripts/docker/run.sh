set -e

docker build --tag=imager-nodejs:latest .
docker run -it imager-nodejs:latest
