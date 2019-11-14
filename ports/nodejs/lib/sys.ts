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
export interface Buffer {
    readonly type: "Buffer",
    readonly ptr: any,
}

///////////////////////////////////////////////////////////////////////////////
// BUFFER CONSTRUCTION
///////////////////////////////////////////////////////////////////////////////

/**
 * Lower level interface - Consider this function private.
 */
export function buffer_open(path: String): Promise<Buffer> {
    return imager_native.buffer_open(path);
}

///////////////////////////////////////////////////////////////////////////////
// BUFFER METHODS
///////////////////////////////////////////////////////////////////////////////

/**
 * Lower level interface: consider this function private.
 */
export async function buffer_save(buffer: Buffer, path: String): Promise<void> {
    return imager_native
        .buffer_save(buffer, path)
        .then((x: any) => {
            return;
        });
}

/**
 * Lower level interface: consider this function private.
 */
export async function buffer_opt(buffer: Buffer, args?: data.OptArgs | string): Promise<Buffer> {
    let size: string = "full";
    if (args && typeof(args) === 'string') {
        size = args;
    } else if (args && typeof(args) !== 'string' && args.size) {
        size = args.size;
    }
    return imager_native.buffer_opt(buffer, size);
}

