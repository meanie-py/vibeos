
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

include_directories(
    concfg
    consrv)

list(APPEND CONSRV_SOURCE
    consrv/alias.c
    consrv/coninput.c
    consrv/conoutput.c
    consrv/console.c
    consrv/frontendctl.c
    consrv/handle.c
    consrv/history.c
    consrv/init.c
    consrv/lineinput.c
    consrv/popup.c
    consrv/settings.c
    consrv/shutdown.c
    consrv/subsysreg.c
    consrv/condrv/coninput.c
    consrv/condrv/conoutput.c
    consrv/condrv/console.c
    consrv/condrv/dummyterm.c
    consrv/condrv/graphics.c
    consrv/condrv/text.c
    consrv/frontends/input.c
    consrv/frontends/terminal.c
    consrv/frontends/wcwidth.c
    consrv/frontends/gui/conwnd.c
    consrv/frontends/gui/fullscreen.c
    consrv/frontends/gui/guiterm.c
    consrv/frontends/gui/guisettings.c
    consrv/frontends/gui/graphics.c
    consrv/frontends/gui/text.c
    consrv/frontends/tui/tuiterm.c
    consrv/frontends/serial/serialterm.c
    # consrv/consrv.rc
    consrv/consrv.h)

#
# Explicitly enable MS extensions to be able to use unnamed (anonymous) nested structs.
#
# FIXME: http://www.cmake.org/Bug/view.php?id=12998
if(MSVC)
    ## NOTE: No need to specify it as we use MSVC :)
    ##target_compile_options(consrv PRIVATE "/Ze")
    #set_source_files_properties(${CONSRV_SOURCE} PROPERTIES COMPILE_OPTIONS "/Ze")
else()
    #target_compile_options(consrv PRIVATE "-fms-extensions")
    set_source_files_properties(${CONSRV_SOURCE} PROPERTIES COMPILE_OPTIONS "-fms-extensions")
endif()

if(ARCH STREQUAL "arm64")
    list(APPEND CONSRV_SOURCE consrv/arm64_delay_stubs.c)
endif()

add_library(consrv ${CONSRV_SOURCE})
target_link_libraries(consrv pseh)
add_dependencies(consrv psdk)
if(ARCH STREQUAL "arm64")
    target_compile_definitions(consrv PRIVATE SERIALTERM_COMPILE)
endif()
add_pch(consrv consrv/consrv.h CONSRV_SOURCE)
#add_object_library(consrv ${CONSRV_SOURCE})
#list(APPEND CONSRV_IMPORT_LIBS)
if(ARCH STREQUAL "arm64")
    # ARM64: llvm-dlltool converts delay-imports to regular imports, which
    # causes csrss to load shell32 + 30 transitive deps at startup,
    # overflowing the user-mode stack. Skip shell32/ole32 (clipboard/OLE
    # not needed during boot). Keep psapi (small, no deep dep chain).
    list(APPEND CONSRV_DELAY_IMPORT_LIBS psapi)
else()
    list(APPEND CONSRV_DELAY_IMPORT_LIBS shell32 ole32 psapi)
endif()
list(APPEND CONSRV_TARGET_LINK_LIBS concfg uuid)
set_module_type(consrv module UNICODE)
