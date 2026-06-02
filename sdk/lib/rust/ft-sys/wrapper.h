/*
 * ft-sys bindgen wrapper.
 *
 * Replicates the exact FreeType include set that win32ss/gdi/ntgdi/freetype.c
 * uses (the public FT header set plus the two FT_INTERNAL headers it reaches
 * into), so bindgen emits byte-exact FT and TT type layouts and the full
 * function surface. FT2_BUILD_LIBRARY is required for the internal headers.
 */
#define FT2_BUILD_LIBRARY
#include <ft2build.h>
#include FT_FREETYPE_H
#include FT_GLYPH_H
#include FT_BITMAP_H
#include FT_OUTLINE_H
#include FT_TYPE1_TABLES_H
#include FT_TRUETYPE_TABLES_H
#include FT_TRUETYPE_TAGS_H
#include FT_TRUETYPE_IDS_H
#include FT_TRIGONOMETRY_H
#include FT_WINFONTS_H
#include FT_SFNT_NAMES_H
#include FT_SYNTHESIS_H
#include <freetype/internal/internal.h>
#include FT_INTERNAL_TRUETYPE_TYPES_H
