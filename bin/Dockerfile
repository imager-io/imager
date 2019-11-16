###############################################################################
# BUILD PHASE
###############################################################################
FROM rust:latest as build

# SYSTEM DEPENDENCIES
RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install build-essential software-properties-common curl git vim tree

# PROJECT DEPENDENCIES - CORE
RUN apt-get install -y llvm-dev libclang-dev clang

# PROJECT DEPENDENCIES
WORKDIR /code/
RUN apt-get install -y openssl pkg-config libssl-dev && apt-get clean


RUN mkdir -p imager-cli/src imager-server/src
RUN echo 'fn main() {panic!("stub")}' > imager-cli/src/main.rs
RUN echo 'fn main() {panic!("stub")}' > imager-server/src/main.rs
ADD Cargo.toml .
ADD imager-cli/Cargo.toml imager-cli/Cargo.toml
ADD imager-server/Cargo.toml imager-server/Cargo.toml
RUN cargo build --release
RUN cargo test

# # PROJECT ASSETS
# ADD ./assets/test ./assets/test

# # PROJECT APPLICATION
# RUN rm ./target/release/deps/imager*
# ADD ./src/ ./src/
# RUN cargo test
# RUN cargo build --release

# # INSTALL
# RUN cargo install --force --path .


# ###############################################################################
# # RUNTIME ENVIRONMENT
# ###############################################################################
# FROM ubuntu:18.04 as runtime

# # SETUP
# RUN apt-get -y update && \
#     apt-get -y upgrade && \
#     apt-get -y install build-essential software-properties-common curl git vim tree
# COPY --from=build /usr/local/cargo/bin/imager /bin/imager

# SECURITY & SANITY CHECK
# RUN sha1sum dist/native/libimager_nodejs.linux.node > dist/native/libimager_nodejs.linux.node.sha1
