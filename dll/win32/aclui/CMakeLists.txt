
spec2def(aclui.dll aclui.spec)
add_definitions(-DSUPPORT_UXTHEME)

# aclui.dll — Rust cohabitation flip (all 4 TRANSLATABLE src *.c → aclui-rust).
#
# The ACL editor: the "Security" property page every shell object shows, plus the owner-drawn
# check-list control it is built around. Plain Win32 — dialog procs, a custom window class, and
# one COM consumer (IDsObjectPicker, the "Select Users or Groups" dialog).
#
# guid.c stays C ON PURPOSE and is NOT a translation left behind. It is `#include <initguid.h>`
# and nothing else — it says so itself, "NO CODE HERE, THIS IS JUST REQUIRED FOR THE GUID
# DEFINITIONS". The GUID bytes are emitted by the DEFINE_GUID macros as the preprocessor walks
# aclui.h/objsel.h, so there is no function for `rustconv translate` to buy and no way to write
# the file in Rust that is not re-typing the same constants by hand.
#
# Three things the island profile carries for this module (gen_driver_types.sh):
#   * BLOCKLIST_FUNCS_FROM_CRATE — aclui is a dialog module, so nearly every line calls
#     user32/comctl32/advapi32. Emitting the header's fn surface minus the crate's own
#     `#[no_mangle]` set replaces ~200 hand-written prototypes, each of which is a silent
#     corruption risk if it is wrong by one pointer level.
#   * MACRO_SIGNED — control ids (IDC_*/IDD_*) and window messages (WM_*/LVM_*/SB_*) are `int`
#     in the C; bindgen's u32 default turns every use into an E0308.
#   * UNBLOCK=DLGPROC/WNDPROC/LPTHREAD_START_ROUTINE (+PTHREAD_START_ROUTINE, _WNDCLASSW) —
#     Rust compares fn-pointer ABIs NOMINALLY and ntdll-sys spells these with the arch's literal
#     convention, which unifies with neither `extern "system"` spelling. Emitting the island's
#     own copies (the ABI normaliser then spells them "system" on all three arches) is what lets
#     ONE definition of each callback serve every target instead of per-arch `#[cfg]` twins.
set(ACLUI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(ACLUI_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(ACLUI_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(ACLUI_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(ACLUI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-aclui-target" aclui-rust)
set(ACLUI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libaclui_rust.a")
rust_crate_sources(ACLUI_RUST_SRCS "${ACLUI_RUST_WORKSPACE_DIR}" aclui-rust)
add_custom_command(
    OUTPUT ${ACLUI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${ACLUI_RUST_TARGET_DIR}
        cargo build --release --target ${ACLUI_RUST_TARGET}
            -p aclui-rust
            --manifest-path ${ACLUI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${ACLUI_RUST_TARGET_DIR}/${ACLUI_RUST_TARGET}/release/libaclui_rust.a
        ${ACLUI_RUST_LIB}
    DEPENDS ${ACLUI_RUST_SRCS}
            ${ACLUI_RUST_WORKSPACE_DIR}/aclui-rust/Cargo.toml
    WORKING_DIRECTORY ${ACLUI_RUST_WORKSPACE_DIR}
    COMMENT "Building aclui-rust (${ACLUI_RUST_TARGET})"
)
add_custom_target(rust_aclui DEPENDS ${ACLUI_RUST_LIB})

# All 4 translatable src *.c → aclui-rust (src/aclui/*.rs), kept on disk per the
# never-delete rule.
#   aclui.c checklist.c misc.c sidcache.c   → aclui-rust
# list(APPEND SOURCE
#     aclui.c
#     checklist.c
#     misc.c
#     sidcache.c)

# rust_varargs_shim.c is a C-language gap filler, not a translation left in C: misc.c's
# LoadAndFormatString is variadic, and stable Rust can declare a C-variadic but not DEFINE one
# (va_start is legal only inside one). The entry point does the va_start here and tail-calls the
# fixed-arity `_v` twin that holds the translated body. See the file's header comment.
list(APPEND SOURCE
    rust_varargs_shim.c
    ${CMAKE_CURRENT_BINARY_DIR}/aclui_stubs.c)

list(APPEND PCH_SKIP_SOURCE
    guid.c)

add_library(aclui MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    aclui.rc
    ${CMAKE_CURRENT_BINARY_DIR}/aclui.def)

set_module_type(aclui win32dll UNICODE)
# libaclui_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(aclui ${ACLUI_RUST_LIB})
add_dependencies(aclui rust_aclui)
add_importlibs(aclui user32 gdi32 comctl32 ole32 oleaut32 advapi32 uxtheme msvcrt kernel32 ntdll)
# add_pch dropped with the C: guid.c was already PCH_SKIP, and the two TUs left
# (rust_varargs_shim.c and the generated aclui_stubs.c) do not include precomp.h.
add_cd_file(TARGET aclui DESTINATION reactos/system32 FOR all)
