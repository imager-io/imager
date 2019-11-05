#include <webp/encode.h>
#include <webp/decode.h>
#include <webp/types.h>
#include <stdio.h>
#include <assert.h>

void encoder_stub() {
    printf("encoder_stub\n");
}

void webp_config_init(WebPConfig* config) {
    assert(WebPConfigInit(config));
}

void webp_config_preset(WebPConfig* config, WebPPreset preset, float quality) {
    assert(WebPConfigPreset(config, preset, quality));
}

void webp_validate_config(WebPConfig* config) {
    assert(WebPValidateConfig(config));
}



