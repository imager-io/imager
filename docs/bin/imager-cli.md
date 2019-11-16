# Interface
The overall interface is very simple.

Two flags are important, `-i` and `-o`, for input image file(s), and the output directory, respectively. 

#### [required] `-i` or `--input` 
> input image(s)

Images to be optimized. Can be multiple file paths, and also supports file globs.

E.g. An input glob for both JPEG (`.jpeg` and `.jpg`) and PNG files
```
imager -i images/*.jpeg images/*.jpg images/*.png -o output
```  

#### [required] `-o` or `--output`
> output directory

Where to save the optimized images. If the given directory is missing it will be created automatically. 
> For your sake, `imager` will never implicitly override input file paths. 

The image output(s) will always have the same file name as the input image. So e.g. given `imager -i input1.jpeg -o output`. The optimized `input1.jpeg` will be saved under `output/input1.jpeg`. 

#### [optional] `--single`
> Activate single I/O mode.

Changes the interpretation of the input and output arguments. When activated, will save the output image to the specified `output` file path. Obviously this argument is incompatible with multiple `input` images.

Considered to be useful for scripts and other automated settings that deal with single files. Long term wise, I’m thinking of adding an argument called `--save path/to/image.jpeg` that expands to something like `--single --output path/to/image.jpeg`.

#### [optional] `-s` or `--size`
> optional max resolution constraint 

If the given image exceeds the given `size` or resolution. Downsize to given dimension. This will always preserve aspect ratio, and will only downsize images. I.e. it will never scale images to a larger resolution (since this isn’t what people commonly want). 

Example:
```shell
imager -i path/to/image.jpeg -o output -s 1200x1200
```

#### [optional] `-f` or `—format`
> output format. Default is ‘jpeg’.

Currently only `jpeg` is supported, so this parameter isn’t all that useful. 
 
Example
```shell
imager -i path/to/image.jpeg -o output -f jpeg
```

# Examples of `imager`

## Basic

To optimize a single image, given some:
* `path/to/image.jpeg`
* `output/path/`

```shell
imager -i path/to/image.jpeg -o output/path/
```

The result will then be saved to `output/path/image.jpeg`.

## Batch - Basic

To optimize multiple images, given some:
* `path/to/image/dir/*.jpeg`
* `output/path/`

```shell
imager -i path/to/image/dir/*.jpeg -o output/path/
```

The result will then be saved to `output/path/`; see [output flag](https://github.com/imager-io/imager/blob/master/docs/imager-opt.md#imager-opt) for details.

## Batch - Multiple Input Types

To optimize multiple images, given some:
* `path/to/image/dir/*.jpeg`
* `path/to/image/dir/*.jpg`
* `path/to/image/dir/*.png`
* `output/path/`

```shell
imager -i path/to/image/dir/*.jpeg path/to/image/dir/*.jpg path/to/image/dir/*.png -o output/path/
```

The result will then be saved to `output/path/`; see [output flag](https://github.com/imager-io/imager/blob/master/docs/imager-opt.md#imager-opt) for details.

Each output image will have the same file name as the input image. 

## Batch - Recursive Wildcard

To optimize multiple images, given some:
* `path/to/image/dir/**/*.*`
* `output/path/`

```shell
imager -i path/to/image/dir/**/*.* -o output/path/
```

The result will then be saved to `output/path/`; see [output flag](https://github.com/imager-io/imager/blob/master/docs/imager-opt.md#imager-opt) for details.


## Help:
```shell
$ imager --help
```
