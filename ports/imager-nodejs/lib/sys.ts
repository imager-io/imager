// import * as data from "./data";


// /**
//  * Lower level interface - Consider this module private.
//  */

// import fs, { Dirent } from "fs";
// import path from "path";

// // @ts-ignore
// import foreign from "../build/Release/web_images_napi.node";


// ///////////////////////////////////////////////////////////////////////////////
// // IMAGE TYPES
// ///////////////////////////////////////////////////////////////////////////////

// /**
//  * A decoded dynamic image.
//  * 
//  * Lower level interface - Consider this type private.
//  */
// export interface Image {
//     readonly type: "Image",
//     readonly ptr: any,
// }

// /**
//  * A decoded grayscale image.
//  * 
//  * Lower level interface - Consider this type private.
//  * 
//  * Primarily used in more advanced image processing pipelines.
//  * Laymen users are probably looking for the 'Image' type.
//  * 
//  * Each pixel is a 32-bit unsigned integer,
//  * primarily used for representing labeled images/regions.
//  * 
//  */
// export interface GrayImageU32 {
//     readonly type: "GrayImageU32",
//     readonly ptr: any,
// }


// ///////////////////////////////////////////////////////////////////////////////
// // IMAGE METHODS - CORE
// ///////////////////////////////////////////////////////////////////////////////

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function open(path: String): Promise<Image> {
//     return foreign.image_open(path);
// }


// /**
//  * Lower level interface - Consider this function private.
//  */
// export function open_with_format(path: String, format: data.ImageFormat): Promise<Image> {
//     return foreign.image_open_with_format(path, format);
// }


// /**
//  * Lower level interface - Consider this function private.
//  */
// export function create(args: data.NewArgs): Promise<Image> {
//     return foreign.image_create(args);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function dimensions(image: Image): Promise<data.Resolution> {
//     return foreign.image_dimensions(image);
// }


// /**
//  * Lower level interface - Consider this function private.
//  */
// export function crop(image: Image, args: data.CropArgs): Promise<Image> {
//     return foreign.image_crop(image, args);
// }


// /**
//  * Lower level interface - Consider this function private.
//  */
// export function color(image: Image): Promise<data.ColorInfo> {
//     return foreign.image_color(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function grayscale(image: Image): Promise<Image> {
//     return foreign.image_grayscale(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function invert(image: Image): Promise<Image> {
//     return foreign.image_invert(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function resize(image: Image, args: data.ResizeArgs): Promise<Image> {
//     return foreign.image_resize(image, args);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function thumbnail(image: Image, args: data.ThumbnailArgs): Promise<Image> {
//     return foreign.image_thumbnail(image, args);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function blur(image: Image, sigma: Number): Promise<Image> {
//     return foreign.image_blur(image, sigma);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function unsharpen(image: Image, sigma: Number, threshold: Number): Promise<Image> {
//     return foreign.image_unsharpen(image, sigma, threshold);
// }

// export type Kernel3x3 = [Number, Number, Number, Number, Number, Number, Number, Number, Number];

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function filter3x3(image: Image, kernel: Kernel3x3): Promise<Image> {
//     return foreign.image_filter3x3(image, kernel);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function adjust_contrast(image: Image, contrast: Number): Promise<Image> {
//     return foreign.image_adjust_contrast(image, contrast);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function brighten(image: Image, value: Number): Promise<Image> {
//     return foreign.image_brighten(image, value);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function huerotate(image: Image, value: Number): Promise<Image> {
//     return foreign.image_huerotate(image, value);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function flipv(image: Image): Promise<Image> {
//     return foreign.image_flipv(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function fliph(image: Image): Promise<Image> {
//     return foreign.image_fliph(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function rotate90(image: Image): Promise<Image> {
//     return foreign.image_rotate90(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function rotate180(image: Image): Promise<Image> {
//     return foreign.image_rotate180(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function rotate270(image: Image): Promise<Image> {
//     return foreign.image_rotate270(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function save(image: Image, path: String): Promise<null> {
//     return foreign.image_save(image, path);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function save_with_format(image: Image, path: String, format: data.ImageFormat): Promise<null> {
//     return foreign.image_save(image, path, format);
// }

// ///////////////////////////////////////////////////////////////////////////////
// // IMAGE METHODS - TRAVERSAL
// ///////////////////////////////////////////////////////////////////////////////

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function map_rgba(
//     image: Image,
//     f: (x: number, y: number, px: Array<number>) => Array<number>
// ): Promise<Image> {
//     return foreign.image_map_rgba(image, f);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function reduce_rgba<T>(
//     image: Image,
//     initial_value: T,
//     f: (accumulator: T, x: number, y: number, px: Array<number>) => T
// ): Promise<T> {
//     return foreign.image_reduce_rgba(image, initial_value, f);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function map_luma(
//     image: Image,
//     f: (x: number, y: number, px: number) => number
// ): Promise<Image> {
//     return foreign.image_map_luma(image, f);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function reduce_luma<T>(
//     image: Image,
//     initial_value: T,
//     f: (accumulator: T, x: number, y: number, px: number) => T
// ): Promise<T> {
//     return foreign.image_reduce_luma(image, initial_value, f);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function grayimage_u32_map(
//     image: GrayImageU32,
//     f: (x: number, y: number, px: number) => number
// ): Promise<GrayImageU32> {
//     return foreign.grayimage_u32_map(image, f);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function grayimage_u32_reduce<T>(
//     image: GrayImageU32,
//     initial_value: T,
//     f: (accumulator: T, x: number, y: number, px: number) => T
// ): Promise<T> {
//     return foreign.grayimage_u32_reduce(image, initial_value, f);
// }

// ///////////////////////////////////////////////////////////////////////////////
// // IMAGE METHODS - CONVERSION
// ///////////////////////////////////////////////////////////////////////////////

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function grayimage_u32_to_image(image: GrayImageU32): Promise<Image> {
//     return foreign.grayimage_u32_to_image(image);
// }


// ///////////////////////////////////////////////////////////////////////////////
// // IMAGE METHODS - ADVANCED PROCESSING
// ///////////////////////////////////////////////////////////////////////////////


// /**
//  * Lower level interface - Consider this function private.
//  */
// export function adaptive_threshold(image: Image, block_radius: Number): Promise<Image> {
//     return foreign.image_adaptive_threshold(image, block_radius);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function equalize_histogram(image: Image): Promise<Image> {
//     return foreign.image_equalize_histogram(image, undefined);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function match_histogram(image: Image, target: Image): Promise<Image> {
//     return foreign.image_match_histogram(image, target);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function otsu_level(image: Image): Promise<Image> {
//     return foreign.image_otsu_level(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function stretch_contrast(image: Image, lower: Number, upper: Number): Promise<Image> {
//     return foreign.image_stretch_contrast(image, lower, upper);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function threshold(image: Image, thresh: Number): Promise<Image> {
//     return foreign.image_threshold(image, thresh);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function distance_transform(image: Image, norm: "L1" | "LInf"): Promise<Image> {
//     return foreign.image_distance_transform(image, norm);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function canny(image: Image, low_threshold: Number, high_threshold: Number): Promise<Image> {
//     return foreign.image_canny(image, low_threshold, high_threshold);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function box_filter(image: Image, x_radius: Number, y_radius: Number): Promise<Image> {
//     return foreign.image_box_filter(image, x_radius, y_radius);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function gaussian_blur_f32(image: Image, sigma: Number): Promise<Image> {
//     return foreign.image_gaussian_blur_f32(image, sigma);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function horizontal_filter(image: Image, kernel: Array<Number>): Promise<Image> {
//     return foreign.image_horizontal_filter(image, kernel);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function median_filter(image: Image, x_radius: Number, y_radius: Number): Promise<Image> {
//     return foreign.image_median_filter(image, x_radius, y_radius);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function separable_filter(image: Image, h_kernel: Array<Number>, v_kernel: Array<Number>): Promise<Image> {
//     return foreign.image_separable_filter(image, h_kernel, v_kernel);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function separable_filter_equal(image: Image, kernel: Array<Number>): Promise<Image> {
//     return foreign.image_separable_filter_equal(image, kernel);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function sharpen3x3(image: Image): Promise<Image> {
//     return foreign.image_sharpen3x3(image);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function sharpen_gaussian(image: Image, sigma: Number, amount: Number): Promise<Image> {
//     return foreign.image_sharpen_gaussian(image, sigma, amount);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function vertical_filter(image: Image, kernel: Array<Number>): Promise<Image> {
//     return foreign.image_vertical_filter(image, kernel);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function morph_close(image: Image, norm: "L1" | "LInf", k: Number): Promise<Image> {
//     return foreign.image_morph_close(image, norm, k);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function morph_dilate(image: Image, norm: "L1" | "LInf", k: Number): Promise<Image> {
//     return foreign.image_morph_dilate(image, norm, k);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function morph_erode(image: Image, norm: "L1" | "LInf", k: Number): Promise<Image> {
//     return foreign.image_morph_erode(image, norm, k);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function morph_open(image: Image, norm: "L1" | "LInf", k: Number): Promise<Image> {
//     return foreign.image_morph_open(image, norm, k);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function gaussian_noise(image: Image, mean: Number, stddev: Number, seed: Number): Promise<Image> {
//     return foreign.image_gaussian_noise(image, mean, stddev, seed);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function salt_and_pepper_noise(image: Image, rate: Number, seed: Number): Promise<Image> {
//     return foreign.image_salt_and_pepper_noise(image, rate, seed);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function connected_components(image: Image, conn: "Four" | "Eight", background: Number): Promise<GrayImageU32> {
//     return foreign.image_connected_components(image, conn, background);
// }

// /**
//  * Lower level interface - Consider this function private.
//  */
// export function shrink_width(image: Image, target_width: Number): Promise<Image> {
//     return foreign.image_shrink_width(image, target_width);
// }


