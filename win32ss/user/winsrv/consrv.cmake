
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

include_directories(
    concfg
    consrv)

list(APPEND CONSRV_SOURCE
    # --- Rust cohabitation: consrv core (top + condrv) — DONE ----------
    # Both the consrv top-level AND condrv/ layers are now flipped to Rust.
    # The three gaps that previously blocked this are all closed:
    #   1. The ~40 dropped CON_API Srv* handlers were recovered (81ae68dd).
    #   2. The condrv internal helpers (TEXTMODE_BUFFER_Initialize,
    #      ConDrvFillConsoleOutput, CONSOLE_SCREEN_BUFFER_*, ConioCoordToPointer,
    #      ClearLineBuffer, ConDrvGetActiveScreenBuffer, Convert*, Term*, …)
    #      were #[no_mangle]-promoted with canonical per-fn ABI (cdecl unless
    #      C says NTAPI) and their cross-crate extern decls reconciled to match.
    #   3. The C macros/intrinsics (CONTAINING_RECORD, InitializeObjectAttributes,
    #      _InterlockedExchange) are provided by consrv_rust_shim.c (i386 block).
    # frontends/ (input/terminal/wcwidth + gui/tui/serial) is ALSO flipped now
    # (#36): the FORCEINLINE/macro helpers GetScreenBufferSizeUnits, SmallRectToRect,
    # GetType, GetConsoleInputBufferMode, ConsoleAllocHeap/FreeHeap, CON_SET_OUTPUT_CP,
    # GetSelectionBeginEnd were #[no_mangle]-provided in consrv/mod.rs+conwnd.rs
    # with canonical ABI (cdecl helpers extern "C"; consrv-internal extern "system");
    # InitTTFontCache/RefreshTTFontCache stay cdecl (concfg exports); MonitorFromWindow/
    # ScrollWindowEx are user32 stdcall; s_Colors is a #[no_mangle] static.
    # Only the macro/intrinsic shim (consrv_rust_shim.c) remains C.
    # --- consrv top-level flip → Rust (winsrv-rust/src/consrv/*.rs) ----
    # consrv/alias.c        — replaced by Rust (consrv/alias.rs)
    # consrv/coninput.c     — replaced by Rust (consrv/coninput.rs)
    # consrv/conoutput.c    — replaced by Rust (consrv/conoutput.rs)
    # consrv/console.c      — replaced by Rust (consrv/console.rs)
    # consrv/frontendctl.c  — replaced by Rust (consrv/frontendctl.rs)
    # consrv/handle.c       — replaced by Rust (consrv/handle.rs)
    # consrv/history.c      — replaced by Rust (consrv/history.rs)
    # consrv/init.c         — replaced by Rust (consrv/init.rs)
    # consrv/lineinput.c    — replaced by Rust (consrv/lineinput.rs)
    # consrv/popup.c        — replaced by Rust (consrv/popup.rs)
    # consrv/settings.c     — replaced by Rust (consrv/settings.rs)
    # consrv/shutdown.c     — replaced by Rust (consrv/shutdown.rs)
    # consrv/subsysreg.c    — replaced by Rust (consrv/subsysreg.rs)
    # --- condrv core flip → Rust (winsrv-rust/src/consrv/condrv/*.rs) ----
    # frontends/ stays C and imports condrv ConDrv*/Conio* symbols, so the
    # condrv .rs exports must be #[no_mangle] with matching ABI (42 exports).
    # consrv/condrv/coninput.c   — replaced by Rust (consrv/condrv/coninput.rs)
    # consrv/condrv/conoutput.c  — replaced by Rust (consrv/condrv/conoutput.rs)
    # consrv/condrv/console.c    — replaced by Rust (consrv/condrv/console.rs)
    # consrv/condrv/dummyterm.c  — replaced by Rust (consrv/condrv/dummyterm.rs)
    # consrv/condrv/graphics.c   — replaced by Rust (consrv/condrv/graphics.rs)
    # consrv/condrv/text.c       — replaced by Rust (consrv/condrv/text.rs)
    # --- frontends/ flip → Rust (winsrv-rust/src/consrv/frontends/*.rs) ----
    # Top layer: terminal dispatch (terminal.rs owns FrontEndLoadingMethods +
    # ConSrvTermVtbl), input event conversion (input.rs), wide-char width
    # table (wcwidth.rs, exports mk_wcwidth_cjk cdecl for the shim), plus the
    # gui/tui/serial frontends. GuiLoadFrontEnd is the only registered frontend
    # on i386/amd64 (the live boot path); arm64 also compiles serialterm.
    # consrv/frontends/input.c          — replaced by Rust (frontends/input.rs)
    # consrv/frontends/terminal.c       — replaced by Rust (frontends/terminal.rs)
    # consrv/frontends/wcwidth.c        — replaced by Rust (frontends/wcwidth.rs)
    # consrv/frontends/gui/conwnd.c     — replaced by Rust (frontends/gui/conwnd.rs)
    # consrv/frontends/gui/fullscreen.c — replaced by Rust (frontends/gui/fullscreen.rs)
    # consrv/frontends/gui/guiterm.c    — replaced by Rust (frontends/gui/guiterm.rs)
    # consrv/frontends/gui/guisettings.c— replaced by Rust (frontends/gui/guisettings.rs)
    # consrv/frontends/gui/graphics.c   — replaced by Rust (frontends/gui/graphics.rs)
    # consrv/frontends/gui/text.c       — replaced by Rust (frontends/gui/text.rs)
    # consrv/frontends/tui/tuiterm.c    — replaced by Rust (frontends/tui/tuiterm.rs)
    # consrv/frontends/serial/serialterm.c — replaced by Rust (frontends/serial/serialterm.rs)
    # Provider shim for C macros / MSVC intrinsics that the Rust consrv
    # translation extern-declares (IS_FULL_WIDTH, IsListEmpty, _Interlocked*,
    # IShellLinkW_*/IPersistFile_* COBJMACROS). Self-contained (no SDK headers).
    consrv/consrv_rust_shim.c
    # consrv/consrv.rc
    consrv/consrv.h)

#
# Explicitly enable MS extensions to be able to use unnamed (anonymous) nested structs.
#
# FIXME: http://www.cmake.org/Bug/view.php?id=12998
#target_compile_options(consrv PRIVATE "-fms-extensions")
set_source_files_properties(${CONSRV_SOURCE} PROPERTIES COMPILE_OPTIONS "-fms-extensions")

add_library(consrv ${CONSRV_SOURCE})
target_link_libraries(consrv pseh)
add_dependencies(consrv psdk)
if(ARCH STREQUAL "arm64")
    target_compile_definitions(consrv PRIVATE SERIALTERM_COMPILE)
endif()
add_pch(consrv consrv/consrv.h CONSRV_SOURCE)
# The Rust-shim is intentionally self-contained (no SDK headers) so it can
# define the macro/intrinsic providers without colliding with their macro
# declarations; keep the consrv PCH (and -fms-extensions) out of it.
set_source_files_properties(consrv/consrv_rust_shim.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON COMPILE_OPTIONS "")
#add_object_library(consrv ${CONSRV_SOURCE})
#list(APPEND CONSRV_IMPORT_LIBS)
list(APPEND CONSRV_DELAY_IMPORT_LIBS shell32 ole32 psapi)
list(APPEND CONSRV_TARGET_LINK_LIBS concfg uuid)
set_module_type(consrv module UNICODE)
