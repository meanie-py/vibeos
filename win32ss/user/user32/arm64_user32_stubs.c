/*
 * Stubs for libpng functions that user32 delay-imports on x86/amd64
 * but cannot delay-import on ARM64 (llvm-dlltool limitation).
 * These are used for PNG cursor/icon resource loading — not needed
 * during ARM64 boot.
 */

#ifdef _M_ARM64

#include <stddef.h>

typedef unsigned long png_uint_32;

void * png_create_info_struct(void *png_ptr)
{
    (void)png_ptr;
    return NULL;
}

void * png_create_read_struct(const char *user_png_ver, void *error_ptr,
                              void *error_fn, void *warn_fn)
{
    (void)user_png_ver; (void)error_ptr; (void)error_fn; (void)warn_fn;
    return NULL;
}

void png_destroy_read_struct(void **png_ptr_ptr, void **info_ptr_ptr,
                             void **end_info_ptr_ptr)
{
    (void)png_ptr_ptr; (void)info_ptr_ptr; (void)end_info_ptr_ptr;
}

void png_error(void *png_ptr, const char *error_message)
{
    (void)png_ptr; (void)error_message;
}

void png_free(void *png_ptr, void *ptr)
{
    (void)png_ptr; (void)ptr;
}

png_uint_32 png_get_IHDR(void *png_ptr, void *info_ptr,
                         png_uint_32 *width, png_uint_32 *height,
                         int *bit_depth, int *color_type,
                         int *interlace_method, int *compression_method,
                         int *filter_method)
{
    (void)png_ptr; (void)info_ptr;
    if (width) *width = 0;
    if (height) *height = 0;
    if (bit_depth) *bit_depth = 0;
    if (color_type) *color_type = 0;
    if (interlace_method) *interlace_method = 0;
    if (compression_method) *compression_method = 0;
    if (filter_method) *filter_method = 0;
    return 0;
}

unsigned char png_get_channels(void *png_ptr, void *info_ptr)
{
    (void)png_ptr; (void)info_ptr;
    return 0;
}

unsigned char png_get_color_type(void *png_ptr, void *info_ptr)
{
    (void)png_ptr; (void)info_ptr;
    return 0;
}

void * png_get_io_ptr(void *png_ptr)
{
    (void)png_ptr;
    return NULL;
}

size_t png_get_rowbytes(void *png_ptr, void *info_ptr)
{
    (void)png_ptr; (void)info_ptr;
    return 0;
}

void * png_malloc(void *png_ptr, size_t size)
{
    (void)png_ptr; (void)size;
    return NULL;
}

void png_read_end(void *png_ptr, void *info_ptr)
{
    (void)png_ptr; (void)info_ptr;
}

void png_read_image(void *png_ptr, void **image)
{
    (void)png_ptr; (void)image;
}

void png_read_info(void *png_ptr, void *info_ptr)
{
    (void)png_ptr; (void)info_ptr;
}

void png_read_update_info(void *png_ptr, void *info_ptr)
{
    (void)png_ptr; (void)info_ptr;
}

void png_set_gray_to_rgb(void *png_ptr)
{
    (void)png_ptr;
}

void png_set_palette_to_rgb(void *png_ptr)
{
    (void)png_ptr;
}

void png_set_read_fn(void *png_ptr, void *io_ptr, void *read_data_fn)
{
    (void)png_ptr; (void)io_ptr; (void)read_data_fn;
}

void png_set_rows(void *png_ptr, void *info_ptr, void **row_pointers)
{
    (void)png_ptr; (void)info_ptr; (void)row_pointers;
}

void png_set_scale_16(void *png_ptr)
{
    (void)png_ptr;
}

void png_set_sig_bytes(void *png_ptr, int num_bytes)
{
    (void)png_ptr; (void)num_bytes;
}

int png_sig_cmp(const void *sig, size_t start, size_t num_to_check)
{
    (void)sig; (void)start; (void)num_to_check;
    return -1; /* Not a PNG */
}

#endif /* _M_ARM64 */
