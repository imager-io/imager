# [GitHub (Prebuilt) Releases](https://github.com/imager-io/imager/releases)

Pretty easy method.

## E.g. MacOS Users

Download & Extract:
```shell
$ http --follow https://github.com/imager-io/imager/releases/download/imager-v0.3/imager-v0.3.1-apple.tar.gz > imager-v0.3.1-apple.tar.gz
$ tar -xvzf imager-v0.3.1-apple.tar.gz
```
Verify:
```
$ ./apple/bin/imager --help
```

## E.g. Linux Users

Download & Extract:
```shell
$ http --follow https://github.com/imager-io/imager/releases/download/imager-v0.3/imager-v0.3.1-linux.tar.gz > imager-v0.3.1-linux.tar.gz
$ tar -xvzf imager-v0.3.1-linux.tar.gz
```

Verify:
```
$ ./linux/bin/imager --help
```


# Imager Lib/CLI - Building From Source

## General Requirements

* make/cargo build tools
* libclang
* C/C++ Compiler (with c++ std lib)
* tar with xz decompression

### Step 1. [Cargo](https://rustup.rs)

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Step 2.

#### For MacOS

```shell
$ brew install llvm
```

#### For Debian-based Linuxes

```shell
$ apt-get install -y build-essential llvm-dev libclang-dev clang openssl pkg-config libssl-dev xz-utils
```

### Step 3. Optional

Report any issues.

## Install

### Step 1. Download

```shell
$ git clone https://github.com/imager-io/imager.git && cd imager
```

### Step 2. Build & Install
> Will install `imager` to `~/.cargo/bin`.

```shell
$ cargo install --path imager --force
```
