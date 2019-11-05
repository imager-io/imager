#include <webp/encode.h>
#include <webp/decode.h>
#include <webp/types.h>
#include <imageio/image_enc.h>
#include <imageio/image_dec.h>
#include <stdlib.h>
#include <stdio.h>
#include <assert.h>

void webp_picture_from(const uint8_t* data, size_t data_size, WebPPicture* const pic) {
    WebPImageReader reader;
    // reader = WebPGuessImageReader(data, data_size);
    // assert(reader(data, data_size, pic, 1, NULL));
    // assert(pic);
    // free((void*)data);
}