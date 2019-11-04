#include <webp/encode.h>
#include <webp/decode.h>
#include <webp/types.h>
#include <stdio.h>

void encoder_stub() {
    printf("encoder_stub\n");
}

void webp_config_init(WebPConfig* config) {
    WebPConfigInit(config);
}

void webp_config_preset(WebPConfig* config, WebPPreset preset, float quality) {
    WebPConfigPreset(config, preset, quality);
}

void webp_validate_config(WebPConfig* config) {
    WebPValidateConfig(config);
}



