# windows-host QEMU + WHPX — bring-up spike

**Goal:** boot our x86 guests under `-accel whpx` (Windows Hypervisor Platform — real
hardware virt on the Windows host) instead of TCG, for seconds-not-minutes x86
boot/smoke/gdb cycles. This is the doc that `tests/qemu/backend.py` and
`tests/qemu/cli/config.py` reference. arm64 is out of scope (WHPX is x86-only).

Status: **2026-07-03 — BOTH arches boot to desktop under WHPX** (i386 to
`[EXPLORER] _tWinMain` + full service startup, amd64 to `[WIN32K] shell ready`)
after two fixes: the freeldr Rust varargs bug (i386) and `kernel-irqchip=on`
(amd64 — see below). Default accel stays `tcg` until whpx smokes prove reliable
across repeats; flip via `VIBEOS_QEMU_WINDOWS_ACCEL=whpx`.

## How to run it

```bash
VIBEOS_QEMU_WINDOWS_ACCEL=whpx ./vibeos-cli smoke --arch amd64   # or --arch i386
VIBEOS_QEMU_WINDOWS_ACCEL=whpx ./vibeos-cli gdb start amd64      # live instance, serial in staging dir
```

Plumbing (see `tests/qemu/backend.py`): `rewrite_accel` swaps `-accel kvm` →
`-accel whpx,kernel-irqchip=off` (irqchip=off REQUIRED) and drops `-cpu host`
(faults OVMF under whpx). WHPX lives only in `qemu-system-x86_64.exe`, so i386
guests run on the 64-bit emulator. `whpx_available()` probes
`Win32_ComputerSystem.HypervisorPresent` via CIM.

## History — the two original blockers (both root-caused 2026-07-03)

The spike's institutional memory was "i386 hangs at freeldr stack-switch; amd64
BUGCHECKs 0x9 at uniata DMA". Both reproduced; neither is what it looked like.

### i386 "freeldr stack-switch hang" — FIXED (Rust varargs array-by-value)

`[freeldr] Switching stack...` is merely the **last ConOut message before the TUI
menu takes over the screen** — everything after it is serial-silent by design, so
any later wedge "looks like" a stack-switch hang.

The real wedge: `TuiDrawMenuTimeout` (`sdk/lib/rust/freeldr-rust/src/tuimenu.rs`)
passed `UiTimeText` — a `[CHAR; 260]` static — **by value** into the C-variadic
`RtlStringCbPrintfA("%s %d", ...)`. Rust arrays do NOT decay to pointers like C
arrays: all 260 bytes were copied into the i386 varargs stack area, and `streamout`'s
`%s` read the first 4 bytes ("Seco" of "Seconds until highlighted choice…") as the
string POINTER → `strnlen(0x6f636553, SIZE_MAX)` over unmapped guest-phys.

- Under **TCG** unmapped reads return instantly → the scan terminates fast →
  latent for months (menu just renders garbage nobody looked at).
- Under **WHPX** every probe of unmapped guest-phys is a VM exit (~µs) → the
  4G-iteration scan ≈ 12 hours → "hang". Diagnosed by gdb-attaching to the wedged
  guest (register reads work under WHPX even where soft breakpoints don't) and
  sampling the spinning PC (counter advanced ~100k/s between samples).

Fix: `UiTimeText.as_ptr()` at both call sites (tuimenu.rs:196/205). A crate-wide
scan for the same anti-pattern (array-typed ident passed bare to a variadic) found
no other real instance. **Result: i386 boots to `[EXPLORER] _tWinMain` under WHPX.**

Gotcha class for the field manual: **Rust → C-variadic calls never decay arrays.**
Any `[T; N]` passed to a `...` extern must be `.as_ptr()`d. On amd64 this bug hides:
the Win64 ABI passes >8-byte aggregates by reference, so the callee accidentally
receives a valid pointer.

### amd64 "BUGCHECK 0x9 at uniata DMA" — FIXED (`kernel-irqchip=on`)

Reproduced deterministically with no gdb attached:
`*** BUGCHECK: 0x00000009 (P1=<pool lock>, P2=0, 0, 0)` right after
`IopInitializeBuiltinDriver: uniata.sys` + scsiport's "DMA is not supported yet"
(the DMA message also appears in green TCG boots — it's noise, not the trigger).

Drill-down (instrumented every 0x9 assert with a site id + CR8 resampling):
the firing site is `KeReleaseSpinLockFromDpcLevel` — the ACQUIRE passed at
DISPATCH+, the RELEASE of the same pool lock (scsiport's
`DeviceExtension->SpinLock`, via the `ScsiPortIoTimer` DPC path) read CR8=0,
persistently (8/8 samples). CR8 collapsed *inside* the critical section, which
does IDE port I/O → VM exits.

**Root cause, proven by an in-guest probe** (one-shot in the first DPC
interrupt: `mov cr8, 12` → `in al, 0x80` (one port-I/O exit) → read CR8 back,
×1000, interrupts disabled):

| irqchip | CR8 writes lost across one port-I/O exit |
|---|---|
| `kernel-irqchip=off` | **1000/1000** (readback = 0) |
| `kernel-irqchip=on`  | **0/1000** |

With `kernel-irqchip=off`, QEMU emulates the APIC in userspace and round-trips
the TPR through a lazy cache (`vcpu->tpr` / `cpu_get_apic_tpr` in qemu
`target/i386/whpx/whpx-all.c`); the guest's CR8 write never reaches that cache,
and the exit path restores the stale value — **guest CR8 is reset to 0 on every
VM exit**. amd64 NT keeps IRQL *in* CR8 (`mov reg, cr8` in
`ntkrnl-sys::ke::KeGetCurrentIrql`), so every raised IRQL silently collapses at
the first exit; the only place that *notices* is the first asserting release —
hence the deterministic "uniata" signature (ScsiPortIoTimer is the first DPC
that asserts inside an exit-heavy critical section). i386 is immune because its
PIC HAL keeps IRQL in memory, which is why i386 boots fine under the same flag.

Fix: `rewrite_accel` now emits a **per-arch `kernel-irqchip`**: `on` for amd64
(Hyper-V's in-kernel APIC — TPR/CR8 handled natively by the hypervisor), `off`
for i386. i386 must stay `off`: its UP kernel drives the legacy 8259 PIC and
keeps IRQL in memory (immune to the CR8 bug), and under `irqchip=on` it wedges
mid-boot (last serial: NETSTACK self-tests) — while under `off` it boots to
explorer. The old "irqchip=off is REQUIRED" lore came from the original spike
against an older QEMU; on QEMU v11.0.0-dev (weilnetz 2026 build) it's per-arch
as above. `VIBEOS_WHPX_IRQCHIP=on|off|split` overrides for experiments. amd64
boots to `shell ready` with this alone — no guest change.

## Other WHPX findings

- **Firmware CD I/O is glacial under WHPX**: OVMF's ATA/ATAPI driver reads the
  livecd via PIO port I/O (port 0x170/0x177 secondary IDE) — every 2-byte read is
  a VM exit. The uefildr file-load phase takes minutes (vs seconds under TCG's
  fast port I/O) and is serial-silent, so the x86 smoke defaults (120s silence
  watchdog / 360s boot timeout) false-abort a healthy whpx boot. `vm.py` now
  scales both when accel resolves to whpx (silence ≥420s, timeout ≥900s).
  Future speedup options: attach the CD via AHCI for the firmware phase (OVMF
  AtaAtapi in AHCI mode does DMA), and/or treat debugcon growth as liveness.
  The kernel phase is FAST once booted (hardware virt).
- **Build-system false-fresh found by this spike** (fixed in
  `reactos-link/src/lib.rs`): cargo-driven modules emitted explicit
  `rerun-if-changed` lines, which disables cargo's default package watching —
  an edit in a transitive path-dep (ntoskrnl-rust behind the kernel-rust
  umbrella) left `vibeos-cli build` printing success while shipping a STALE
  ntoskrnl.exe. reactos-link now watches every workspace crate's `src/`.
- The gdb engine assumes the stub arch matches the guest arch; an i386 guest under
  WHPX runs on `qemu-system-x86_64.exe`, whose gdbstub speaks i386:x86-64 —
  `vibeos-cli gdb inspect` verbs fail on the 'g' packet. Workaround: raw
  `gdb-multiarch` + `set architecture i386:x86-64`. (Engine fix TODO.)
- gdb attach to the whpx gdbstub works for register/memory reads; each
  attach/resume does a full register writeback from QEMU's (possibly stale) CPU
  state — treat CR8 as suspect after any attach.
- `-no-reboot` + `whpx`: a triple fault would exit QEMU (good for harness verdicts).

## Remaining work — the amd64 net suite under whpx (13/15)

With the savevm degrade in place the net suite runs: **13/15 pass** under whpx.
The two failures are the INBOUND-connection tests (`test_tcp_echo_roundtrip`,
`test_tdi_server_listen_accept` — host connects into the guest via slirp
hostfwd; the outbound TCP/ICMP/DNS tests all pass, so the guest RX path works).
Evidence so far:

- One full-suite run: both failed at `s.recv()` timeout with the VM up and the
  guest's echo listener announced.
- A same-boot serial breadcrumb on a later run: `[NETSTACK] TDI server: LISTEN
  did not complete 0xc00000b5` (STATUS_IO_TIMEOUT) — the guest-side TDI listen
  endpoint itself sometimes fails to come up under whpx.
- Isolated re-runs of just these two tests were blocked 3/3 by the
  **pre-existing** amd64 service-startup wedge (`DcomLaunch done: 1053`
  discriminator — the Mm pushlock lost-wakeup lane, ~50% on amd64 on ANY
  accel), so a clean A/B against test-ordering hasn't landed yet. Re-run once
  that lane's fix lands. Note: direct `pytest tests/qemu/test_net.py` needs
  `--timeout 900` under whpx — the default 300s pytest-timeout kills the
  session fixture during the slow whpx firmware phase.

## Perf notes (why we still want this)

i386 whpx: firmware+freeldr PIO phase slow (~2-4 min), kernel-to-explorer fast;
total wall ~5 min with most of it in the CD-PIO phase — fixable per above. TCG
i386 smoke is ~2 min total but burns ~4 host cores at 100% for the duration;
whpx's kernel phase uses a fraction of that. Proper numbers once amd64 boots.
