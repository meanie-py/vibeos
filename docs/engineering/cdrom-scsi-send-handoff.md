# cdrom-rust SCSI-send bring-up — Codex handoff

**Date:** 2026-06-12  **Branch:** `cdrom-port`  **Worktree:** `/home/curi/Projects/os-playground/vibeos-wt-cdrom`
**Arch under debug:** amd64 (i386/arm64 not yet retested this round)

cdrom.sys is the only **KMDF** driver in the Rust-cohab fleet. It is translated C→Rust
(`sdk/lib/rust/cdrom-rust/`) and cohab-linked into the cmake-built `cdrom.sys`. Every `Wdf*`
call routes through the `kmdf-sys` dispatch table (`sdk/lib/rust/kmdf-sys/src/dispatch.rs`,
populated by `WdfVersionBind`).

## TL;DR — where we are

cdrom went from **never loading** → **`DriverEvtDeviceAdd` 100% success** → **device-start
(`DeviceEvtSelfManagedIoInit`) running its full init chain** up to the first real SCSI command.

> **UPDATE 2026-06-12 (end of session): the SCSI-send blocker was ROOT-CAUSED and a one-line fix
> applied (UNCOMMITTED) — `__SEH2_ProbeAndLockPages` (ddk_shims.rs:436) returned `1`, but callers
> treat the return as `0=success / non-zero=exception` (`if status != 0 { break }`). So the MDL
> lock "succeeded" yet `DeviceSendSrbSynchronously` read it as an exception and bailed BEFORE the
> send loop → `st=1, SrbStatus=PENDING, dtl=0`. Changed the shim to return `0`. NEEDS VERIFICATION:
> build+smoke and read `[SMIO]`/`[INQ]` — INQUIRY should now execute. If it does, the read path /
> CdRomCount++ / cdfs mount are the next milestones; expect more bug-class instances (see patterns
> below). The `[SRB]` send-loop trace never fired (confirming the break was before the loop), so the
> earlier IRP-send hypotheses below are likely moot — keep them only if INQUIRY still fails.**
>
> **VERIFICATION 2026-06-12:** warm amd64 build succeeded and `./vibeos-cli smoke --arch amd64`
> reached the raw SRB send loop. Serial proof:
> `[SRB] pre-send ioctl=1b0011 mj=f ...`,
> `[SRB] post-send st=0 srbst=81 dtl=24`,
> `[INQ] send1 st=0 srbst=81 dtl=24 req=24 addlen=1f`,
> `[SMIO] 4b-inquiry st=0`.
> So the SCSI-send blocker is fixed. The next crash was a separate MCN init translation bug:
> `BUGCHECK 0x1E (0xc0000005, 0xfffff80004a10c44, ...)`. The screenshot gave the exact
> `cdrom.sys` base (`fffff800049fd000`), mapping the PC to PE VA `0x23c44` inside
> `DeviceInitializeMcn`: `WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes, 1 as *const ...)`
> compiled into an absolute load from address `0x19`. Fixed by passing
> `&CDROM_REQUEST_CONTEXT_TYPE_INFO`.
>
> **VERIFICATION 2 2026-06-12:** warm amd64 build succeeded and the next smoke advanced through
> MCN, MMC, ZPODD, READ CAPACITY, and `CdRomCount++`:
> `[SMIO] 5a-mcd st=0`,
> `[SMIO] 5b-mmc st=0`,
> `[SMIO] 5c-zpodd st=0`,
> `[SMIO] 5d-pre-readcapacity`,
> `[SRB] post-send st=0 srbst=81 dtl=8`,
> `[SMIO] 8-CdRomCount++`,
> `[CDARC] matched signature, CdRomCount=1 EnabledDisks=1`.
> The remaining failure is downstream:
> `PsLocateSystemDll` -> `BUGCHECK 0x6B (0xc000003a, 2, 0, 0)`, i.e. system-root/path lookup,
> not SCSI send, MCN, or read capacity. Do not reopen the SRB-send investigation unless `[INQ]`
> regresses.

The blocker was the raw-SRB SCSI send primitive:

> **`DeviceSendSrbSynchronously` (common.rs:539) returned `st=1` (STATUS_WAIT_1) with
> `SrbStatus=0` (SRB_STATUS_PENDING) and `DataTransferLength=0` — the function bailed at the MDL
> probe-and-lock before the send loop (see UPDATE above).**

Because INQUIRY (`DeviceCacheDeviceInquiryData`, init.rs:1214) fails with
`STATUS_DEVICE_PROTOCOL_ERROR` (0xc0000186), the rest of `DeviceEvtSelfManagedIoInit` is gated
off, `CdRomCount` is never incremented (step 8, init.rs ~410), the ARC `\Device\CdRom0` name is
never created, `\SystemRoot` never resolves, and the boot ends in **BUGCHECK 0x6B** at
`PsLocateSystemDll`. **The 0x6B is a downstream victim — do NOT hunt the read path or the kernel
ARC code; fix the SCSI send.**

Key discriminator: **the IOCTL path works, the raw-SRB path does not.**
- `WdfIoTargetSendIoctlSynchronously` (IOCTL_SCSI_GET_ADDRESS, IOCTL_STORAGE_QUERY_PROPERTY) →
  `[SMIO] 1a-scsiaddr done`, `[SMIO] 1b-descriptors done st=0`. **Works.**
- raw SRB via `WdfIoTargetFormatRequestForInternalIoctlOthers` (idx 191) + manual IRP stack
  override + `WdfRequestSend`. **Broken** (this doc).

## Fixed bug — `DeviceSendSrbSynchronously` (common.rs:539)

Send chain:
```
DeviceSendSrbSynchronously (common.rs:539)
  → builds SRB, creates WDFREQUEST against deviceExtension->IoTarget
  → IoAllocateMdl + __SEH2_ProbeAndLockPages on the data buffer
  → loop:
      WdfRequestReuse
      WdfIoTargetFormatRequestForInternalIoctlOthers(IoTarget, request, ioctlCode, ...)  // idx 191
      // manual WDM SRB attach (common.rs ~743-752):
      nextStack = IoGetNextIrpStackLocation(irp);
      nextStack->MajorFunction = IRP_MJ_SCSI;            // 0x0F
      nextStack->Parameters.DeviceIoControl.IoControlCode = ioctlCode;   // union…
      nextStack->Parameters.Scsi.Srb = Srb;                              // …overlaps, Srb wins
      DeviceSendRequestSynchronously(Device, request, 1)   // common.rs:504
        → RequestSend(..., WDF_REQUEST_SEND_OPTION_SYNCHRONOUS, ...)   // common.rs:3445
          → WdfRequestSend(request, IoTarget, &options)
          → status = WdfRequestGetStatus(request)
```

Original symptom captured by the `[SRB]`/`[INQ]` DbgPrint traces:
`[INQ] send1 st=1 srbst=0 dtl=0 req=24 addlen=0`.
After the `__SEH2_ProbeAndLockPages` return-contract fix, the verified symptom is gone:
`[INQ] send1 st=0 srbst=81 dtl=24 req=24 addlen=1f`.

### Ruled out this session
- **ioctlCode used before assignment** — FIXED (commit 121452a2bd9). The format call used
  `ioctlCode` while it was still 0; it was assigned in step 4.2 *after*. Now computed before the
  loop (`IOCTL_SCSI_EXECUTE_NONE/IN/OUT`). Did **not** resolve the symptom.
- **NULL WDF context-type deref** in this function — FIXED (commit 2282306ac70): the
  `WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attr, null())` is now
  `addr_of!(crate::ddk_shims::CDROM_REQUEST_CONTEXT)`.
- `WDF_REQUEST_SEND_OPTION_SYNCHRONOUS = 0x01` — correct (verified vs WDF).
- `WdfIoTargetFormatRequestForInternalIoctlOthers` is a real dispatch (idx 191), not a stub.

### Next steps (suggested)
1. **Read the fresh `[SRB]` trace** (added in the uncommitted edit — run a build+smoke, see
   "Build/test" below). It prints, in the send loop:
   `[SRB] pre-send ioctl=%x mj=%x srb=%p irp=%p` then
   `[SRB] post-send st=%x srbst=%x dtl=%x`. This tells you whether the IRP is even set up with
   `mj=0x0F` (IRP_MJ_SCSI) at send time, and the status/SRB right after `WdfRequestSend`.
2. **Suspect the manual IRP-stack override vs the WDF format.** `WdfIoTargetFormatRequestForInternalIoctlOthers`
   sets up the *next* stack location as IRP_MJ_INTERNAL_DEVICE_CONTROL for the target; the C code
   then re-fetches `IoGetNextIrpStackLocation` and overrides `MajorFunction = IRP_MJ_SCSI` +
   `Parameters.Scsi.Srb`. Verify (a) the Rust `IoGetNextIrpStackLocation` returns the SAME stack
   slot the WDF format wrote (KMDF may have advanced the IRP's CurrentLocation), and (b) the
   `Parameters` union access (`nextStack->Parameters.Scsi.Srb`) lands at the correct offset in the
   bindgen `IO_STACK_LOCATION` (check `ntkrnl-sys` `_IO_STACK_LOCATION.Parameters` union layout vs
   the C header — a wrong union arm/offset would leave Srb unattached → port driver sees no SRB →
   SrbStatus stays PENDING).
3. **Check `WdfRequestSend` to `IoTarget` actually forwards to the lower (physical) device.**
   `IoTarget` was created in `DriverEvtDeviceAdd` via `WdfIoTargetCreate` +
   `WdfIoTargetOpen(... WdfDeviceWdmGetAttachedDevice(Device))` (cdrom.rs ~709). If the target's
   attached-device is wrong/NULL, the IRP completes at the WDF layer (status 1?) without descending
   to scsiport. Inspect what `WdfDeviceWdmGetAttachedDevice` returns at runtime.
4. `st=1` (STATUS_WAIT_1, 0x00000001) is an odd completion status — trace `WdfRequestGetStatus`
   vs the IRP `IoStatus.Status` directly to see who sets 1.

## Translation-bug patterns found this session (recognize these downstream)

The read path (post-INQUIRY) will likely have more of the same classes. When debugging, suspect:
1. **Fabricated constants** — LLM invents IOCTL/SRB/enum *values*. Build is green (just a u32),
   fails at runtime. Examples fixed: `IOCTL_SCSI_GET_ADDRESS 0x41018c→0x41018`, SRB flag/status
   consts, IOCTL_SCSI_EXECUTE_*. **Verify every `const` against `types_amd64.rs` (bindgen) or the
   C header before suspecting logic.**
2. **`write_bytes` element-vs-byte** — `core::ptr::write_bytes::<T>(p,v,n)` writes `n*size_of::<T>()`
   bytes; C `memset/RtlZeroMemory` `n` is BYTES. On a typed `*mut T` this over-zeroes by
   `size_of::<T>()×`. **Every `write_bytes` target must be `*mut u8`.** Swept 17 sites already; if a
   new one appears, same fix. (A 2× variant on `*mut u16` smashed a pool header → BUGCHECK 0x19.)
3. **NULL WDF context-type** — `WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&a, null())` → derefs
   `(*ContextType).UniqueType` at +0x18 → 0x1E. Pass the real type-info.
4. **WDF_OBJECT_ATTRIBUTES via `mem::zeroed()`** leaves `ExecutionLevel/SynchronizationScope = 0`
   (the *Invalid* enum) → `STATUS_WDF_OBJECT_ATTRIBUTES_INVALID` (c0200209). Must be `1`
   (InheritFromParent) — use the `WDF_OBJECT_ATTRIBUTES_INIT*` shims (ddk_shims.rs).
5. **Statement-order inversion** — value used before assignment (the ioctlCode bug). The c2rust/LLM
   pass occasionally reorders.
6. **No-op stubs** — `RtlStringCchPrintfW`, `TracePrint` in ddk_shims.rs are stubs (TracePrint
   drops varargs → its FATAL messages never print; add a local `extern{fn DbgPrint}` for real
   tracing). Check ddk_shims.rs for any function that silently returns success.

## Build / test / debug tooling

All from the worktree root `/home/curi/Projects/os-playground/vibeos-wt-cdrom`.

```bash
# Build amd64 (warm). Build then smoke must be SEPARATE invocations or smoke flags stale media.
./vibeos-cli build  --arch amd64
ls -la build-amd64/drivers/storage/class/cdrom/cdrom.sys   # confirm relinked (mtime) — a FAILED
                                                           # cargo build leaves the OLD .sys
./vibeos-cli smoke  --arch amd64                            # boots livecd, HANG/BUGCHECK verdict

# Read the serial trace (the live log is the source of truth; stdout is a bounded summary):
grep -aoE "\[(SMIO|ALOC|INQ|SRB|D0E|CDS)\][^\x00]{0,60}" qemu/amd64-uefi_smoke_live.log | tail -30
grep -anE "CdRomCount|BUGCHECK|PsLocateSystemDll|REASSIGN" qemu/amd64-uefi_smoke_live.log | tail
```

Existing debug traces in the tree (DbgPrint, intentionally left in — OK to keep/commit):
- `[CDS] N st=%x` — `DriverEvtDeviceAdd` steps 9..20 (cdrom.rs).
- `[D0E] enter` — `DeviceEvtD0Entry` (cdrom.rs:1098).
- `[SMIO] ...` — `DeviceEvtSelfManagedIoInit` steps (init.rs:285+): enter, 1a-scsiaddr, 1b-descriptors,
  1-bufs, 3-scanspecial, 4-hotplug, 4b-inquiry, 5a-mcd, 5b-mmc, 5c-zpodd, 5d-readcapacity, 8-CdRomCount++.
- `[ALOC] ...` — `DeviceInitAllocateBuffers` (init.rs:4433): enter/fdo/zeroed/sense/scratch/done.
- `[INQ] send1 ...` — first INQUIRY in `DeviceCacheDeviceInquiryData` (init.rs:1214).
- `[SRB] pre-send/post-send ...` — inside `DeviceSendSrbSynchronously` send loop (common.rs ~755,
  **UNCOMMITTED** at handoff time — `git diff` to see; build+smoke to capture).

GDB toolkit (for BUGCHECK/hang triage; preflight fails on Rust-no-debuginfo, use `--skip-preflight`):
```bash
RUN=$(qemu/start.sh amd64 --gdb --skip-preflight 2>/dev/null | grep -oE "amd64-[0-9]+-[0-9]+" | head -1)
qemu/wait-for.sh "$RUN" --pattern "CDS\] 09" --timeout 120
# Fixed slide (no KASLR): runtime = fileVA + 0xfffff80000000000. ntoskrnl KeBugCheckEx fileVA via:
grep -P "\tKeBugCheckEx\b" qemu/symbols-amd64.tsv
qemu/build-index.sh amd64        # (re)builds qemu/symbols-amd64.tsv for addr2sym
qemu/addr2sym.sh amd64 0x4a473c  # symbolicate an ntoskrnl fileVA
```
- **Hardware watchpoint recipe** (used to crack the 0x19): connect early, `hbreak`/`watch` on a
  fixed linear addr (deterministic across boots here), `continue`. `hbreak` uses debug registers so
  it arms before the page is mapped (plain `break` fails "cannot access memory" in UEFI phase).
- **cdrom.sys base** (~`0xfffff80004a00000`) walk: at a KeBugCheckEx hbreak, walk
  `PsLoadedModuleList` (fileVA in symbols-amd64.tsv); amd64 `LDR_DATA_TABLE_ENTRY`: DllBase +0x30,
  BaseDllName.Buffer +0x60.

## Commits on `cdrom-port` (this session, all broken-but-integrated = correct state)
- `726f2c58904` kmdf cohab substrate (KeTickCount/KiEnableTimerWatchdog x86_64 export; WdfFunctions_01017)
- `3de091965b0` ~123 invented constants + WDF param bugs (AddDevice ~95%)
- `88538bb5fc1` 6 bugs → DriverEvtDeviceAdd fully succeeds
- `2282306ac70` write_bytes size² sweep (17) + NULL WDF context derefs → device-start runs
- `121452a2bd9` ioctlCode-before-format in DeviceSendSrbSynchronously + INQ trace
- (uncommitted) `[SRB]` send-loop trace in common.rs

Full chronology + every fix detail: memory `project_cdrom_kmdf_blocker.md`. Driver-translation bug
catalog: `docs/engineering/translation-debugging-field-manual.md`,
`feedback_llm_invented_enum_constants.md`.

---

## UPDATE 2026-06-13 — Async read path DONE; smoke blocked by a STORAGE-layer stall (bank until uniata)

**cdrom-rust boots amd64 to `shell ready` (full desktop), 0 bugchecks.** Read path is correct.

### Option A landed: async Rust read path (C shim no longer in the read dispatch)
- `cdrom.rs` queue config (`EvtIoRead`/`EvtIoWrite`) now points at the **Rust** `SequentialQueueEvtIoReadWrite` → `RequestValidateReadWrite` → `RequestHandleReadWrite` → `ScratchBuffer_PerformNextReadWrite` + `ScratchBuffer_ReadWriteCompletionRoutine` (async; `[SCR] ... sync=0`).
- The C shim's synchronous read reimplementation (`CdromSequentialQueueEvtIoReadWrite`/`CdromHandleReadWrite`/`CdromScratchBufferPerformReadWriteSynchronously`) is **dead code now** (not registered). Owed: physically delete the shim's read-path C once converged.

### Why smoke still times out (NOT a cdrom bug)
Smoke (6min) fails `HANG_TIMEOUT`; the boot takes ~8min. Gap profiler (`CdromGapLog`, `[GAP]` lines, in `rust_dbgprint_shim.c`, called from the Rust read entry) shows:
- **26 silent mega-stalls of fixed ~6.47s (20×) and ~7.9s (6×) ≈ 177s** dominate the boot.
- Individual reads are ~0.5ms (`[SENDPERF]`/`[RWPERF]`). Mid-gaps total ~1.5s.

Ruled out with instrumentation:
- **NOT MCN** — `mcnPolls=0` on every stall (counter via `CdromMcnPollMark` from `RequestSetupMcnRequest`).
- **NOT async-specific** — the SYNC shim path has the identical ~6.47s stalls.
- **NOT cdrom's read SRB timeout** — capping read `TimeOutValue` to 2s did **not** shrink the stalls (stayed ~6.35s). So the 6.47s/7.9s is a **storage-layer internal timer** (atapi/scsiport/uniata), not any cdrom `TimeOutValue` (SCSI_CDROM_TIMEOUT=10, TUR=20, GESN=4 — none match).

**Conclusion:** every ~30–45 reads, a cdrom read SRB's completion is missed by the storage stack; the port layer hits its own ~6.47s/7.9s timeout, then the command completes/retries (silent). crp's C cdrom on the same storage stack does NOT hit this. This is a **storage completion-reliability issue, outside cdrom translation.**

### → uniata convergence handoff
The fix belongs in the IDE/ATAPI port path (uniata / scsiport-rust / atapi-rust): periodically a cdrom READ command's interrupt/completion is lost. When that lands, re-converge cdrom:
1. Rebuild + `vibeos-cli smoke --arch amd64` — expect the `[GAP]` mega-stalls gone and boot < 6min (green).
2. Delete the shim read-path C + the perf instrumentation (`CdromGapLog`/`SENDPERF`/`RWPERF`/`CdromMcnPollMark`).
3. Verify i386 + arm64 build/boot of the async path.

**State:** all uncommitted on `cdrom-port`. Async read path active; perf instrumentation + normal cdrom traces live.
