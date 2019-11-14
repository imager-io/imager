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
 * Example:
 * ```typescript
 * ImageBuffer
 *        .open("path/to/input.jpeg")
 *        .then(img => img.opt("900x900"))
 *        .then(img => img.save("test.jpeg"));
 * ```
 */
export class ImageBuffer {
    private handle!: sys.Buffer;

    /**
     * Treat this as private unless you’re using the lower level `sys` module.
     * For construction prefer the `Buffer.open` and related static methods.
     */
    constructor(x: sys.Buffer) {
        console.assert(typeof(x) === 'object');
        console.assert((x as object).hasOwnProperty("type"));
        console.assert((x as object).hasOwnProperty("ptr"));
        console.assert(x.type === "Buffer");
        this.handle = x;
    }

    ///////////////////////////////////////////////////////////////////////////
    // CONSTRUCTION
    ///////////////////////////////////////////////////////////////////////////

    /**
     * 
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
            .buffer_open(path)
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
        return sys.buffer_save(this.handle, path);
    }

    /**
     * Optimize the given image buffer.
     */
    async opt(args?: OptArgs | string): Promise<ImageBuffer> {
        return sys
            .buffer_opt(this.handle, args)
            .then(x => new ImageBuffer(x));
    }
}
