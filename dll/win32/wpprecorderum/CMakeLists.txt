# wpprecorderum.dll: the user-mode WPP in-flight recorder (wpprecorder.sys's counterpart),
# cargo-linked from sdk/lib/rust/wpprecorderum-cargo. 64-bit only (dll/win32/CMakeLists.txt).
spec2def(wpprecorderum.dll wpprecorderum.spec ADD_IMPORTLIB)
add_cargo_module(wpprecorderum wpprecorderum-cargo wpprecorderum.dll KIND win32dll
    BODY_CRATE wpprecorderum-rust)
