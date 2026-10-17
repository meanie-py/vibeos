remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(mi.dll mi.spec ADD_IMPORTLIB)

# mi.dll, the Management Infrastructure client, is Rust written fresh over wbemprox
# (sdk/lib/rust/mi-rust); there is no C. cargo links it: sdk/lib/rust/mi-cargo.
add_cargo_module(mi mi-cargo mi.dll
    KIND win32dll
    BODY_CRATE mi-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS psdk)
add_dependencies(mi psdk)
