import {ImageBuffer, OptArgs} from "./index";


test('ImageBuffer', async () => {
    return ImageBuffer
        .open("assets/test/1.jpeg")
        .then(buffer => buffer.opt("900x900"))
        .then(buffer => buffer.save("assets/output/test/1.jpeg"));
});