
include_directories(
    include
    ${CMAKE_CURRENT_BINARY_DIR}/include
    ${CMAKE_CURRENT_BINARY_DIR}/include/internal
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers
    ${REACTOS_SOURCE_DIR}/sdk/lib/rtl)

add_definitions(
    -D_NTOSKRNL_
    -D_NTSYSTEM_
    -DNTDDI_VERSION=0x05020400
    )

# KD64: Rust on all 3 arches (i386/amd64/arm64). C is no longer needed.
# Verified 2026-04-30: arm64 with Rust kd64 fails identically to arm64 with C kd64
# (same 13,449 bytes serial, same hang at PspUserThreadStartup) — the arm64
# baseline regression is somewhere else, not in kd64.
# (The historical "recursive exception flood on amd64" turned out to be stale —
# fixed silently by CGU=256 + opaque_ptr sweep + RAII guards in 2026 Q1.)

# CC subsystem: Rust (ntoskrnl-rust/src/cc/). The ntoskrnl/cc/*.c sources
# and the NEWCC alternate cache (ntoskrnl/cache/*.c) are gone — the existing
# Rust CC is the only implementation. If the cache subsystem needs work, the
# fix lands in Rust directly, not via porting a parallel C version.

list(APPEND SOURCE
    # config/ — registry CM, replaced by Rust (ntoskrnl-rust/src/config/).
    # cmdata.c kept in C: pure globals, no functions for tree-sitter to extract.
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmalloc.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmapi.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmboot.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmconfig.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmcontrl.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmdata.c
    # config/cm_rust_shims.c — REMOVED 2026-05-07. All 18 wrappers translated
    # to sdk/lib/rust/cm-inlines/src/lib.rs.
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmdelay.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmhook.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmhvlist.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cminit.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmkcbncb.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmlazy.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmmapvw.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmnotify.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmparse.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmquota.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmse.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmsecach.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmsysini.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmvalche.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmwraprs.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/config/ntapi.c
    # dbgk/ — replaced by Rust (ntoskrnl-rust/src/dbgk/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/dbgk/dbgkobj.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/dbgk/dbgkutil.c
    # ex/ — replaced by Rust (ntoskrnl-rust/src/ex/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/atom.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/callback.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/dbgctrl.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/efi.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/event.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/evtpair.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/exintrin.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/fmutex.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/handle.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/harderr.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/hdlsterm.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/init.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/interlocked.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/keyedevt.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/locale.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/lookas.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/mutant.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/profile.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/pushlock.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/resource.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/rundown.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/sem.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/shutdown.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ex/sysinfo.c  # Replaced by Rust ex/sysinfo.rs
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/time.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/timer.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/uuid.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/win32k.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/work.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/xipdisp.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/zone.c
    # ex_rust_shims.c — replaced by Rust (ntkrnl-sys/src/inline/ex.rs + shim_exports.rs)
    # FSRTL subsystem: replaced by Rust (ntoskrnl-rust/src/fsrtl/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/dbcsname.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/fastio.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/faulttol.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/filelock.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/filter.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/filtrctx.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/fsfilter.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/fsrtlpc.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/largemcb.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/mcb.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/name.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/notify.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/oplock.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/pnp.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/stackovf.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/tunnel.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fsrtl/unc.c
    # fstub/ — fully replaced by Rust (ntoskrnl-rust/src/fstub/).
    # i386 boot regression in SMSS \KnownDlls path is a known issue to
    # debug — kernel is left in this state intentionally so the smoke
    # harness reports the regression. Don't re-enable the C versions.
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fstub/disksup.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fstub/fstubex.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fstub/halstub.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/fstub/translate.c
    # inbv/ — replaced by Rust (ntoskrnl-rust/src/inbv/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/inbv/bootanim.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/inbv/inbv.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/inbv/inbvport.c
    # IO Tier 0: adapter, controller, ioevent, iomdl, iotimer, iowork, remlock, symlink → Rust
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/adapter.c
    # IO Tier 2a: bootlog, error, iocomp, irq, ramdisk, util → Rust
    # IO Tier 2b: arcname, iomgr, irp, rawfs → Rust
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/arcname.c  # Replaced by Rust arcname.rs (all functions)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/bootlog.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/controller.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/io_rust_shims.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/device.c  # Testing Rust device.rs
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/deviface.c  # Fixed: byte-string-as-wide-string bugs caused BUGCHECK 0x6B
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/error.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/file.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/iocomp.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/ioevent.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/iofunc.c  # Replaced by Rust iofunc.rs
    # io_tables.c retired 2026-05-08, replaced by Rust iomgr/io_tables.rs.
    # Was the last source file in the active build that pulled <ntoskrnl.h>.
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/iomdl.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/iomgr.c  # amd64/i386 uses Rust; ARM64 uses C (in arm64 section below)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/iorsrce.c  # Replaced by Rust iorsrce.rs
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/iotimer.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/iowork.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/irp.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/irq.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/ramdisk.c  # ARM64 only — see below
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/rawfs.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/remlock.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/symlink.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/util.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/volume.c  # Replaced by Rust volume.rs
    # pnpmgr/ — Rust on all arches
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/arb/arbbus.c>
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/arb/arbdma.c>
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/arb/arbirq.c>
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/arb/arbmem.c>
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/arb/arbport.c>
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/devaction.c>
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/devnode.c>
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/plugplay.c>
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpdma.c  # Replaced by Rust
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpinit.c>
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpirp.c  # Replaced by Rust pnpirp.rs
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpmap.c>
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpmgr.c>
    # pnpnotify.c: Rust version on all arches
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpnotify.c>
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpprop.c  # Replaced by Rust (DeviceProperty, upstream 3cc9aa99292)
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpreport.c>
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnpres.c  # Replaced by Rust pnpres.rs
    #$<$<STREQUAL:${ARCH},i386>:${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnproot.c>
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/pnpmgr/pnputil.c  # Replaced by Rust pnputil.rs
    #${REACTOS_SOURCE_DIR}/ntoskrnl/io/debug.c  # Replaced by Rust debug.rs
    # KD64 generic subsystem: Rust on all 3 arches (kd64/{arch}/kd*.c stays in C).
    #${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/kdapi.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/kdbreak.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/kddata.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/kdinit.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/kdlock.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/kdprint.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/kdtrap.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/ke_rust_shims.c
    # ke/ — replaced by Rust (ntoskrnl-rust/src/ke/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/apc.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/balmgr.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/bug.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/clock.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/config.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/devqueue.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/dpc.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/eventobj.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/except.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/freeze.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/gate.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/gmutex.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/ipi.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/krnlinit.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/mutex.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/processor.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/procobj.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/profobj.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/queue.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/semphobj.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/spinlock.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/thrdobj.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/thrdschd.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/time.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/timerobj.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/wait.c
    # lpc/ — replaced by Rust (ntoskrnl-rust/src/lpc/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/close.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/complete.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/connect.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/create.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/listen.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/port.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/send.c
    # ARM3/ — replaced by Rust (ntoskrnl-rust/src/ARM3/)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/contmem.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/drvmgmt.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/dynamic.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/expool.c (Rust pool crate)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/hypermap.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/iosup.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/kdbg.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/largepag.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/mdlsup.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/mmdbg.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/mminit.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/mmsup.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/ncache.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/pagfault.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/pfnlist.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/pool.c (Rust pool crate)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/procsup.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/section.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/session.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/special.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/sysldr.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/syspte.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/vadnode.c (Rust)
    # mm/ARM3/virtual.c — Rust on all 3 arches (ntoskrnl-rust/src/ARM3/virtual.rs).
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/virtual.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/wslist.cpp (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/zeropage.c (Rust)
    # mm/ — replaced by Rust (ntoskrnl-rust/src/mm/)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/balance.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/freelist.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/marea.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/mmfault.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/mminit.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/pagefile.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/region.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/rmap.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/section.c (Rust)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/shutdown.c (Rust)
    # mm/section_pe.c — replaced by Rust (ntoskrnl-rust/src/mm/section_pe.rs)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/mm/section_pe.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/mm_rust_shims.c
    # mm/mm_sysldr_shims.c eliminated — fully translated to Rust
    # (ntoskrnl-rust/src/ARM3/sysldr.rs).
    # OB subsystem -- replaced by Rust (ntoskrnl-rust/src/ob/)
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/devicemap.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/obdir.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/obhandle.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/obinit.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/oblife.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/oblink.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/obname.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/obref.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/obsdcach.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/obsecure.c
    # ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/obwait.c
    # ob_rust_shims.c — replaced by Rust (ntkrnl-sys/src/inline/ob.rs + shim_exports.rs)
    # po/ — replaced by Rust (ntoskrnl-rust/src/po/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/po/events.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/po/poshtdwn.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/po/povolume.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/po/power.c
    # ps/ — replaced by Rust (ntoskrnl-rust/src/ps/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/apphelp.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/debug.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/job.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/kill.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/process.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/psmgr.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/psnotify.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/query.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/quota.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/security.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/state.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/thread.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/win32.c
    # ps_rust_shims.c — replaced by Rust (ntkrnl-sys/src/inline/ps.rs + shim_exports.rs)
    # rtl/ (kernel) — replaced by Rust (ntoskrnl-rust/src/rtl/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/rtl/libsupp.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/rtl/misc.c
    # se/ — replaced by Rust (ntoskrnl-rust/src/se/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/access.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/accesschk.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/acl.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/audit.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/client.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/objtype.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/priv.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/sd.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/semgr.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/sid.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/sqos.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/srm.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/subject.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/token.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/tokenadj.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/tokencls.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/se/tokenlif.c
    # Split-out minimal trampolines — see docs/engineering/shim-elimination.md.
    # Each has narrowly-scoped headers and survives the future cargo-driven
    # cc-rs build path.
    ${REACTOS_SOURCE_DIR}/ntoskrnl/variadic_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/lto_workaround.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/mm_lock_outlines.c
    # arm64_uart.c retired — ARM64_UART_TRACE / _HEX now in shim_exports.rs.
    # Shared C trampoline for variadic shims — calls into Rust ntprintf crate
    # for format-string parsing + emission. See ntprintf_trampoline.c header.
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ntprintf_trampoline.c
    # vf/ — replaced by Rust (ntoskrnl-rust/src/vf/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/vf/driver.c
    # wmi/ — replaced by Rust (ntoskrnl-rust/src/wmi/)
    #${REACTOS_SOURCE_DIR}/ntoskrnl/wmi/guidobj.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/wmi/smbios.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/wmi/wmi.c
    #${REACTOS_SOURCE_DIR}/ntoskrnl/wmi/wmidrv.c
    # tier1_rust_shims.c — replaced by Rust (ntkrnl-sys/src/inline/tier1.rs + shim_exports.rs)
    )

if(DBG)
    #list(APPEND SOURCE ${REACTOS_SOURCE_DIR}/ntoskrnl/se/debug.c)  # replaced by Rust
endif()

if(REACTOS_TEST_KERNEL)
    add_definitions(-DREACTOS_TEST_KERNEL)
    # All 29 test files (dispatch + 28 NtTestX bodies) translated to Rust at
    # ntoskrnl-rust/src/tests/. Stubs only — bodies need real translation, see
    # tests/mod.rs doc-comment. Per-test TODO list in stubs.rs.
endif()

list(APPEND ASM_SOURCE ${REACTOS_SOURCE_DIR}/ntoskrnl/ex/zw.S)

if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ex/i386/fastinterlck_asm.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ex/i386/ioport.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/ctxswitch.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/trap.s
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/usercall_asm.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/zeropage.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/rtl/i386/prefetch.S)
    list(APPEND SOURCE
        # config/i386/cmhardwr.c — replaced by Rust (ntoskrnl-rust/src/config/i386/cmhardwr.rs)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/config/i386/cmhardwr.c
        # kd64/i386/kdx86.c — replaced by Rust (ntoskrnl-rust/src/kd64/i386/kdx86.rs)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/i386/kdx86.c
        # ke/i386/ — replaced by Rust (ntoskrnl-rust/src/ke/i386/)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/abios.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/cpu.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/context.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/exp.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/freeze.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/irqobj.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/kiinit.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/ldt.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/mtrr.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/patpge.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/thrdini.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/traphdlr.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/usercall.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/v86vdm.c
        # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/i386/page.c (Rust)
        # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/i386/procsup.c (Rust)
        # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/i386/init.c (Rust)
        # ps/i386/ — replaced by Rust (ntoskrnl-rust/src/ps/i386/)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/i386/psctx.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/i386/psldt.c
        # vdm/ — replaced by Rust (ntoskrnl-rust/src/vdm/)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/vdm/vdmmain.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/vdm/vdmexec.c
        # ke/x86x64/cpuinfo.c — new upstream shared x86/x64 CPU-info TU (post-fork).
        # NOT enabled: ke/i386/cpu.c and ke/amd64/cpu.c are Rust here, so a live
        # cpuinfo.c would duplicate symbols the Rust twins already export.
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/x86x64/cpuinfo.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/x86x64/cacheinfo.c  # Replaced by Rust (upstream 83a6af0773b)
    )
elseif(ARCH STREQUAL "amd64")
    list(APPEND ASM_SOURCE
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/boot.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/ctxswitch.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/trap.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/usercall_asm.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/zeropage.S)
    list(APPEND SOURCE
        # config/i386/cmhardwr.c — replaced by Rust (ntoskrnl-rust/src/config/i386/cmhardwr.rs)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/config/i386/cmhardwr.c
        # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/i386/page.c (Rust - shared in i386/page.rs)
        # kd64/amd64/kdx64.c — replaced by Rust (ntoskrnl-rust/src/kd64/amd64/kdx64.rs)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/amd64/kdx64.c
        # ke/amd64/ — replaced by Rust (ntoskrnl-rust/src/ke/amd64/)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/context.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/cpu.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/except.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/freeze.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/interrupt.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/ipi.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/irql.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/kiinit.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/krnlinit.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/spinlock.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/thrdini.c
        # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/amd64/init.c (Rust)
        # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/amd64/procsup.c (Rust)
        # ps/amd64/ — replaced by Rust (ntoskrnl-rust/src/ps/amd64/)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/amd64/psctx.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/stubs.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/traphandler.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/usercall.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/xstate.c
        # ke/x86x64/cpuinfo.c — new upstream shared x86/x64 CPU-info TU (post-fork).
        # NOT enabled: ke/i386/cpu.c and ke/amd64/cpu.c are Rust here, so a live
        # cpuinfo.c would duplicate symbols the Rust twins already export.
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/x86x64/cpuinfo.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/x86x64/cacheinfo.c  # Replaced by Rust (upstream 83a6af0773b)
        )
elseif(ARCH STREQUAL "arm64")
    list(APPEND ASM_SOURCE
        # ke/arm64/stubs_asm.S + boot.S — REMOVED 2026-05-07. Translated to
        # global_asm! in sdk/lib/rust/ke-macros/src/lib.rs (KiSwitchToBootStack,
        # _enable, _disable).
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/arm64/ctxswitch.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/arm64/trap.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/arm64/zwdispatch.S
        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/arm64/usercall_asm.S)
    list(APPEND SOURCE
        # ke/arm64/ — replaced by Rust (ntoskrnl-rust/src/ke/arm64/)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/arm64/stubs.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/arm64/kiinit.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/arm64/thrdini.c
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/arm64/trap.c   # Probing Rust trap.rs (audit's "breaks boot" predates CGU/opaque_ptr)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/ke/arm64/usercall.c
        # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/arm/init.c (Rust)
        # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/arm64/page.c (Rust)
        #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/driver.c  # Testing Rust driver.rs on ARM64
        #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/iomgr.c   # Rust iomgr.rs active on all 3 arches
        #${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/ramdisk.c # Rust ramdisk.rs active on all 3 arches
        )
endif()

if(NOT _WINKD_)
    if(KDBG)
        add_definitions(-DKDBG)
    endif()

    if(ARCH STREQUAL "i386")
        # kd/i386/kdserial.c — replaced by Rust (ntoskrnl-rust/src/i386/kdserial.rs)
        #list(APPEND SOURCE ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/i386/kdserial.c)
        if(KDBG)
            # kdbg/i386/kdb_help.S — replaced by Rust global_asm! (kdbg/kdb_help.rs)
            #list(APPEND ASM_SOURCE ${REACTOS_SOURCE_DIR}/ntoskrnl/kdbg/i386/kdb_help.S)
            #list(APPEND SOURCE ${REACTOS_SOURCE_DIR}/ntoskrnl/kdbg/i386/i386-dis.c)
        endif()
    elseif(ARCH STREQUAL "amd64")
        # amd64 still uses i386 kdserial.c — also replaced by Rust (i386/kdserial.rs)
        #list(APPEND SOURCE ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/i386/kdserial.c)
        if(KDBG)
            # kdbg/amd64/kdb_help.S — replaced by Rust global_asm! (kdbg/kdb_help.rs)
            #list(APPEND ASM_SOURCE ${REACTOS_SOURCE_DIR}/ntoskrnl/kdbg/amd64/kdb_help.S)
            #list(APPEND SOURCE ${REACTOS_SOURCE_DIR}/ntoskrnl/kdbg/i386/i386-dis.c)
        endif()
    elseif(ARCH STREQUAL "arm64")
        # kd/arm64/kdserial.c — replaced by Rust (arm64/kdserial.rs)
        #list(APPEND SOURCE ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/arm64/kdserial.c)
    endif()

    if(KDBG AND NOT ARCH STREQUAL "arm64")
        # kdbg generic — replaced by Rust (ntoskrnl-rust/src/kdbg/)
        # kdbg_rust_shims.c provides variadic KdbPrintf/KdbpPrint that
        # Rust can't define on stable, plus file-private statics from
        # the C kdb_cli.c that Rust references via extern decls.
        list(APPEND SOURCE
            ${REACTOS_SOURCE_DIR}/ntoskrnl/kdbg/kdbg_rust_shims.c)
    endif()

    # kd/ generic — replaced by Rust (ntoskrnl-rust/src/kd/)
    #list(APPEND SOURCE
    #    ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/kdio.c
    #    ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/kdmain.c
    #    ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/kdprompt.c
    #    ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/kdps2kbd.c
    #    ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/kdserial.c
    #    ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/kdterminal.c)

    # kd_rust_shims.c removed — KdComPortInUse is now set directly from
    # kd/kdio.rs via the `__imp_KdComPortInUse` IAT pointer.

else()
    add_definitions(-D_WINKD_)
endif()
