
list(APPEND HAL_UP_SOURCE
    generic/buildtype.c
    generic/spinlock.c)
    # generic/up.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/up.rs)

add_library(lib_hal_up OBJECT ${HAL_UP_SOURCE})
add_dependencies(lib_hal_up bugcodes xdk)
