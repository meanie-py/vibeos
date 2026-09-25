
spec2def(user32.dll user32.spec ADD_IMPORTLIB)

include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys
    include
    ${REACTOS_SOURCE_DIR}/sdk/include/wine
    ${REACTOS_SOURCE_DIR}/win32ss/include)

list(APPEND SOURCE
    # controls/appswitch.c — replaced by Rust (user32-rust/src/controls/appswitch.rs); cohab via libuser32_rust.a
    # controls/button.c — replaced by Rust (user32-rust/src/controls/button.rs); cohab via libuser32_rust.a
    # controls/combo.c — replaced by Rust (user32-rust/src/controls/combo.rs); cohab via libuser32_rust.a
    # controls/edit.c — replaced by Rust (user32-rust/src/controls/edit.rs); cohab via libuser32_rust.a
    # controls/ghost.c — replaced by Rust (user32-rust/src/controls/ghost.rs); cohab via libuser32_rust.a
    # controls/icontitle.c — replaced by Rust (user32-rust/src/controls/icontitle.rs); cohab via libuser32_rust.a
    # controls/listbox.c — replaced by Rust (user32-rust/src/controls/listbox.rs); cohab via libuser32_rust.a
    # controls/regcontrol.c — replaced by Rust (user32-rust/src/controls/regcontrol.rs); cohab via libuser32_rust.a
    # controls/scrollbar.c — replaced by Rust (user32-rust/src/controls/scrollbar.rs); cohab via libuser32_rust.a
    # controls/static.c — replaced by Rust (user32-rust/src/controls/static_.rs); cohab via libuser32_rust.a
    # misc/dde.c — replaced by Rust (user32-rust/src/misc/dde.rs); cohab via libuser32_rust.a
    # misc/ddeclient.c — replaced by Rust (user32-rust/src/misc/ddeclient.rs); cohab via libuser32_rust.a
    # misc/ddemisc.c — replaced by Rust (user32-rust/src/misc/ddemisc.rs); cohab via libuser32_rust.a
    # misc/ddeserver.c — replaced by Rust (user32-rust/src/misc/ddeserver.rs); cohab via libuser32_rust.a
    # misc/desktop.c — replaced by Rust (user32-rust/src/misc/desktop.rs); cohab via libuser32_rust.a
    # misc/display.c — replaced by Rust (user32-rust/src/misc/display.rs); cohab via libuser32_rust.a
    # misc/dllmain.c — replaced by Rust (user32-rust/src/misc/dllmain.rs); cohab via libuser32_rust.a
    # misc/exit.c — replaced by Rust (user32-rust/src/misc/exit.rs); cohab via libuser32_rust.a
    # misc/exticon.c — replaced by Rust (user32-rust/src/misc/exticon.rs); cohab via libuser32_rust.a
    # misc/imm.c — replaced by Rust (user32-rust/src/misc/imm.rs); cohab via libuser32_rust.a
    # misc/logon.c — replaced by Rust (user32-rust/src/misc/logon.rs); cohab via libuser32_rust.a
    # misc/misc.c — replaced by Rust (user32-rust/src/misc/misc.rs); cohab via libuser32_rust.a
    # misc/object.c — replaced by Rust (user32-rust/src/misc/object.rs); cohab via libuser32_rust.a
    # misc/resources.c — replaced by Rust (user32-rust/src/misc/resources.rs); cohab via libuser32_rust.a
    # misc/rtlstr.c — replaced by Rust (user32-rust/src/misc/rtlstr.rs); cohab via libuser32_rust.a
    # misc/stubs.c — replaced by Rust (user32-rust/src/misc/stubs.rs); cohab via libuser32_rust.a
    # misc/timer.c — replaced by Rust (user32-rust/src/misc/timer.rs); cohab-linked via libuser32_rust.a
    # misc/usrapihk.c — replaced by Rust (user32-rust/src/misc/usrapihk.rs); cohab via libuser32_rust.a
    # misc/winhelp.c — replaced by Rust (user32-rust/src/misc/winhelp.rs); cohab via libuser32_rust.a
    # misc/winsta.c — replaced by Rust (user32-rust/src/misc/winsta.rs); cohab via libuser32_rust.a
    # windows/accel.c — replaced by Rust (user32-rust/src/windows/accel.rs); cohab via libuser32_rust.a
    # windows/caret.c — replaced by Rust (user32-rust/src/windows/caret.rs); cohab via libuser32_rust.a
    # windows/class.c — replaced by Rust (user32-rust/src/windows/class.rs); cohab via libuser32_rust.a
    # windows/clipboard.c — replaced by Rust (user32-rust/src/windows/clipboard.rs); cohab via libuser32_rust.a
    # windows/cursoricon.c — replaced by Rust (user32-rust/src/windows/cursoricon.rs); cohab via libuser32_rust.a
    # windows/dc.c — replaced by Rust (user32-rust/src/windows/dc.rs); cohab via libuser32_rust.a
    # windows/defwnd.c — replaced by Rust (user32-rust/src/windows/defwnd.rs); cohab via libuser32_rust.a
    # windows/dialog.c — replaced by Rust (user32-rust/src/windows/dialog.rs); cohab via libuser32_rust.a
    # windows/draw.c — replaced by Rust (user32-rust/src/windows/draw.rs); cohab via libuser32_rust.a
    # windows/font.c — replaced by Rust (user32-rust/src/windows/font.rs); cohab via libuser32_rust.a
    # windows/hook.c — replaced by Rust (user32-rust/src/windows/hook.rs); cohab via libuser32_rust.a
    # windows/input.c — replaced by Rust (user32-rust/src/windows/input.rs); cohab via libuser32_rust.a
    # windows/mdi.c — replaced by Rust (user32-rust/src/windows/mdi.rs); cohab via libuser32_rust.a
    # windows/menu.c — replaced by Rust (user32-rust/src/windows/menu.rs); cohab via libuser32_rust.a
    # windows/messagebox.c — replaced by Rust (user32-rust/src/windows/messagebox.rs); cohab via libuser32_rust.a
    # windows/message.c — replaced by Rust (user32-rust/src/windows/message.rs); cohab via libuser32_rust.a
    # windows/nonclient.c — replaced by Rust (user32-rust/src/windows/nonclient.rs); cohab via libuser32_rust.a
    # windows/paint.c — replaced by Rust (user32-rust/src/windows/paint.rs); cohab via libuser32_rust.a
    # windows/prop.c — replaced by Rust (user32-rust/src/windows/prop.rs); cohab via libuser32_rust.a
    # windows/rect.c — replaced by Rust (user32-rust/src/windows/rect.rs); cohab via libuser32_rust.a
    # windows/spy.c — replaced by Rust (user32-rust/src/windows/spy.rs); cohab via libuser32_rust.a
    # windows/text.c — replaced by Rust (user32-rust/src/windows/text.rs); cohab via libuser32_rust.a
    # windows/window.c — replaced by Rust (user32-rust/src/windows/window.rs); cohab via libuser32_rust.a
    # windows/winpos.c — replaced by Rust (user32-rust/src/windows/winpos.rs); cohab via libuser32_rust.a
    # win32ss/user/rtl/image.c — replaced by Rust (user32-rust/src/misc/image.rs); cohab via libuser32_rust.a (shared w/ win32k; usermode #else half)
    # win32ss/user/rtl/text.c — replaced by Rust (user32-rust/src/misc/rtltext.rs); cohab via libuser32_rust.a (shared w/ win32k; usermode #else half)
    ${CMAKE_CURRENT_BINARY_DIR}/user32_stubs.c
    include/user32.h)

if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE
        windows/wndproc_fixup.S)
endif()

add_asm_files(user32_asm ${ASM_SOURCE})

if(ARCH STREQUAL "arm64")
    list(APPEND SOURCE arm64_user32_stubs.c)
endif()

# --- user32-rust cohab staticlib (libuser32_rust.a) -------------------------------------
# Rust translation of the user32 sources, built as a staticlib and cohab-linked into
# user32.dll with the translated .c commented out of SOURCE above (per-file flip, mirrors
# dll/win32/kernel32 / kernel32-rust). User-mode STANDARD per-arch targets; i386 gets
# -sse,-sse2 via sdk/lib/rust/.cargo/config.toml (user-mode no-SSE rule). The crate's
# #[no_mangle] symbols are the sole providers of the commented-out .c functions, so no
# --allow-multiple-definition is needed. NtUser* imports resolve via the win32ksys importlib.
set(USER32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(USER32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-user32-target" user32-rust)
set(USER32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libuser32_rust.a")
if(ARCH STREQUAL "i386")
    set(USER32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(USER32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(USER32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
# GLOB_RECURSE so .rs under src/<subdir>/ also retrigger the libuser32_rust.a rebuild
# (a non-recursive src/*.rs glob silently used a stale archive in kernel32 — same fix here).
rust_crate_sources(USER32_RUST_SRCS "${USER32_RUST_WORKSPACE_DIR}" user32-rust)
add_custom_command(
    OUTPUT ${USER32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USER32_RUST_TARGET_DIR}
        cargo build --release --target ${USER32_RUST_TARGET}
            -p user32-rust
            --manifest-path ${USER32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USER32_RUST_TARGET_DIR}/${USER32_RUST_TARGET}/release/libuser32_rust.a
        ${USER32_RUST_LIB}
    DEPENDS ${USER32_RUST_SRCS}
            ${USER32_RUST_WORKSPACE_DIR}/user32-rust/Cargo.toml
    WORKING_DIRECTORY ${USER32_RUST_WORKSPACE_DIR}
    COMMENT "Building user32-rust (${USER32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_user32 DEPENDS ${USER32_RUST_LIB})

add_library(user32 MODULE
    ${SOURCE}
    ${user32_asm}
    user32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/user32.def)
add_pch(user32 include/user32.h SOURCE)

set_module_type(user32 win32dll UNICODE ENTRYPOINT DllMain 12)
target_link_libraries(user32 user32_vista_static user32_optional_vista user32_wsprintf wine win32ksys ${PSEH_LIB} ${USER32_RUST_LIB})
add_dependencies(user32 asm rust_user32)

if(ARCH STREQUAL "arm64")
    # ARM64: skip libpng delay-import (PNG cursor/icon loading not needed during
    # boot; stubs in arm64_user32_stubs.c). Keep usp10 (text shaping, small deps).
    add_delay_importlibs(user32 usp10)
else()
    add_delay_importlibs(user32 usp10 libpng)
endif()
add_importlibs(user32 gdi32 advapi32 kernel32 ntdll)
add_cd_file(TARGET user32 DESTINATION reactos/system32 FOR all)
