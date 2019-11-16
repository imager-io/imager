# Building Complete API Documentation
```shell
$ git clone https://github.com/imager-io/imager.git && cd imager/ports/nodejs/
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


