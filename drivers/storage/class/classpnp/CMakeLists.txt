
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
# Version resource (class.rc) dropped. BOOT-CRITICAL (disk.sys / cdrom.sys are
# class drivers that import classpnp). The spec2def(... ADD_IMPORTLIB) above
# STAYS (consumers link the libclasspnp import lib, independent of this target).
# cnt-rust-lib (libcntpr's Rust half) + pseh dummy.c + memcmp + the
# classpnp-referenced ntoskrnl_vista subset are now cargo-native (rust_lib /
# c_shims / compiler-builtins-mem / ntoskrnl-vista-rust). step 8: liblibcntpr.a
# is also gone — cnt-rust-lib is the sole libcntpr provider (classpnp pulled no
# C residue from it; links clean ×3). So rust_cnt / ${PSEH_LIB} / memcmp /
# ntoskrnl_vista / libcntpr all drop from EXTRA_DEPENDS; only chkstk stays.
add_cargo_module(classpnp classpnp-cargo classpnp.sys KIND kerneldll
    BODY_CRATE classpnp-rust DESTINATION reactos/system32/drivers
    EXTRA_DEPENDS chkstk)
