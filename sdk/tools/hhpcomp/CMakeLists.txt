
list(APPEND SOURCE
    hhpcomp.cpp
    hhp_reader.cpp
    utils.cpp
    ${REACTOS_SOURCE_DIR}/vendor/chmc/chmc.c
    ${REACTOS_SOURCE_DIR}/vendor/chmc/err.c
    ${REACTOS_SOURCE_DIR}/vendor/lzx_compress/lz_nonslide.c
    ${REACTOS_SOURCE_DIR}/vendor/lzx_compress/lzx_layer.c
    ../port/mkstemps.c)

# The includes "chmc/chmc.h" and "port/port.h" keep their directory prefix
# (a bare name could collide); these two roots resolve them.
include_directories(${REACTOS_SOURCE_DIR}/vendor ${REACTOS_SOURCE_DIR}/sdk/tools)

# used by lzx_compress
add_definitions(-DNONSLIDE)

add_executable(hhpcomp ${SOURCE})
target_link_libraries(hhpcomp)
