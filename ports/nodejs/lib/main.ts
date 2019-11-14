import {ImageBuffer, OptArgs} from "./index";

async function main() {
    ImageBuffer
        .open("assets/samples/small/low/2yV-pyOxnPw300.jpeg")
        .then(buffer => buffer.opt("900x900"))
        .then(buffer => buffer.save("test.jpeg"));
}
main();
