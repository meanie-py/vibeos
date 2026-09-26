
list(APPEND SOURCE
    samplerate.c
    src_linear.c
    src_sinc.c
    src_zoh.c
    precomp.h)

add_library(libsamplerate STATIC ${SOURCE})

# Since 0.2.0 upstream guards its own `#include "config.h"` behind HAVE_CONFIG_H,
# the switch autotools and CMake both define. Without it the sources compile
# without PACKAGE/VERSION/HAVE_STDBOOL_H and fail. Note this is NOT covered by
# precomp.h: add_pch is a no-op unless the PCH option is on, which it is not by
# default, so nothing force-includes precomp.h in an ordinary build.
target_compile_definitions(libsamplerate PRIVATE HAVE_CONFIG_H)

add_pch(libsamplerate precomp.h SOURCE)
add_dependencies(libsamplerate psdk)
target_include_directories(libsamplerate INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
