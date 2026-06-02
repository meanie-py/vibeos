/*
 * Cohabitation stub for the Rust concfg (font.rs + settings.rs) — task #45.
 *
 * font.c + settings.c are commented out of SOURCE in concfg/CMakeLists.txt;
 * their symbols — CodePageToCharSet, CreateConsoleFontEx/2, CreateConsoleFont,
 * GetFontCellSize, IsValidConsoleFont/2, InitTTFontCache, ClearTTFontCache,
 * RefreshTTFontCache, FindCachedTTFont, and the ConCfg* settings API
 * (ConCfgOpenUserSettings, ConCfgReadUserSettings, ConCfgWriteUserSettings,
 * ConCfgInitDefaultSettings, ConCfgGetDefaultSettings) — are now provided by
 * libwinsrv_rust.a (winsrv-rust/src/concfg/), which is linked into winsrv.dll
 * (the consumer of concfg). font.c was translated by Opus 4.8 after the
 * rustconv scanner learned SAL annotations + the CALLBACK calling convention +
 * body-internal #if 0 / #ifndef FONT_CACHE_PRESENT preprocessor blocks.
 *
 * CMake's add_library() needs at least one TU it always compiles; keep the .c
 * files on disk (translation philosophy: never delete the C).
 */

typedef int concfg_rust_cohab_stub_t;
