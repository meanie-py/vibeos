
spec2def(kdcom.dll kdcom.spec ADD_IMPORTLIB)

# ============================================================================
# kdcom.dll (serial KD transport) — cargo-driven final link
# ============================================================================
# kdcom.c / kddll.c / kdserial.c are translated to Rust
# (sdk/lib/rust/kdcom-serial-rust, kept on disk per the never-delete rule;
# a distinct crate from kdcom-rust, which is the shipping kdrosdbg stub). The
# 8 kdcom.spec exports come from libkdcom_serial_rust.a; the kdcom.dll link is
# driven by cargo (kdcom-serial-cargo's build.rs -> reactos-link) linking it
# with cportlib + libcntpr (CRT) + memcmp + the ntoskrnl/hal import libs.
# Mirrors the kdrosdbg cohab. Only added under _WINKD_ && !GDB (see
# drivers/base/CMakeLists.txt). i386 + amd64.
#
#   kdcom.c kddll.c kdserial.c  -- replaced by Rust (kdcom-serial-rust); kept.
# The version resource: cmake compiles it in this OBJECT library, whose windres
# command the cargo link replays.
add_library(kdcom_res OBJECT kdcom.rc)
target_include_directories(kdcom_res PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
add_cargo_kernel_dll(kdcom kdcom-serial-cargo kdcom.dll BODY_CRATE kdcom-serial-rust
    EXTRA_DEPENDS kdcom_res psdk cportlib libcntpr memcmp)
