
# Every translation unit is Rust now (tree-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(tree_rust tree-rust "" TRUE)
add_executable(tree
    # tree.c  # replaced by Rust (sdk/lib/rust/tree-rust/src/tree/tree.rs)
    rust_cohab_stub.c
    tree.rc)
add_dependencies(tree rust_tree_rust)
target_link_libraries(tree ${tree_rust_LIB})
set_module_type(tree win32cui UNICODE)
target_link_libraries(tree conutils)
add_importlibs(tree msvcrt kernel32)
set_target_properties(tree PROPERTIES SUFFIX ".com")
add_cd_file(TARGET tree DESTINATION reactos/system32 FOR all)
