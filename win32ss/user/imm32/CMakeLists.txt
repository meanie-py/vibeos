
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/wine
    ${REACTOS_SOURCE_DIR}/win32ss/include)

spec2def(imm32.dll imm32.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # candidate.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/candidate.rs)
    # compstr.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/compstr.rs)
    # ctf.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/ctf.rs)
    # guideline.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/guideline.rs)
    # ime.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/ime.rs)
    # imemenu.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/imemenu.rs)
    # imepro.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/imepro.rs)
    # imm.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/imm.rs)
    # install.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/install.rs)
    # keymsg.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/keymsg.rs)
    # regword.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/regword.rs)
    # softkbd.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/softkbd.rs)
    # utils.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/utils.rs)
    # win3.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/win3.rs)
    # win3send.c  # replaced by Rust (sdk/lib/rust/imm32-rust/src/imm32/win3send.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/imm32_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/imm32.def)

file(GLOB imm32_rc_deps ${CMAKE_CURRENT_SOURCE_DIR}/res/*.*)
add_rc_deps(imm32.rc ${imm32_rc_deps})

# Every translation unit is Rust now (imm32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(imm32_rust imm32-rust "" TRUE)
add_library(imm32 MODULE ${SOURCE} imm32.rc)
add_dependencies(imm32 rust_imm32_rust)
target_link_libraries(imm32 ${imm32_rust_LIB})
set_module_type(imm32 win32dll UNICODE ENTRYPOINT ImmDllInitialize 12)
target_link_libraries(imm32 wine win32ksys uuid)
add_importlibs(imm32 advapi32 user32 gdi32 kernel32 ntdll)
add_cd_file(TARGET imm32 DESTINATION reactos/system32 FOR all)

option(IMM_WIN3_SUPPORT "IMM Win 3.x support" ON)
if(IMM_WIN3_SUPPORT)
    target_compile_definitions(imm32 PRIVATE IMM_WIN3_SUPPORT)
endif()
