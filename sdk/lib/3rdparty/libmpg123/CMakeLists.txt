
# This tree mirrors upstream's src/ directory exactly: libmpg123/, common/,
# compat/, include/ and version.h sit where mpg123 puts them, so the sources'
# own relative includes ("../common/debug.h", "../version.h") resolve without
# edits. config.h is ours -- upstream generates it here from config.h.in.
# Do NOT hoist the headers somewhere flatter again; that is what forced the old
# tree to patch include lines.

add_definitions(
    -DOPT_I386
    -DREAL_IS_FLOAT
    -DNOXFERMEM
    -DHAVE_CONFIG_H
    -DEOVERFLOW=75
    -Dstrcasecmp=_stricmp
    -Dstrncasecmp=_strnicmp
    -D_DLL -D__USE_CRTIMP
    -D_CRT_NO_POSIX_ERROR_CODES)

include_directories(BEFORE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_SOURCE_DIR}/libmpg123)

list(APPEND SOURCE
    compat/compat.c
    compat/compat_str.c
    libmpg123/dct64.c
    libmpg123/dct64_i386.c
    libmpg123/equalizer.c
    libmpg123/feature.c
    libmpg123/format.c
    libmpg123/frame.c
    libmpg123/icy.c
    libmpg123/icy2utf8.c
    libmpg123/id3.c
    libmpg123/index.c
    libmpg123/layer1.c
    libmpg123/layer2.c
    libmpg123/layer3.c
    libmpg123/lfs_wrap.c
    libmpg123/libmpg123.c
    libmpg123/ntom.c
    libmpg123/parse.c
    libmpg123/readers.c
    libmpg123/stringbuf.c
    libmpg123/synth.c
    libmpg123/synth_8bit.c
    libmpg123/synth_real.c
    libmpg123/synth_s32.c
    libmpg123/tabinit.c)

list(APPEND PCH_SKIP_SOURCE
    libmpg123/optimize.c)

add_library(libmpg123 STATIC
    ${SOURCE}
    ${PCH_SKIP_SOURCE})
add_pch(libmpg123 precomp.h "${PCH_SKIP_SOURCE}")

target_link_libraries(libmpg123 oldnames)
add_dependencies(libmpg123 psdk)
