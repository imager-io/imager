# Imager

## Features

### Brute Force Image Optimization

> Optimizes the compression using ML based metrics in a trial ’n error sorta manner.

This is a tool that can competitively optimize (e.g.) extremely noisy, high resolution images; at the expense of increased encoding time and CPU overhead. This is a tradeoff that should be suitable for over 90% of online content, where site performance matters.

It's pretty easy too.

<small>Using the CLI interface:</small>
```shell
$ imager -i path/to/input/images/*.jpeg -o output/
```

<small>Using the HTTP server:</small>
```shell
$ imager-server --address 127.0.0.1:3000
```

```shell
$ http 127.0.0.1:3000/opt < path/to/input/image.jpeg > path/to/output/image.jpeg
```

<small>Using the JavaScript non-blocking API:</small>

```javascript
const {ImageBuffer} = require("imager-io");
ImageBuffer
	.open("source-image.jpeg")
	.then(buffer => buffer.opt())
	.then(buffer => buffer.save("result.jpeg"))
	.then(() => console.log("done"));
```


## [Compression Benchmarks](https://github.com/colbyn/imager-bench-2019-11-2)

```text
source        : ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇ 39.00M (4 images)
kraken.io     : ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇ 24M
jpegmini.com  : ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇ 16M
compression.ai: ▇▇▇▇▇▇▇▇ 8.90M
imager        : ▇▇▇▇ 4.20M
```

### Supported **Input** Image Formats

| Format | Decoding |
| ------ | -------- |
| PNG    | All supported color types |
| JPEG   | Baseline and progressive |
| GIF    | Yes |
| BMP    | Yes |
| ICO    | Yes |
| TIFF   | Baseline(no fax support) + LZW + PackBits |
| WebP   | Lossy(Luma channel only) |
| PNM    | PBM, PGM, PPM, standard PAM |

Essentially supports any image decodable by [image-rs](https://github.com/image-rs/image.git).

### Supported **Output** Image Formats

> These are your optimization targets (for lack of a better name). It’s a bit higher level, since e.g. rate control is automatically handled.

| Format | Encoding |
| ------ | -------- |
| JPEG   | progressive |

For now, support will pretty much just correspond to whats popularly available in browsers. I’m considering `WebP` for the next supported codec.


#### [Download & Install](https://github.com/imager-io/imager/releases)

Prebuilt binaries can be found [here](https://github.com/imager-io/imager/releases).

## Objective
Nothing short of becoming *the industry standard* for image optimization! :)

More concretely. Expose a uniform interface for image transcoding and optimization of popular codecs. Based on off-the-shelf encoders, akin to FFmpeg. With support predominately concerned with lossy codecs.

## Feedback, Requests, Bugs, Confusion & Performance Issues
Just use the GitHub issue tracker for this project.

## Other Miscellaneous

### Articles

* [Modern Image Optimization for 2020 - Issues, Solutions, and Open Source Solutions](https://medium.com/@colbyn/modern-image-optimization-for-2020-issues-solutions-and-open-source-solutions-543af00e3e51)


<hr/>

Copyright 2019 Colbyn Wadman