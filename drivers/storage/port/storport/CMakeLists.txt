
# Every .c is Rust (storport-rust, re-ported with `rustconv port` for the 2026-10-15 upstream
# sync). spec2def still exports the StorPort* API (satisfied by Rust #[no_mangle]) + the
# import lib storahci links. NOT boot-loaded in smoke -> link-clean x3 bar.

spec2def(storport.sys storport.spec ADD_IMPORTLIB)
include_directories(${REACTOS_SOURCE_DIR}/sdk/lib/drivers/sptilib)


# rust_varargs_shim.c — va_start/va_arg trampolines for the cdecl-varargs exports
# (StorPortDebugPrint/ExtendedFunction/Notification). Stable Rust cannot DEFINE a
# C-variadic fn; the logic stays in storport-rust's *_v twins. Delete once c_variadic
# stabilizes (rust-lang#44930).
# C bodies replaced by Rust (storport-rust), kept on disk as the translation reference:
# commented entries of the list the target expands, so the translator can prove their owner.
list(APPEND SOURCE
    #fdo.c
    #ioctl.c
    #miniport.c
    #misc.c
    #pdo.c
    #srb.c
    #storport.c
    #stubs.c
    )
# Every translation unit is Rust now (storport-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(storport_rust storport-rust "" TRUE)
add_library(storport MODULE
    ${SOURCE}
    # guid.c  # replaced by Rust (sdk/lib/rust/storport-rust/src/storport/guid.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    storport.rc
    ${CMAKE_CURRENT_BINARY_DIR}/storport.def)
add_dependencies(storport rust_storport_rust)
target_link_libraries(storport ${storport_rust_LIB})
set_module_type(storport kernelmodedriver)
add_importlibs(storport ntoskrnl hal)
add_cd_file(TARGET storport DESTINATION reactos/system32/drivers NO_CAB FOR all)
