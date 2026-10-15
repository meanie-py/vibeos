include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# CARGO-DRIVEN LINK + FULL Rust flip. smss.exe's link is now driven by cargo
# (smss-cargo → reactos-link ModuleKind::NativeExe) instead of cmake
# add_executable — the fifth of OUR translated spine binaries moved cmake→cargo
# (after lsass + userinit + winlogon + services), and the first NATIVE one.
# All 9 smss .c are flipped to Rust (smss-csrss-rust, built `smss_exe_only`);
# libsmss_csrss_rust.a is now built by cargo (.rust_staticlib) — the cmake
# add_custom_command that used to build it is retired. The lone C
# smss_rust_stub.c was an empty TU (archive-extraction shim) → omitted; cargo
# links the whole staticlib. Keep all .c on disk (never delete the C —
# translation philosophy).
#
# smss.rc is version-only (non-load-bearing) → omitted, like lsass.

# Leaf statics the cargo link reuses (libsmlib the Sm* client lib — real C debt;
# pseh; chkstk) live in sdk/lib, so DEPENDS on their .a FILE paths for
# parallel-build ordering. libnt.a is GONE — the _NtProcessStartup@4 entry is
# now a Rust port in smss-csrss-rust (crate::entry_point → Rust _main). The
# ntdll import lib is generated in-cargo from ntdll.spec.
set(SMSS_REUSED
    ${CMAKE_BINARY_DIR}/sdk/lib/smlib/libsmlib.a)

add_cargo_module(smss smss-cargo smss.exe
    KIND native
    BODY_CRATE smss-csrss-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS ${SMSS_REUSED})

# Three runtime bugs cleared while flipping (2026-05-31), all "local per-file
# re-definition shadows the correct ntdll_sys bindgen" footguns:
#   1. smss.rs re-defined SMP_*_FLAG with SCRAMBLED values (SUBSYSTEM=0x02 vs the
#      real 0x08, ASYNC/AUTOCHK/INVALID_PATH all shifted) while smsubsys.rs used
#      the correct bindgen values → SmpExecuteCommand's `& SUBSYSTEM_FLAG` never
#      matched the 0x08 the caller passed → SmpLoadSubSystem never ran → csrss
#      never launched. Fixed: deleted the local consts (use ntdll_sys::nt).
#   2. smutil.rs had SMP_INVALID_PATH=0x08 (real 0x10). Same fix.
#   3. smsessn.rs hand-rolled a TRUNCATED `struct SMP_SUBSYSTEM { Entry,
#      MuSessionId }` → MuSessionId at offset 8, but the real layout has Event/
#      ProcessHandle/ImageType/PortHandle/SbApiPort/ClientId in between (offset
#      36 on i386). CheckDuplicateMuSessionId read the Event HANDLE (0x3D0=976)
#      as MuSessionId, never matched session 0, SmpSbCreateSession returned
#      OBJECT_NAME_NOT_FOUND → smss (critical) terminated → BUGCHECK 0xc000021a.
#      Fixed: deleted the truncated mirror (use bindgen _SMP_SUBSYSTEM).
# After these, all 3 arches boot to `shell ready` with smss.exe in Rust.
