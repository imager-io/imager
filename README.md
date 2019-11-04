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

Something that isn’t benchmarked here that I've been curious about. Presumably, negatively effecting every image related SAAS venture. Latency and bandwidth overhead. Unless I suppose clients are communicating directly to such services instead of VIA an intermediate backend server.


## Status

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

For now, support will pretty much just correspond to whats popularly available in browsers.

I’m considering `WebP` for the next supported codec.


## Objective

Nothing short of becoming *the industry standard* for image optimization! :)

More concretely. Expose a uniform interface for image transcoding and optimization of popular codecs. Based on off-the-shelf encoders, akin to FFmpeg. With support predominately concerned with lossy codecs.



# Install

```shell
$ git clone https://github.com/imager-io/imager.git && cd imager
$ cargo install --path . --force
```

Note that long term wise I’d like to remove cargo from the installation picture for the CLI tool.

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


# Other Miscellaneous

## Future - Short (to long) Term

In addition to the preexisting CLI tool and in accordance with “becoming the industry standard” mantra.

Port to every major programming language. Idiomatically following the given languages conventions, including dependency management. What I have in mind is not requiring cargo in the installation picture, and distributing self contained libs. 

So for NodeJS using NAPI (which I have some experience with) computationally expensive work should be offloaded from the main thread to the NodeJS managed thread pool. Following on the JS side, control will return immediately with a promise wrapping the eventual result. I’ve written a macro that does essentially this [here](https://github.com/colbyn/web-images-js/blob/9b766b8bdfccb2c429832e461d2be680b61966c9/src/utils.rs#L116).


Although for now the CLI tool can work everywhere, as is common with FFmpeg.




## Future - Long Term

* [Investigation] Internally, how I use VMAF contradicts the official recommendations (from what little documentation or commentary exists). 

* [Feature] [Advanced] Next-gen video codecs! This can work today in supporting browsers VIA HTML5 video APIs. I think the biggest issue will be that:
	1. Backend/frontend developers (outside the video streaming world) aren’t accustomed to fragmented codec support. Since e.g. JPEG is practically supported everywhere.
	2. Laymen users copying images will probably expect the download to be something encoded as JPEG. I think browsers send a redundant http request when ‘copying’ an image. So perhaps the request can be intercepted and made to return a JPEG encoded variant. This way we don’t need to do anything that visually or rather noticeably overrides default browser behavior.


**Note that all future aspirations is predominantly predicated on this project getting popular and/or funding (e.g. VIA Patreon).** So if this project is beneficial at your work, let others know about it! :)


## Regarding Imagers SAAS competitors
> This is something I realized from trying to implement a SAAS model.

In my mind SAAS products don’t make sense when it’s competing with a function. Contrarily, database services or team divisions are commonly split into separate services. Yet just resizing images rarely are, unless such is being pushed by SAAS ventures.

Furthermore using a SAAS component (outside official products from the big cloud platforms) entails SAAS specific requirements, the simplest being authorization, so they know *who to bill* on their end. This means the app will go down if something as simple as authorization fails. So environments from development to production will now have a requirement on such. A function or related with no effects, that simply maps images to images doesn’t have this issue. Another is not only offering a REST API, but an SDK that masks over the aforementioned REST API. Since no developer is going to prefer an HTTP based API over something idiomatic and operable from their language of choice.

Overall SAAS for basic image operations feel contrived. It doest e.g. automate scalability pain points and furthermore proposes a deep and fundamental risk of “what if this service that I’m spending time and money in integration goes down temporarily, or even out of business?“.

If there is a real and legitimate market for such feel free to email me about it. Since going forward the only benefit I see is as a means of funding the open source work.

<hr/>

Copyright 2019 Colbyn Wadman