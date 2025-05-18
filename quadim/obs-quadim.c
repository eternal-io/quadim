#include <obs-module.h>
#include <dlfcn.h>
#include <stdint.h>

OBS_DECLARE_MODULE()
OBS_MODULE_USE_DEFAULT_LOCALE("obs-quadim", "en-US")

typedef int (*quadim_fn)(
    uint8_t *, uint32_t, uint32_t, size_t,
    uint8_t, uint8_t, uint8_t, uint8_t, uint8_t, uint32_t, uint32_t);

static quadim_fn quadim_process;

/// Filter 对象
struct quadim_filter {
    obs_source_t *context;
};

static const char *quadim_filter_get_name(void *unused) {
    UNUSED_PARAMETER(unused);
    return "Quadim Filter";
}

static void *quadim_filter_create(obs_data_t *settings, obs_source_t *context) {
UNUSED_PARAMETER(settings);
    struct quadim_filter *f = bzalloc(sizeof(*f));
    f->context = context;
    return f;
}

static void quadim_filter_destroy(void *data) {
    bfree(data);
}

static obs_properties_t *quadim_filter_properties(void *data) {
UNUSED_PARAMETER(data);
    obs_properties_t *props = obs_properties_create();
    // 可按需添加 ratio/depth/阈值等属性
    return props;
}

static struct obs_source_frame *quadim_filter_video_render(
    void *data, struct obs_source_frame *frame)
{
UNUSED_PARAMETER(data);
    // 原位处理 RGBA
    quadim_process(
        frame->data[0], frame->width, frame->height,
        frame->width * frame->height,
        1, 1, 8, 20, 2, 0, 0);
    return frame;
}

static struct obs_source_info quadim_filter_info = {
     .id            = "obs-quadim-filter",
     .type          = OBS_SOURCE_TYPE_FILTER,
     .output_flags  = OBS_SOURCE_VIDEO,
     .get_name      = quadim_filter_get_name,
     .create        = quadim_filter_create,
     .destroy       = quadim_filter_destroy,
     .get_properties= quadim_filter_properties,
     .filter_video  = quadim_filter_video_render,
 };

bool obs_module_load(void)
{
#if defined(__APPLE__)
    const char *libname = "@loader_path/libquadim_ffi.dylib";
#else
    const char *libname = "libquadim_ffi.so";
#endif

    void *lib = dlopen(libname, RTLD_NOW | RTLD_GLOBAL);
    if (!lib) {
        blog(LOG_ERROR, "Failed to load %s: %s", libname, dlerror());
        return false;
    }

    quadim_process = (quadim_fn)dlsym(lib, "quadim_process_rgba_u8_default");
    if (!quadim_process) {
        blog(LOG_ERROR, "Failed to find symbol quadim_process_rgba_u8_default");
        return false;
    }

    obs_register_source(&quadim_filter_info);
    return true;
}
