###############################################################################
# BUILD PHASE
###############################################################################
FROM rust:latest as build

# SETUP
WORKDIR /code/
RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install build-essential software-properties-common curl git vim tree


# SYSTEM DEPENDENCIES
RUN apt-get install -y llvm-dev libclang-dev clang openssl pkg-config libssl-dev

# BUILD PROJECT DEPENDENCIES FIRST
RUN mkdir -p imager/src
RUN echo 'fn main() {panic!("stub")}' > imager/src/main.rs
ADD Cargo.toml .
ADD imager/Cargo.toml imager/Cargo.toml
RUN cargo build --release

# BUILD PROJECT CODE
RUN rm target/release/deps/imager-*
ADD imager/src imager/src
RUN cargo build --release

# INSTALL
RUN cargo install --force --path imager


###############################################################################
# RUNTIME ENVIRONMENT
###############################################################################
# FROM ubuntu:18.04 as runtime

# # SETUP
# RUN apt-get -y update && \
#     apt-get -y upgrade && \
#     apt-get -y install build-essential software-properties-common curl git vim tree
# COPY --from=build /usr/local/cargo/bin/imager /bin/imager
# COPY --from=build /usr/local/cargo/bin/imager-server /bin/imager-server

# # SECURITY & SANITY CHECK
# RUN sha1sum /bin/imager > /bin/imager.sha1
# RUN sha1sum /bin/imager-server > /bin/imager-server.sha1