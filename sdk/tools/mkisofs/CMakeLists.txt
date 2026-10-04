
add_definitions(
    -DDUPLICATES_ONCE
    -DINS_BASE="\\\".\\\""
    -DSCHILY_BUILD
    -DSORTING)

include_directories(
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/include
    reactos
    ${REACTOS_BINARY_DIR}/sdk/include)

# libschily and libsiconv are licensed under CDDL whereas mkisofs is licensed under GPL.
# As such, we cannot mix the library sources with mkisofs sources and have to build them separately.
# Later on, they may still be linked together though.
#
# Reference: http://opensource.stackexchange.com/questions/2094

add_library(libmdigest
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libmdigest/sha3.c)

add_library(libschily
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/stdio/cvmod.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/stdio/dat.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/stdio/fcons.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/stdio/ffileread.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/stdio/fgetline.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/stdio/fileopen.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/stdio/filewrite.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/stdio/flag.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/stdio/niwrite.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/astoll.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/checkerr.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/comerr.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/dirent.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/eaccess.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/error.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/fconv.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/fillbytes.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/fnmatch.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/format.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/getargs.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/geterrno.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/getexecpath.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/gettimeofday.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/gid.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/jsprintf.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/jssnprintf.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/match.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/mem.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/movebytes.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/raisecond.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/saveargs.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/searchinpath.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/seterrno.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/streql.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/strlcat.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/strlcpy.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/uid.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libschily/zerobytes.c)

add_library(libsiconv
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/libsiconv/sic_nls.c)
target_link_libraries(libsiconv libschily)

add_host_tool(mkisofs
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/boot.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/eltorito.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/hash.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/inode.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/isonum.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/joliet.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/match.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/mkisofs.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/multi.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/name.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/rock.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/stream.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/tree.c
    ${REACTOS_SOURCE_DIR}/vendor/schilytools/mkisofs/write.c)
target_link_libraries(mkisofs libmdigest libschily libsiconv)

# libschily implements an own printf function with support for the %r formatter.
# Silence compilers checking for invalid formatting sequences.
target_compile_options(libschily PRIVATE "-Wno-format")
# MATCHES must be used here because on macOS, CMake uses the compiler ID "AppleClang".
if(CMAKE_C_COMPILER_ID MATCHES "Clang$" AND CMAKE_C_COMPILER_VERSION VERSION_GREATER_EQUAL "15")
    # mkisofs uses K&R-style function definitions to support very old compilers.
    # This causes warnings with modern compilers.
    target_compile_options(libmdigest PRIVATE "-Wno-deprecated-non-prototype")
    target_compile_options(libschily PRIVATE "-Wno-deprecated-non-prototype")
    target_compile_options(libsiconv PRIVATE "-Wno-deprecated-non-prototype")
    target_compile_options(mkisofs PRIVATE "-Wno-deprecated-non-prototype")
endif()
