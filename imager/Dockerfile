###############################################################################
# BUILD PHASE
###############################################################################
FROM rust:1.51.0 as build

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

# ASSETS
COPY assets/test assets/test

# BUILD PROJECT CODE
COPY src src
COPY Cargo.toml Cargo.toml
RUN cargo build --release

# INSTALL
RUN cargo install --force --path .


###############################################################################
# RUNTIME ENVIRONMENT
###############################################################################
FROM ubuntu:20.04 as runtime

ENV DEBIAN_FRONTEND=noninteractive

# SETUP
RUN apt-get -y update && \
  apt-get -y upgrade && \
  apt-get -y install build-essential software-properties-common curl git vim tree
COPY --from=build /usr/local/cargo/bin/imager /bin/imager

# # SECURITY & SANITY CHECK
RUN sha1sum /bin/imager > /bin/imager.sha1
