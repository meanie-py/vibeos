
include_directories(usersrv)

list(APPEND USERSRV_SOURCE
    # --- Rust cohabitation (winsrv-rust/src/usersrv/) ---------------
    # All 4 usersrv .c are replaced by Rust; libwinsrv_rust.a provides
    # the symbols (pulled via the winsrv.def export roots). The .c stay
    # on disk — never delete the C. usersrv_rust_stub.c keeps the static
    # library a valid (non-empty) TU.
    usersrv/usersrv_rust_stub.c
    # Non-inline providers for ntstrsafe/rtl FORCEINLINE helpers referenced by
    # the Rust usersrv translation (harderror.rs). 2 are C-variadic and cannot
    # be Rust definitions — see memory feedback_degraded_variadic_stubs.
    usersrv/usersrv_rust_strsafe_shim.c
    # usersrv/harderror.c   — replaced by Rust (winsrv-rust/src/usersrv/harderror.rs)
    # usersrv/init.c        — replaced by Rust (winsrv-rust/src/usersrv/init.rs)
    # usersrv/register.c    — replaced by Rust (winsrv-rust/src/usersrv/register.rs)
    # usersrv/shutdown.c    — replaced by Rust (winsrv-rust/src/usersrv/shutdown.rs)
    # usersrv/usersrv.rc
    usersrv/usersrv.h)

add_library(usersrv ${USERSRV_SOURCE})
target_link_libraries(usersrv pseh)
add_dependencies(usersrv xdk)
add_pch(usersrv usersrv/usersrv.h USERSRV_SOURCE)
# The strsafe shim is intentionally self-contained (no SDK headers) so it can
# define the inline-only Rtl* helpers without colliding with the FORCEINLINE
# declarations; keep the usersrv PCH out of it.
set_source_files_properties(usersrv/usersrv_rust_strsafe_shim.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
#add_object_library(usersrv ${USERSRV_SOURCE})
list(APPEND USERSRV_IMPORT_LIBS basesrv)
list(APPEND USERSRV_DELAY_IMPORT_LIBS psapi)
set_module_type(usersrv module UNICODE)
