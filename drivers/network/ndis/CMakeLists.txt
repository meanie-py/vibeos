
include_directories(BEFORE include)
spec2def(ndis.sys ndis.spec ADD_IMPORTLIB)

add_definitions(
    -DNDIS_WRAPPER
    -DNDIS51
    -DNDIS51_MINIPORT
    -DNDIS_MINIPORT_DRIVER
    -DNDIS_LEGACY_DRIVER=1
    -DNDIS_LEGACY_MINIPORT=1
    -DNDIS_LEGACY_PROTOCOL=1)

list(APPEND SOURCE
    # ndis/30stubs.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/30stubs.rs)
    # ndis/40stubs.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/40stubs.rs)
    # ndis/50stubs.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/50stubs.rs)
    # ndis/buffer.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/buffer.rs)
    # ndis/cl.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/cl.rs)
    # ndis/cm.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/cm.rs)
    # ndis/co.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/co.rs)
    # ndis/config.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/config.rs)
    # ndis/control.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/control.rs)
    # ndis/efilter.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/efilter.rs)
    # ndis/guid.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/guid/mod.rs, generated: port data)
    # ndis/hardware.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/hardware.rs)
    # ndis/io.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/io.rs)
    # ndis/main.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/main.rs)
    # ndis/memory.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/memory.rs)
    # ndis/miniport.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/miniport.rs)
    # ndis/misc.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/misc.rs)
    # ndis/protocol.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/protocol.rs)
    # ndis/string.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/string.rs)
    # ndis/time.c  # replaced by Rust (sdk/lib/rust/ndis-rust/src/ndis/time.rs)
    rust_cohab_stub.c
    include/ndissys.h)

# Every translation unit is Rust now (ndis-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ndis_rust ndis-rust "" TRUE)
add_library(ndis MODULE
    ${SOURCE}
    ndis.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ndis.def)
add_dependencies(ndis rust_ndis_rust)
target_link_libraries(ndis ${ndis_rust_LIB})

set_module_type(ndis kernelmodedriver)
add_importlibs(ndis ntoskrnl hal)
add_pch(ndis include/ndissys.h SOURCE)
add_cd_file(TARGET ndis DESTINATION reactos/system32/drivers FOR all)
