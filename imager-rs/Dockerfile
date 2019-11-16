FROM rust:latest

# SYSTEM DEPENDENCIES
RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install build-essential software-properties-common curl git vim tree

# PROJECT DEPENDENCIES - CORE
RUN apt-get install -y llvm-dev libclang-dev clang

# PROJECT DEPENDENCIES
WORKDIR /code/
RUN apt-get install -y openssl pkg-config libssl-dev && apt-get clean
WORKDIR /code/
RUN mkdir -p src
RUN echo '' > ./src/lib.rs
RUN echo 'fn main() {panic!("stub")}' > ./src/main.rs
ADD ./Cargo.toml .
RUN cargo build --release
RUN cargo test

# PROJECT ASSETS
ADD ./assets/test ./assets/test

# # PROJECT APPLICATION
# RUN rm ./target/release/deps/imager.*
# ADD ./src/ ./src/
# RUN cargo test
# RUN cargo build --release
# RUN mkdir -p dist/native && \
#     cp target/release/libimager_nodejs.so dist/native/libimager_nodejs.linux.node
