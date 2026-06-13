
spec2def(cabinet.dll cabinet.spec ADD_IMPORTLIB)

# ============================================================================
# cabinet.dll — cargo-driven final link (sdk/lib/rust/cabinet-cargo)
# ============================================================================
# All four cabinet C sources (cabinet_main/fci/fdi/stubs) are translated to Rust
# (sdk/lib/rust/cabinet-rust, kept on disk per the never-delete rule), including
# the hand-ported LZX/QTM/MSZIP decompression engine + FDICopy. Replaces the
# win32dll MODULE link + rust_cabinet custom_command. cabinet-cargo's build.rs
# (reactos-link, ModuleKind::Win32Dll) compiles rust_cohab_stub.c (wine debug
# no-ops + HRESULT_FROM_WIN32) via c_shims, EXPORTS via spec2def(cabinet.spec),
# reuses the wine/zlib/msvcrtex/oldnames statics, and imports msvcrt/kernel32/
# ntdll. Version resource (cabinet.rc) dropped. The spec2def(... ADD_IMPORTLIB)
# above STAYS (consumers link the libcabinet import lib, independent of this).
add_cargo_module(cabinet cabinet-cargo cabinet.dll KIND win32dll
    BODY_CRATE cabinet-rust
    EXTRA_DEPENDS wine zlib msvcrtex)
