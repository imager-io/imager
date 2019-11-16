# Building From Source
> For imager developers only. Laymen devs should probably stick to NPM releases (since it’s easier).

## Requirements

* `make` build tool 
* `cargo` build tool
* `c/c++` compiler
* `libclang`
* `libc++` - The C++ standard library

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
$ apt install llvm-dev libclang-dev clang
```

Additionally if you don’t have `make` installed:
```shell
$ apt install build-essential
```

## Project

```shell
$ git clone https://github.com/imager-io/imager.git && cd imager/ports/nodejs/
$ npm install
$ ./scripts/build-rust.sh
$ npm run build
$ npm run test # optional
```



# Building VIA Docker (For Linux Users)
> For imager developers only. Laymen devs should probably stick to NPM releases (since it’s easier).

```shell
$ git clone https://github.com/imager-io/imager.git && cd imager/ports/nodejs/
$ npm install
$ ./scripts/docker/build.sh
$ PRE_BUILT_LIB_IMAGER_NODEJS=1 npm run build
$ npm run test # optional
```


# Building Complete API Documentation
```shell
$ git clone https://github.com/imager-io/imager.git && cd imager/ports/nodejs/
$ npm install
$ npm run doc
```
Will be under `./docs`.

# Examples

## Optimize
```typescript
import {ImageBuffer, OptArgs} from "imager-io";

ImageBuffer
    .open("path/to/input/image.jpeg")
    .then(buffer => buffer.opt())
    .then(buffer => buffer.save("path/to/output/image.jpeg"));
```

## Resize & Optimize
> See [API Docs](#Building-Complete-API-Documentation) for further details.
```typescript
import {ImageBuffer, OptArgs} from "imager-io";

ImageBuffer
    .open("path/to/input/image.jpeg")
    .then(buffer => buffer.opt("900x900"))
    .then(buffer => buffer.save("path/to/output/image.jpeg"));
```


