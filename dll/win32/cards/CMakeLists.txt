
# Possible definitions: CARDSTYLE_DEFAULT or CARDSTYLE_BAVARIAN
add_definitions(-DCARDSTYLE_DEFAULT)

spec2def(cards.dll cards.spec)

list(APPEND SOURCE
    # cards.c  # replaced by Rust (sdk/lib/rust/cards-rust/src/cards/cards.rs)
    rust_cohab_stub.c
    cards.rc
    ${CMAKE_CURRENT_BINARY_DIR}/cards.def)

# Every translation unit is Rust now (cards-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(cards_rust cards-rust "" TRUE)
add_library(cards MODULE ${SOURCE})
add_dependencies(cards rust_cards_rust)
target_link_libraries(cards ${cards_rust_LIB})
set_module_type(cards win32dll UNICODE)
add_importlibs(cards user32 gdi32 msvcrt kernel32)
add_cd_file(TARGET cards DESTINATION reactos/system32 FOR all)
