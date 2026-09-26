
list(APPEND SOURCE
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/boot.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/check.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/common.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/fat.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/file.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/io.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/lfn.c
    common.c
    fat12.c
    fat16.c
    fat32.c
    vfatlib.c
    vfatlib.h)

# dosfsck.h includes the vendored checker as "dosfstools/x.h".
include_directories(${REACTOS_SOURCE_DIR}/vendor)

add_library(vfatlib ${SOURCE})
add_dependencies(vfatlib xdk)
add_pch(vfatlib vfatlib.h SOURCE)
