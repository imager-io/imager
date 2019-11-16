# Imager
> Brute force image optimization; optimizes the compression using ML based metrics in a trial ’n error sorta manner.

## About

This is a tool that can competitively optimize (e.g.) extremely noisy, high resolution images; at the expense of increased encoding time and CPU overhead. This is a tradeoff that should be suitable for over 90% of online content, where site performance matters.


## [Benchmarks](https://github.com/colbyn/imager-bench-2019-11-2)

```text
source        : ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇ 39.00M (4 images)
compression.ai: ▇▇▇▇▇▇▇▇ 8.90M
imager        : ▇▇▇▇ 4.20M
```

## Status - Fundamental

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

## Status - Ecosystem

### Supported Operating Systems

| OS     | Current Status |
| ------ | -------- |
| Linux   | ✅ [GOOD] |
| MacOS   | ✅ [GOOD] |
| Windows   | ❌ [UNPRIORITIZED] |

### Supported Languages

| Name | Status | Links | Self Contained (i.e. no sys deps) |
| ------ | -------- | -------- | -------- |
| Rust   | ✅ [GOOD] | [crates](https://crates.io/crates/imager) | NO |
| NodeJS   | ✅ [GOOD] | [npm](https://www.npmjs.com/package/imager-io) - [example](https://git.io/Jeo6e) | YES |

### Supported Dev Tools

| Name | Status |
| ------ | -------- |
| Webpack   | ❎ [IN-PROCESS] |

### Supported Bins

| Name                     | Status    | Description                 |
| -                        | -         | -                           |
| `imager-cli` or `imager` | ✅ [GOOD] | The Imager CLI Interface    |
| `imager-server`   	   | ✅ [GOOD] | The Imager Server Interface |


## Objective
Nothing short of becoming *the industry standard* for image optimization! :)

More concretely. Expose a uniform interface for image transcoding and optimization of popular codecs. Based on off-the-shelf encoders, akin to FFmpeg. With support predominately concerned with lossy codecs.

## Feedback & Feature Requests
Just use the GitHub issue tracker for this project.

## Bugs, Confusion, Performance Issues
Just use the GitHub issue form.

Regarding performance, Imager may be ‘slower’ but it shouldn’t be ‘too slow’, kindly open a new issue if such is the case for you.

## Other Miscellaneous

## Articles

* [Modern Image Optimization for 2020 - Issues, Solutions, and Open Source Solutions](https://medium.com/@colbyn/modern-image-optimization-for-2020-issues-solutions-and-open-source-solutions-543af00e3e51)


<hr/>

Copyright 2019 Colbyn Wadman