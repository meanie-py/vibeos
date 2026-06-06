
add_definitions(
    -D_NTHALDLL_
    -D_NTHAL_)

include_directories(
    generic
    ${REACTOS_SOURCE_DIR}/ntoskrnl/include)

# ============================================================================
# hal.dll (arm64) is CARGO-DRIVEN (sdk/lib/rust/hal-cargo) — link block below.
# ============================================================================
# The arm64 HAL is pure-Rust stubs (sdk/lib/rust/hal-rust/src/arm64/halstubs.rs;
# generic/halstubs.c is fully retired). The runtime hal.dll link, and hal-rust
# itself, are now built by the cargo link (add_cargo_kernel_dll → reactos-link,
# ModuleKind::KernelDll), the same cutover as halx86 (i386/amd64). The former
# `rust_hal_arm64` custom_command (built hal-rust on the standard cargo target)
# + `add_library(hal MODULE)` are retired.
#
# Unlike halx86 the arm64 link is minimal: the authoritative
# build-arm64/build.ninja hal.dll line is just libhal_rust.a + the ntoskrnl
# import lib (self-hosted by the cargo link) + libmemcmp.a — no asm, no
# resource, none of the x86 cntpr/setjmp/arbiter/pseh leaf libs.

# spec2def(... ADD_IMPORTLIB) stays — ntoskrnl/ntkrnlmp import hal's exports.
spec2def(hal.dll ../hal.spec ADD_IMPORTLIB)

add_cargo_kernel_dll(hal hal-cargo hal.dll
    BODY_CRATE hal-rust DESTINATION reactos/system32
    EXTRA_DEPENDS
        memcmp
        ${CMAKE_CURRENT_SOURCE_DIR}/../hal.spec
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ntoskrnl.spec)
