# cmake -DDEPS=<cargo deps dir> -DSYSROOT=<dir> -DTARGET=<json stem> -P sysroot.cmake
#
# Lays build-std's core + compiler_builtins out as <SYSROOT>/lib/rustlib/<TARGET>/lib,
# the layout `rustc --sysroot` searches. The directory is emptied first: a second
# hashed libcore left beside the first makes every stub compile fail with
# "multiple candidates for `rlib` dependency `core`".
file(REMOVE_RECURSE "${SYSROOT}")
set(_lib "${SYSROOT}/lib/rustlib/${TARGET}/lib")
file(MAKE_DIRECTORY "${_lib}")
foreach(_crate core compiler_builtins)
    file(GLOB _rlibs "${DEPS}/lib${_crate}-*.rlib")
    list(LENGTH _rlibs _n)
    if(NOT _n EQUAL 1)
        message(FATAL_ERROR "spec-stubs sysroot: expected one lib${_crate}-*.rlib in ${DEPS}, found ${_n}: ${_rlibs}")
    endif()
    file(COPY ${_rlibs} DESTINATION "${_lib}")
endforeach()
