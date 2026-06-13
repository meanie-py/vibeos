
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# Keep the cmake import library (spec2def ADD_IMPORTLIB → libcsrsrv.a): other
# cmake-linked modules (csrss.exe, winsrv.dll) still `add_importlibs(... csrsrv)`
# against it. csrsrv.dll's own EXPORTS are generated separately by the cargo
# link (csrsrv-cargo .spec2def from this same .spec).
spec2def(csrsrv.dll csrsrv.spec ADD_IMPORTLIB)

# CARGO-DRIVEN LINK + FULL Rust flip. csrsrv.dll's link is now driven by cargo
# (csrsrv-cargo → reactos-link ModuleKind::Win32Dll, subsystem native:5.01)
# instead of cmake add_library(MODULE) — the sixth of OUR translated spine
# binaries moved cmake→cargo (after lsass + userinit + winlogon + services +
# smss), and the first native DLL. All 7 csrsrv .c are flipped to Rust
# (smss-csrss-rust, kitchen-sink default features); libsmss_csrss_rust.a is now
# built by cargo (.rust_staticlib) — the cmake add_custom_command that used to
# build it is retired. The lone C csrsrv_rust_stub.c was an empty TU →
# omitted; cargo links the whole staticlib. csrsrv.rc is version-only →
# omitted. Keep all .c on disk (never delete the C — translation philosophy).

# The lone cmake .a the cargo link reuses is libsmlib (the C Sm* client —
# _SmConnectToSm@16 et al., genuinely referenced; ~942 LOC translation debt),
# so DEPENDS on its .a FILE path for parallel-build ordering. libpseh.a was
# dropped (probe: zero PSEH-runtime refs under DUMMY_PSEH). The ntdll import lib
# is generated in-cargo from ntdll.spec; the csrsrv.def is spec2def'd in-cargo.
set(CSRSRV_REUSED
    ${CMAKE_BINARY_DIR}/sdk/lib/smlib/libsmlib.a)

add_cargo_module(csrsrv csrsrv-cargo csrsrv.dll
    KIND win32dll
    BODY_CRATE smss-csrss-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS ${CSRSRV_REUSED})
