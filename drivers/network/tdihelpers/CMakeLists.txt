
# The TDI client helpers (TdiOpenAddressFile, TdiConnect, TdiSend, ...). They were part
# of AFD and netio.sys reuses them, so both drivers link this one library rather than
# each compiling the two TUs. netio builds with _WIN32_WINNT=0x600 and afd with the
# default 0x502; the helpers' object code is identical under both on all three arches
# (llvm-objdump -d -r, 2026-10-17), so one build serves both.
include_directories(
    BEFORE include
    ../afd/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

list(APPEND SOURCE
    # tdi.c  # replaced by Rust (sdk/lib/rust/tdihelpers-rust/src/tdihelpers/tdi.rs)
    # tdiconn.c  # replaced by Rust (sdk/lib/rust/tdihelpers-rust/src/tdihelpers/tdiconn.rs)
    rust_cohab_stub.c
    )

# Every translation unit is Rust now (tdihelpers-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(tdihelpers_rust tdihelpers-rust "" TRUE)
add_library(tdihelpers STATIC ${SOURCE})
add_dependencies(tdihelpers rust_tdihelpers_rust)
target_link_libraries(tdihelpers ${tdihelpers_rust_LIB})
add_dependencies(tdihelpers bugcodes xdk psdk)
