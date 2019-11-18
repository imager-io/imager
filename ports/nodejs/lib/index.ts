import * as sys from "./sys";
import { type } from "os";
import {OptArgs} from "./data";

export * from "./data";


/**
 * Some kinda image binary. It’s not ‘decoded’ so we don’t know e.g. it’s resolution.
 * 
 * This is the most direct interface to Imagers optimization functionality.
 * 
 * This is also the return type for “optimized images”. 
 * 
 * 
 * Example of File IO:
 * ```typescript
 * ImageBuffer
 *        .open("path/to/input.jpeg")
 *        .then(img => img.opt("900x900"))
 *        .then(img => img.save("test.jpeg"));
 * ```
 * 
 * Example of Buffer IO:
 * ```typescript
 * const input_path = "path/to/input/image.jpeg";
 * const output_path = "test.jpeg";
 * let data: Buffer = fs.readFileSync(input_path);
 * return ImageBuffer
 *     .from_buffer(data)
 *     .then(x => x.opt())
 *     .then(x => x.to_buffer())
 *     .then(x => {
 *         fs.writeFileSync(output_path, x);
 *         console.log("done");
 *     });
 * ```
 */
export class ImageBuffer {
    private data!: sys.U8Vec;

    /**
     * Treat this as private unless you’re using the lower level `sys` module.
     * For construction prefer the `Buffer.open` and related static methods.
     */
    constructor(x: sys.U8Vec) {
        console.assert(typeof(x) === 'object');
        console.assert((x as object).hasOwnProperty("type"));
        console.assert((x as object).hasOwnProperty("ptr"));
        console.assert(x.type === "U8Vec");
        this.data = x;
    }

    ///////////////////////////////////////////////////////////////////////////
    // CONSTRUCTION
    ///////////////////////////////////////////////////////////////////////////

    /**
     * Open the image located at the path specified.
     * 
     * ```typescript
     * let buffer: Promise<buffer> = Buffer
     *      .open("test.jpeg")
     *      .then(x => {
     *          console.log("loaded buffer");
     *          return x;
     *      });
     * ```
     */
    static async open(path: string): Promise<ImageBuffer> {
        console.assert(typeof(path) === 'string');
        return sys
            .u8vec_open(path)
            .then(x => new ImageBuffer(x));
    }

    /**
     * Load from a NodeJS Buffer object.
     * 
     * ```typescript
     * const input_path = "path/to/input/image.jpeg";
     * const output_path = "test.jpeg";
     * let data: Buffer = fs.readFileSync(input_path);
     * return ImageBuffer
     *     .from_buffer(data)
     *     .then(x => x.opt())
     *     .then(x => x.to_buffer())
     *     .then(x => {
     *         fs.writeFileSync(output_path, x);
     *         console.log("done");
     *     });
     * ```
     */
    static async from_buffer(buffer: Buffer): Promise<ImageBuffer> {
        return sys
            .u8vec_from_buffer(buffer)
            .then(x => new ImageBuffer(x));
    }

    ///////////////////////////////////////////////////////////////////////////
    // METHODS
    ///////////////////////////////////////////////////////////////////////////

    /**
     * Save the buffer at the path specified.
     */
    async save(path: string): Promise<void> {
        console.assert(typeof(path) === 'string');
        return sys.u8vec_save(this.data, path);
    }

    /**
     * Cast to a vanilla NodeJS Buffer.
     */
    async to_buffer(): Promise<Buffer> {
        return sys.u8vec_to_buffer(this.data);
    }

    /**
     * Optimize the given image buffer.
     */
    async opt(args?: OptArgs | string): Promise<ImageBuffer> {
        return sys
            .u8vec_opt(this.data, args)
            .then(x => new ImageBuffer(x));
    }
}
