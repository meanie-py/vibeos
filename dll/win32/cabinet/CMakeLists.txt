
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
# compiles the CRT startup in-cargo (.msvcrtex_crt), and imports msvcrt/kernel32/
# ntdll. wine .a dropped (step 6, c_shim covers it); zlib .a dropped (step 7,
# cabinet-rust now depends on the pure-Rust libz-rs-sys crate for deflate, and
# its FDI inflate is already its own Rust). Version resource (cabinet.rc)
# dropped. The spec2def(... ADD_IMPORTLIB) above STAYS (consumers link the
# libcabinet import lib, independent of this). Only oldnames.a remains cmake.
add_cargo_module(cabinet cabinet-cargo cabinet.dll KIND win32dll
    BODY_CRATE cabinet-rust)
