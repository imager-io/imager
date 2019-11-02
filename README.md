# Imager
> Brute Force Image Optimization

## About

This is a tool that can competitively optimize (e.g.) extremely noisy, high resolution images; at the expense of increased encoding time and CPU overhead. This is a tradeoff that should be suitable for over 90% of online content, where site performance matters.


## [Benchmarks](https://github.com/colbyn/imager-bench-2019-11-2)

```text
source        : ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇ 39.00M (4 images)
compression.ai: ▇▇▇▇▇▇▇▇ 8.90M
imager        : ▇▇▇▇ 4.20M
```


## Status

Supports any image decodable by `image-rs`. For output targets, currently supports JPEG.


# Install

```shell
$ git clone https://github.com/colbyn/imager.git && cd imager
$ cargo install --path . --force
```


# Example

```shell
$ imager opt -i path/to/images/**/*.jpeg -o assets/output/
```

Also supports resizing:
```shell
$ imager opt -i path/to/images/**/*.jpeg -o assets/output/ -s 1200x1200
```

Help:
```shell
$ imager opt --help
```