
list(APPEND SOURCE
    # spider.cpp  # replaced by Rust (sdk/lib/rust/spider-rust/src/spider/spider.rs)
    # spigame.cpp  # replaced by Rust (sdk/lib/rust/spider-rust/src/spider/spigame.rs)
    rust_cohab_stub.c
    spider.h)

add_rc_deps(rsrc.rc ${CMAKE_CURRENT_SOURCE_DIR}/spider.ico)
# Every translation unit is Rust now (spider-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(spider_rust spider-rust "" TRUE)
add_executable(spider ${SOURCE} rsrc.rc)
add_dependencies(spider rust_spider_rust)
target_link_libraries(spider ${spider_rust_LIB})
target_link_libraries(spider cardlib)
target_include_directories(spider PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
add_pch(spider spider.h SOURCE)
set_module_type(spider win32gui UNICODE)
add_importlibs(spider advapi32 comctl32 shell32 user32 gdi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET spider DESTINATION reactos/system32 FOR all)
