
start_module_group(rostests)

# upstream added a linker script here to give the tests their own section. Our
# llvm-mingw lld rejects it outright ("lld: error: unknown argument: -T") on every
# rostests link, so it is off for Clang. Nothing depends on the section today.
if(NOT MSVC AND NOT CMAKE_C_COMPILER_ID STREQUAL "Clang")
    add_link_options("-Wl,-T,${CMAKE_CURRENT_SOURCE_DIR}/test-section.lds")
endif()

add_definitions(-DHAVE_APITEST)

if(MSVC_IDE)
    add_definitions(-DWINETEST_MSVC_IDE_FORMATTING)
endif()

# Upstream excluded kmtests on GCC-amd64 ("broken with new x64 PSEH") — moot for us:
# llvm-mingw uses USE_DUMMY_PSEH on every arch, so build it everywhere. kmtests is
# outside the ENABLE_ROSTESTS guard so ENABLE_ROSTESTS_KMTESTS can build it alone.
add_subdirectory(kmtests)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    add_compile_options("-Wno-misleading-indentation") # Cause by todo_wine, etc.
endif()

# apitests is outside the ENABLE_ROSTESTS guard for the same reason kmtests is:
# ENABLE_ROSTESTS_APITESTS builds it alone, so the apitest conformance payload
# does not depend on the winetests linking. winetestlib comes with it -- it is a
# one-file static lib (winetestlib.c), and the acpi and crtdll apitests link it.
if(ENABLE_ROSTESTS OR ENABLE_ROSTESTS_APITESTS)
add_subdirectory(winetestlib)
add_subdirectory(apitests)
endif()

if(ENABLE_ROSTESTS)
#add_subdirectory(dibtests)
add_subdirectory(drivers)
#add_subdirectory(dxtest)
#add_subdirectory(regtests)
add_subdirectory(rosautotest)
add_subdirectory(tests)
add_subdirectory(unittests)
add_subdirectory(win32)
add_subdirectory(winetests)
endif()


## Extra files needed for the various tests can be added into the "testdata" subdirectory.
if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/testdata/)
    file(GLOB_RECURSE TESTDATA_FILES RELATIVE ${CMAKE_CURRENT_SOURCE_DIR}/testdata/ ${CMAKE_CURRENT_SOURCE_DIR}/testdata/*)
    foreach(item ${TESTDATA_FILES})
        add_rostests_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/testdata/${item} SUBDIR testdata)
    endforeach(item)
endif()

end_module_group()
