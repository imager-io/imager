import os, { type } from "os";
import * as data from "./data";

const imager_native = function(){
    let platform = os.platform();
    console.assert(
        platform === "darwin" ||
        platform === "linux" ||
        platform === "win32"
    );
    let apple_path = "./native/libimager_nodejs.apple.node";
    let linux_path = "./native/libimager_nodejs.linux.node";
    let windows_path = "./native/libimager_nodejs.windows.node";
    let unknown_platform = (): string => {
        throw "unknown platform";
    };
    let active_path = (platform === "darwin") ? apple_path
        : (platform === "linux") ? linux_path
        : (platform === "win32") ? windows_path
        : unknown_platform();
    return require(active_path);
}();

///////////////////////////////////////////////////////////////////////////////
// DATA TYPES
///////////////////////////////////////////////////////////////////////////////

/**
 * Some encoded image binary.
 * 
 * Lower level interface - Consider this type private.
 */
export interface U8Vec {
    readonly type: "U8Vec",
    readonly ptr: any,
}

///////////////////////////////////////////////////////////////////////////////
// U8VEC CONSTRUCTION
///////////////////////////////////////////////////////////////////////////////

/**
 * Lower level interface - Consider this function private.
 */
export function u8vec_open(path: String): Promise<U8Vec> {
    return imager_native.u8vec_open(path);
}

/**
 * Lower level interface - Consider this function private.
 */
export function u8vec_from_buffer(value: Buffer): Promise<U8Vec> {
    return imager_native.u8vec_from_buffer(value);
}

///////////////////////////////////////////////////////////////////////////////
// U8VEC METHODS
///////////////////////////////////////////////////////////////////////////////

/**
 * Lower level interface: consider this function private.
 */
export async function u8vec_save(buffer: U8Vec, path: String): Promise<void> {
    return imager_native
        .u8vec_save(buffer, path)
        .then((x: any) => {
            return;
        });
}

/**
 * Lower level interface: consider this function private.
 */
export async function u8vec_to_buffer(data: U8Vec): Promise<Buffer> {
    return imager_native.u8vec_to_buffer(data);
}

/**
 * Lower level interface: consider this function private.
 */
export async function u8vec_opt(buffer: U8Vec, args?: data.OptArgs | string): Promise<U8Vec> {
    let size: string = "full";
    if (args && typeof(args) === 'string') {
        size = args;
    } else if (args && typeof(args) !== 'string' && args.size) {
        size = args.size;
    }
    return imager_native.u8vec_opt(buffer, size);
}
