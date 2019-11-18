###############################################################################
# RUST (NATIVE) BUILD
###############################################################################
FROM rust:latest as build-rust

# SYSTEM DEPENDENCIES
RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install build-essential software-properties-common curl git vim tree

# PROJECT DEPENDENCIES - CORE
RUN apt-get install -y llvm-dev libclang-dev clang

# SETUP
WORKDIR /code/

# PROJECT - SPECIAL
ADD .cargo .cargo

# PROJECT DEPENDENCIES
RUN apt-get install -y openssl pkg-config libssl-dev && apt-get clean
RUN mkdir -p src
RUN echo '' > ./src/lib.rs
ADD ./Cargo.toml .
RUN cargo build --release

# PROJECT ASSETS
ADD ./assets/test ./assets/test

# PROJECT APPLICATION
RUN rm ./target/release/deps/libimager_nodejs* && \
    rm ./target/release/deps/imager_nodejs* && \
    rm ./target/release/libimager_nodejs*
ADD ./src/ ./src/
RUN cargo build --release
RUN mkdir -p dist/native && \
    cp target/release/libimager_nodejs.so dist/native/libimager_nodejs.linux.node


###############################################################################
# NODE ENVIRONMENT
###############################################################################
FROM ubuntu:18.04 as node-env

# SETUP
WORKDIR /code
RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install build-essential software-properties-common curl git vim tree
COPY --from=build-rust /code/dist/native/libimager_nodejs.linux.node ./dist/native/libimager_nodejs.linux.node
COPY --from=build-rust /code/dist/native/libimager_nodejs.linux.node ./lib/native/libimager_nodejs.linux.node

# SECURITY & SANITY CHECK
RUN sha1sum dist/native/libimager_nodejs.linux.node > dist/native/libimager_nodejs.linux.node.sha1

# PROJECT DEPENDENCIES
RUN apt-get -y install nodejs npm

# PROJECT - TOP-LEVEL FILES
ADD LICENSE .
ADD README.md .
ADD .babelrc .
ADD jest.config.js .
ADD package-lock.json .
ADD package.json .
ADD tsconfig.json .

# PROJECT - BUILD DEPENDENCIES
RUN npm install

# PROJECT - CODE & OTHER
ADD lib lib
ADD assets/test assets/test
ADD scripts/build-rust.sh scripts/build-rust.sh

# PROJECT - BUILD CODE
ENV PRE_BUILT_LIB_IMAGER_NODEJS=1
RUN npm run build

# PROJECT - TEST
RUN npm test