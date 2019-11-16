# CLI Interface

## `imager-server`

To start the server, just run:

```
imager-server --address 127.0.0.1:3030
```

### Help
```
imager-server --help
```

# HTTP Interface - Endpoints

As a general rule, image operations VIA the CLI interface and HTTP interface are symmetric.

I.e what’s available VIA the CLI interface should be included in the HTTP interface, and vise versa. So e.g. `imager opt --size 900x900` is `/opt?size=900x900`. But with the obvious exception that the image I/O payloads are VIA the HTTP body. Nothing interacts with the file system, and likewise batching currently isn’t possible, except VIA multiple HTTP requests.

## Endpoint `/opt`

### URL Query Parameters

#### [optional] `/opt?size`
> optional max resolution constraint 

If the given image exceeds the given `size` or resolution. Downsize to given dimension. This will always preserve aspect ratio, and will only downsize images. I.e. it will never scale images to a larger resolution (since this isn’t what people commonly want). 

#### [optional] `/opt?format`
> output format. Default is ‘jpeg’.

Currently only `jpeg` is supported, so this parameter isn’t all that useful. 


# Examples

## Start Server

```shell
imager-server --address 127.0.0.1:3030
```

## Client
> This example is using [HTTPie](https://httpie.org).

Given some:
* `path/to/input/image.jpeg`
* `path/to/output` for `path/to/output/image.jpeg`

```shell
http 127.0.0.1:3030/opt < path/to/input/image.jpeg > path/to/output/image.jpeg
```
