
spec2def(mlang.dll mlang.spec ADD_IMPORTLIB)

# ============================================================================
# mlang.dll — cargo-driven final link (sdk/lib/rust/mlang-cargo)
# ============================================================================
# mlang.c is translated to Rust (sdk/lib/rust/mlang-rust, kept on disk per the
# never-delete rule). Replaces the win32dll MODULE link + rust_mlang
# custom_command. mlang-cargo's build.rs (reactos-link, ModuleKind::Win32Dll)
# compiles rust_cohab_stub.c (wine debug no-ops + IsEqualGUID/min/list_* + vtbl
# dispatch) via c_shims, EXPORTS via spec2def(mlang.spec) + spec_stubs (the
# spec2def-generated mlang_stubs.c for the `stub`-only entries), reuses the
# uuid/wine/oldnames/msvcrtex statics, and imports oleaut32 (was a cmake delay
# import; collapses to a regular import on llvm-mingw) + gdi32/advapi32/msvcrt/
# kernel32/ntdll. Version resource (mlang.rc) dropped. The spec2def(...
# ADD_IMPORTLIB) above STAYS (consumers link the libmlang import lib).
add_cargo_module(mlang mlang-cargo mlang.dll KIND win32dll
    BODY_CRATE mlang-rust
    EXTRA_DEPENDS uuid msvcrtex)
