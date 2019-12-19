###############################################################################
# BUILD PHASE
###############################################################################
FROM rust:latest as build

# SETUP
WORKDIR /code/
RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install software-properties-common curl git vim tree

# SYSTEM DEPENDENCIES
RUN apt-get install -y \
    build-essential \
    llvm-dev \
    libclang-dev \
    clang \
    openssl \
    pkg-config \
    libssl-dev \
    xz-utils

# BUILD PROJECT DEPENDENCIES FIRST
RUN mkdir -p src
RUN echo 'fn main() {panic!("stub")}' > src/main.rs
ADD Cargo.toml .
RUN cargo build --release

# ASSETS
ADD assets/test assets/test

# BUILD PROJECT CODE
RUN rm target/release/deps/imager-*
ADD src src
RUN cargo build --release

# INSTALL
RUN cargo install --force --path .


###############################################################################
# RUNTIME ENVIRONMENT
###############################################################################
FROM ubuntu:18.04 as runtime

# SETUP
RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install build-essential software-properties-common curl git vim tree
COPY --from=build /usr/local/cargo/bin/imager /bin/imager

# # SECURITY & SANITY CHECK
RUN sha1sum /bin/imager > /bin/imager.sha1