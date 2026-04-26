
# IO Subsystem Status (updated 2026-03-21)

## Smoke Test Status
- **amd64: 4/4 PASS**
- **i386: 4/4 PASS** (fixed this session — extern "C" vs extern "system" ABI)
- **ARM64: BUGCHECK 0x6B** (Codex investigating now)

## IOMGR — Active on all arches (23 files)

| File | Lines | Status |
|------|-------|--------|
| adapter.rs | 114 | Active all arches |
| arcname.rs | 1,742 | **DONE** — re-enabled, amd64+i386 pass |
| bootlog.rs | 824 | Active all arches |
| controller.rs | 294 | Active all arches |
| debug.rs | 222 | Active all arches |
| device.rs | 3,545 | Active all arches |
| deviface.rs | 3,603 | Active all arches |
| driver.rs | 4,258 | Active all arches |
| error.rs | 1,457 | Active all arches |
| file.rs | 3,341 | Active all arches |
| iocomp.rs | 1,229 | Active all arches |
| ioevent.rs | 182 | Active all arches |
| iofunc.rs | 3,823 | **DONE** — completed via Codex (9 missing functions) |
| iomdl.rs | 347 | Active all arches |
| iorsrce.rs | 2,697 | Active all arches |
| iotimer.rs | 377 | Active all arches |
| iowork.rs | 186 | Active all arches |
| irp.rs | 3,639 | Active all arches |
| irq.rs | 456 | Active all arches |
| rawfs.rs | 2,415 | Active all arches |
| remlock.rs | 527 | Active all arches |
| symlink.rs | 261 | Active all arches |
| util.rs | 612 | Active all arches |
| volume.rs | 2,771 | Active all arches |

## IOMGR — Blocked on ARM64 baseline (2 files)

| File | Lines | Status | Diagnosis |
|------|-------|--------|-----------|
| iomgr.rs | 1,145 | amd64/i386 only | ARM64 excluded — blocked on ARM64 baseline regression |
| ramdisk.rs | 405 | amd64/i386 only | ARM64 excluded — blocked on ARM64 baseline regression |

## PNPMGR — Active on amd64/arm64 (17 files), i386 uses C

| File | Lines | Status |
|------|-------|--------|
| devaction.rs | 5,012 | **DONE** — PnP hang fixed (Codex: DEVICE_RELATIONS flexible-array bug) |
| arb/arbbus.rs | 213 | **DONE** — enabled on amd64/arm64, amd64+i386 pass |
| arb/arbdma.rs | 201 | **DONE** — enabled on amd64/arm64 |
| arb/arbirq.rs | 202 | **DONE** — enabled on amd64/arm64 |
| arb/arbmem.rs | 199 | **DONE** — enabled on amd64/arm64 |
| arb/arbport.rs | 203 | **DONE** — enabled on amd64/arm64 |
| pnpinit.rs | 1,029 | amd64/arm64 Rust, i386 C |
| pnpmgr.rs | 3,717 | amd64/arm64 Rust, i386 C |
| plugplay.rs | 3,475 | amd64/arm64 Rust, i386 C |
| pnproot.rs | 3,109 | amd64/arm64 Rust, i386 C |
| pnpmap.rs | 1,558 | amd64/arm64 Rust, i386 C |
| pnpnotify.rs | 990 | amd64/arm64 Rust, i386 C |
| devnode.rs | 741 | amd64/arm64 Rust, i386 C |
| pnpreport.rs | 612 | amd64/arm64 Rust, i386 C |
| pnpirp.rs | 612 | amd64/arm64 Rust, i386 C |
| pnputil.rs | 381 | amd64/arm64 Rust, i386 C |
| pnpdma.rs | 187 | amd64/arm64 Rust, i386 C |

## What's left (priority order)

1. ~~iofunc.rs~~ — **DONE** (Codex translated 9 missing functions)
2. ~~arcname.rs~~ — **DONE** (re-enabled, amd64+i386 pass)
3. ~~devaction.rs~~ — **DONE** (Codex fixed DEVICE_RELATIONS flexible-array bug)
4. ~~arb/*.rs~~ — **DONE** (enabled on amd64/arm64, no devaction dependency)
5. ~~i386 baseline~~ — **DONE** (Codex fixed extern "C" vs extern "system" ABI in device.rs/iomgr.rs/arcname.rs)
6. **ARM64 baseline** — Codex investigating BUGCHECK 0x6B (PsCreateSystemThread STATUS_INSUFFICIENT_RESOURCES)
7. **ramdisk.rs on ARM64** — blocked on ARM64 baseline fix
8. **iomgr.rs on ARM64** — blocked on ARM64 baseline fix
9. **pnpmgr on i386** — needs extern "C" → extern "system" audit across 12 files (~36 blocks). See `~pnpmgr-i386-audit.md`. Good task for Gemini.

## Key Fixes This Session
- **154 wrong constants** fixed across 61 files via verify_constants.py tool
- **iofunc.rs** completed — 9 missing Nt* functions translated by Codex
- **devaction.rs PnP hang** — root cause: DEVICE_RELATIONS.Objects flexible-array indexing UB (Codex)
- **i386 boot** — root cause: extern "C" vs extern "system" ABI mismatches across device.rs, iomgr.rs, arcname.rs (Codex)
- **arcname.rs** — re-enabled on all arches
- **arb/*.rs** — enabled on amd64/arm64
