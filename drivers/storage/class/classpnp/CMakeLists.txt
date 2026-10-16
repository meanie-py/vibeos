
spec2def(classpnp.sys classpnp.spec ADD_IMPORTLIB)

# ============================================================================
# classpnp.sys — cargo-driven final link (sdk/lib/rust/classpnp-cargo)
# ============================================================================
# All 18 classpnp C sources (autorun/class/classwmi/clntirp/create/data/debug/
# dictlib/dispatch/guid/history/lock/obsolete/power/retry/srblib/utils/xferpkt)
# are translated to Rust (sdk/lib/rust/classpnp-rust, ~166K lines, kept on disk
# per the never-delete rule). The 60 classpnp.spec exports (Class*) + the PE
# DriverEntry come from libclasspnp_rust.a. Replaces the MODULE link +
# rust_classpnp custom_command. classpnp-cargo's build.rs (reactos-link,
# ModuleKind::KernelDll, user-target body + i386 -sse,-sse2) EXPORTS via
# spec2def(classpnp.spec), imports ntoskrnl + hal, and reuses the C floor
# ntoskrnl_vista / libcntpr / pseh / chkstk (i386+arm64) / cntpr_rust / memcmp.
# BOOT-CRITICAL (disk.sys / cdrom.sys are
# class drivers that import classpnp). The spec2def(... ADD_IMPORTLIB) above
# STAYS (consumers link the libclasspnp import lib, independent of this target).
# classpnp links a fully cargo-native floor — ZERO cmake-built .a. cnt-rust-lib
# (libcntpr's Rust half incl. the Wave MATH-i386 no-SSE softfloat) + pseh
# dummy.c + ntoskrnl-vista-rust + compiler-builtins-mem (memcmp) + clang_rt
# (__chkstk) cover everything; liblibcntpr.a (step 8) and libchkstk.a both
# dropped (cold-link confirms nothing referenced chkstk). EXTRA_DEPENDS is empty.
# The version resource: cmake compiles it in this OBJECT library, whose windres
# command the cargo link replays.
add_library(classpnp_res OBJECT class.rc)
target_include_directories(classpnp_res PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
add_cargo_module(classpnp classpnp-cargo classpnp.sys KIND kerneldll
    BODY_CRATE classpnp-rust DESTINATION reactos/system32/drivers
    EXTRA_DEPENDS classpnp_res psdk)
