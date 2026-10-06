# wowx64.dll — the OS side of BTCpu64: x86-64 guest processes on an arm64 host,
# fully emulated over the amd64 system DLLs by the FEX CPU layer
# libwowx64fex.dll (built out of tree, tools/fex/wowx64fex). Fork B of
# docs/engineering/wow64-x64-guest-design.md. The body is Rust
# (sdk/lib/rust/wowx64-rust); there is no C.
#
# arm64 only: on amd64 an x64 image is native, and i386 cannot host one.
if(ARCH STREQUAL "arm64")
    add_cargo_module(wowx64 wowx64-cargo wowx64.dll KIND win32dll
        BODY_CRATE wowx64-rust)
endif()
