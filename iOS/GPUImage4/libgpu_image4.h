//
//  libgpuimage4.h
//
//  Created by Jinlei Li on 2022/10/20.
//

#ifndef libgpu_image4_h
#define libgpu_image4_h

#include <stdint.h>

struct wgpu_canvas;

struct ios_view_obj {
    void *view;
    // CAMetalLayer
    void *metal_layer;
    int maximum_frames;
    void (*callback_to_swift)(int32_t arg);
};

enum filter_type {
    AsciiArt,
    CrossHatch,
    EdgeDetection,
};

struct wgpu_canvas *create_wgpu_canvas(struct ios_view_obj obj);

void set_filter(struct wgpu_canvas *canvas, enum filter_type ty, int opaque_background_color, float param);
void change_filter_param(struct wgpu_canvas *canvas,  float param);
void set_external_texture(struct wgpu_canvas *canvas, void *raw, const char *tex_key, int width, int height);
void remove_texture(struct wgpu_canvas *canvas, const char *tex_key);

void enter_frame(struct wgpu_canvas *canvas, const char *tex_key);

#endif /* libgpu_image4_h */
