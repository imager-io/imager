import {ImageBuffer, OptArgs} from "./index";
import * as fs from "fs";


test('ImageBuffer: #1 File IO', async () => {
    return ImageBuffer
        .open("assets/test/1.jpeg")
        .then(buffer => buffer.opt("900x900"))
        .then(buffer => buffer.save("assets/output/test/1-1.jpeg"));
});

test('ImageBuffer: #2 Buffer IO', async () => {
    const input_path = "assets/test/1.jpeg";
    const output_path = "assets/output/test/1-2.jpeg";
    if (!fs.existsSync("assets/output/test")){
        fs.mkdirSync("assets/output");
        fs.mkdirSync("assets/output/test");
    }
    let data: Buffer = fs.readFileSync(input_path);
    return ImageBuffer
        .from_buffer(data)
        .then(x => x.opt())
        .then(x => x.to_buffer())
        .then(x => {
            fs.writeFileSync(output_path, x);
        });
});