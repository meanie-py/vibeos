
option(ENABLE_ROSAPPS "Enable building the ReactOS extra applications" OFF)
option(ENABLE_ROSTESTS "Enable building the ReactOS test suite" OFF)
# Conformance-ratchet lane: kmtests build clean on llvm-mingw, and this lets
# conformance builds ship them without dragging the rest of the corpus in.
# The winetests are NOT the grind this comment used to claim: measured
# 2026-09-20, 126 of 128 suites compile on i386, the exceptions being
# user32_winetest (llvm-rc cannot parse resource.rc's ACCELERATORS block) and
# ucrtbase_winetest (its CMakeLists asks for -lgcc, a GCC-ism llvm-mingw has
# no library for). Build them with `vibeos-cli build --arch i386 --rostests`,
# which is reconciled OFF again by the next plain build.
option(ENABLE_ROSTESTS_KMTESTS "Enable building only kmtests from the ReactOS test suite" OFF)
# Same bargain for the USER-MODE suites, and for the same reason: the apitests
# are the oracle the upstream-sync lane needs (the 2026-09 merge alone shipped
# 43 new ones, JobObject among them), and all 63 of them build clean, while the
# full ENABLE_ROSTESTS switch also drags in winetests/unittests/win32 -- where 4
# targets do NOT build, one of them an llvm-rc gap in an ACCELERATORS block that
# is a toolchain limitation rather than anything we can patch. Without this
# option the apitest conformance payload could never produce media.
option(ENABLE_ROSTESTS_APITESTS "Enable building only apitests from the ReactOS test suite" OFF)
option(ENABLE_WALLPAPERS "Integrate wallpapers into the built ISOs" OFF)

if(ENABLE_ROSAPPS)
    add_subdirectory(rosapps)
endif()

if(ENABLE_ROSTESTS OR ENABLE_ROSTESTS_KMTESTS OR ENABLE_ROSTESTS_APITESTS)
    add_subdirectory(rostests)
endif()

if(ENABLE_WALLPAPERS)
    add_subdirectory(wallpapers)
endif()

if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/AHK_Tests/)
    add_subdirectory(AHK_Tests)
endif()

## Extra files for the Boot/Live CD. You need to put them under "[boot|live]cd_extras"
## using the same layout as you want them to be in the CD.
## If you change the content of any of the following folders later on,
## run "cmake ." before compiling.
if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/bootcd_extras/)
    file(GLOB_RECURSE EXTRA_FILES RELATIVE ${CMAKE_CURRENT_SOURCE_DIR}/bootcd_extras/ ${CMAKE_CURRENT_SOURCE_DIR}/bootcd_extras/*)
    foreach(item ${EXTRA_FILES})
        add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/bootcd_extras/${item} DESTINATION extras NO_CAB NAME_ON_CD ${item} FOR bootcd regtest)
    endforeach(item)
endif()
if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/livecd_extras/)
    file(GLOB_RECURSE EXTRA_FILES RELATIVE ${CMAKE_CURRENT_SOURCE_DIR}/livecd_extras/ ${CMAKE_CURRENT_SOURCE_DIR}/livecd_extras/*)
    foreach(item ${EXTRA_FILES})
        add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/livecd_extras/${item} DESTINATION extras NO_CAB NAME_ON_CD ${item} FOR livecd)
    endforeach(item)
endif()

## Extra files which you need existing on the hard drive after the installation.
if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/3rdparty/)
    file(GLOB_RECURSE EXTRA_FILES RELATIVE ${CMAKE_CURRENT_SOURCE_DIR}/3rdparty/ ${CMAKE_CURRENT_SOURCE_DIR}/3rdparty/*)
    foreach(item ${EXTRA_FILES})
        add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/3rdparty/${item} DESTINATION reactos/3rdParty NAME_ON_CD ${item} FOR bootcd)
    endforeach(item)
endif()

# Add font file DroidSansFallback.ttf for LiveCD
# See also boot/bootdata/packages/reactos.dff.in
if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/optional/DroidSansFallback.ttf)
    add_cd_file(FILE "${CMAKE_CURRENT_SOURCE_DIR}/optional/DroidSansFallback.ttf" DESTINATION reactos/Fonts FOR livecd)
endif()
