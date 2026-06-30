# ARM64 Platform-Parity Audit (re-verified 2026-06-21)

Where the **arm64** target is weaker than / diverges from **amd64** and **i386**, and
what it would take to close each gap. Produced by a fan-out audit (kernel config, Rust/HAL
stubs, test harness, boot/driver/HAL coverage) after the arm64 fastfat stack-overflow fix
(`3b3f9744c0a`), re-verified against the tree 2026-06-21.

**Status line (2026-06-21):** arm64 **cold-boots green to the graphical desktop in QEMU virt
(~47s)** on mainline — the same real smoke marker as x86. Boot is **not** a current blocker.
Resolved this cycle: PTE-cache alignment fix + fastfat staging + the `ke/arm64`
KiDispatchException EL0 user-AV fixup (`ke/arm64/trap.rs`). Remaining: **(1) a regressed
kernel-stack-size fix that needs re-landing — see ⚠️ below; (2)** a non-blocking flaky arm64
taskbar/netstack `0xF4` (smss EL0 sysreg-write under net load, post-marker); **(3)** the Tier-2
real-hardware HAL stubs (GIC ISR connect, DMA, SMP, reboot) — all still `UNIMPLEMENTED` in the
tree. No GICv2/HAL-interrupt rewrite is landed on mainline yet (in-flight elsewhere).

**Reality check up front:** arm64 currently **boots to the graphical desktop in QEMU virt**,
reaching the *same* real smoke marker as x86 (`[EXPLORER] _tWinMain entered` + `shell ready`;
see `tests/qemu/vm.py:Arm64Config`). It is **not** "idle-detection only" — that lore is
stale (now corrected in `AGENTS.md` § Smoke Tests). The clock IRQ runs through the `ke/arm64`
GIC path, and virtio-mmio devices (input, block) work. So the "no interrupts / no DMA" items
below are **real-hardware / expanded-driver** gaps, not boot blockers today.

---

## Tier 1 — Real parity gaps on the platform we actually run

### ⚠️ REGRESSED — kernel-stack-size fix is NOT in the tree (needs re-landing)
- **Kernel stack was half-size.** `KERNEL_STACK_SIZE`/`KERNEL_LARGE_STACK_COMMIT` were 12 KB
  (the i386 value) vs amd64's 48 KB; `KERNEL_LARGE_STACK_SIZE` 60 KB vs 72 KB. A 64-bit arch
  on a 32-bit-sized kernel stack — and the direct reason the fastfat dirty-path recursion
  *overflowed* instead of being absorbed. The fix bumped it to amd64 parity (0xC000 / 0x12000)
  in `sdk/include/xdk/arm64/ke.h` in commit `26408b0f33a`.
  **🔴 TREE CONTRADICTS THIS (verified 2026-06-21):** `sdk/include/xdk/arm64/ke.h` on both
  `build-cache` and mainline `continue-rust-port` currently reads `KERNEL_STACK_SIZE 12288`,
  `KERNEL_LARGE_STACK_SIZE 61440`, `KERNEL_LARGE_STACK_COMMIT = KERNEL_STACK_SIZE` — i.e. the
  **old undersized values are back**. `26408b0f33a` is an ancestor of mainline but came in via
  a merge *side* branch, and a later merge resolution silently dropped the ke.h hunk (the
  section_table.rs `#[inline(never)] MiSectionPageTableInsert` half of that commit *did*
  survive — it's in the tree). **Action: re-apply the 0xC000 / 0x12000 bump.** Without it the
  deep-path overflow risk that motivated this whole audit is still live on arm64.

### ✅ LANDED this pass (survived in the tree)
- **2 KB stack temporary off the hot section-table path.** (The second half of `26408b0f33a` —
  present in the tree; see Tier-1 detail below.)
- **2 KB stack temporary on the hot section-table path (verified in tree).**
  `MiSectionPageTableGetOrAllocate` held a `_CACHE_SECTION_PAGE_TABLE` (`[ULONG_PTR; 256]`)
  local that, when inlined into `_MmSetPageEntrySectionSegment`, reserved ~2 KB on *every* call
  (incl. the common GET-hit). Extracted into `#[inline(never)] MiSectionPageTableInsert` so it's
  reserved only on the rare allocate path
  (`sdk/lib/rust/ntoskrnl-rust/src/mm/section_table.rs:141`, confirmed present). Cross-arch win.
- **Stale "idle detection" lore corrected** in `AGENTS.md` (arm64 uses the real marker) —
  confirmed in `AGENTS.md` § Smoke Tests.

### 🐛 NON-BLOCKING flaky arm64 `0xF4` under net load (post-marker)
A flaky `0xF4` is seen on **arm64-net under network load** — an smss EL0 sysreg-write fault.
It is **post-`shell ready`** and intermittent, so it does **not** gate the standard smoke
(which has no arm64-net config yet — see the tracked follow-up below). Tracked, not a boot
blocker. (Distinct from the taskbar `0x50` below, which is a win32k child-control read-fault.)

### 🐛 SURFACED by the stack fix — arm64 taskbar BUGCHECK 0x50 (pre-existing win32k bug)
**Caveat (2026-06-21):** the stack-size half of the fix that *surfaced* this is currently
regressed out of the tree (⚠️ above), so on today's mainline arm64 boot does not necessarily
reach the deeper taskbar path that triggers 0x50 — re-landing the stack bump will re-expose it.
With the 48 KB stack, arm64 boot now **progresses further** — explorer's taskbar tray window
+ child controls actually get created (like amd64), and that reaches a **win32k taskbar
read-fault 0x50** (`[EXPLORER] taskbar: child controls created` → `BUGCHECK 0x50`, arg2=0
read, fault addr ~0x2b0 above the trap frame). This is the **arm64 sibling of the
documented-open amd64 taskbar 0x50** (`project_amd64_taskbar_bugcheck_0x50.md`) — same
signature, and amd64 hits it with zero arm64 changes, so it is **not** caused by the stack
fix. Controlled baseline (12 KB stack) confirms: baseline reaches `_tWinMain` but the taskbar
code never runs (no 0x50), so the stack fix *surfaces* a pre-existing bug by enabling deeper
boot, it does not create one. **Note it's PAST the smoke marker** (`_tWinMain`/`shell ready`),
so the standard gate doesn't see it — but it may fire within the post-marker settle window,
which couples landing the stack fix (`37097f9b9b9`) to fixing this. Likely a **shared
win32k bug** (fix helps amd64 + arm64). Next investigation for this lane.

### ⏳ TRACKED follow-ups (real value, but need work/verification — NOT landed blind)
- **No arm64 network smoke config.** amd64 has `Amd64UefiNetConfig` (`amd64-uefi-net`,
  `tests/qemu/test_net.py`); arm64 has none → smoltcp netstack regressions are invisible on
  arm64. **Caveat:** the netstack virtio-net backend is **PCI-only** today; an `arm64-net`
  config needs virtio-**mmio** net support in the netstack first, else it's a known-red config.
  Scope: confirm/add mmio-net binding → add `Arm64NetConfig(Arm64Config)` with
  `-device virtio-net-device` mirroring `Amd64UefiNetConfig`.
- **x86 has no kernel-test config.** The 28-test serial-dispatch kernel suite
  (`tests/qemu/test_kernel.py`) runs on **arm64 only** (`Arm64TestKernelConfig`/`arm64-test`)
  → i386/amd64 are blind to kernel-unit regressions (pool, IRQL, timers, slist…). **Caveat:**
  not a config mirror — arm64 mcopies `/TESTKERNEL` into a FAT `boot.img`; x86 boots a
  read-only livecd ISO, so `i386-test`/`amd64-test` need ISO-rebuild-with-test-kernel +
  `freeldr.ini` injection wiring. Real but multi-hour test-infra work.
- **input/mouse/snapshot-app smoke tests skipped on arm64** (`test_smoke.py:227/333`,
  `test_snapshot_apps.py:201`): QMP `input-send-event` doesn't route to arm64 virtio-mmio
  input → no win32k-input crash-regression coverage. arm64 instead has a driver-level
  delivery test (`test_arm64input_delivery`). Closing needs QEMU-side routing work.
- **idle-CPU-spin detector disabled on arm64** (`test_smoke.py:611`): TCG has no real idle
  (always ~90% host CPU), so the "boots green but 20× slow" detector can't run.
  **Unfixable on an x86 host** — document only; would need aarch64 hardware + KVM.
- **arm64 device-interrupt delivery — GIC dispatch infra LANDED, `HalGetInterruptVector`
  routing still TODO.** Before this pass arm64 drivers couldn't receive IRQs at all:
  `KeConnectInterrupt` was a stub returning FALSE (so `IoConnectInterrupt` always failed),
  `HalEnableSystemInterrupt` was a `TRUE` no-op (GIC line never enabled), and the IRQ
  handler (`HalClockInterruptArm64`) dropped every non-timer INTID → drivers polled
  (netstack M10 RX, scsiport, etc.). **Landed:** a GICv2 device-dispatch path —
  `KeConnectInterrupt`/`KeDisconnectInterrupt` register a KINTERRUPT in a vector→ISR table
  (`ke/arm64/stubs.rs`), `HalEnableSystemInterrupt`/`Disable` program the distributor
  set/clear-enable + priority + CPU0 target for the SPI (`hal-rust/arm64/clock.rs`
  `enable_gic_interrupt`), and `KiIrqHandler → HalClockInterruptArm64` now calls
  `KiDispatchInterruptArm64(intid)` (exported arm64-only in `ntoskrnl.spec`) for non-timer
  INTIDs, which calls the connected ISR. **Boot-safe / inert until the next piece:** the
  ONLY remaining stub is **`HalGetInterruptVector`** (`hal-rust/arm64/halstubs.rs`, returns
  0) — so every driver still bails *before* `IoConnectInterrupt` (e.g. netstack
  `isr.rs::connect` returns null → polls). The real-hardware-routing reason it's hard: on
  QEMU `virt` the virtio-net-pci NIC sits behind the GPEX bridge, whose `interrupt-map`
  routes PCI INTx pins to GIC **SPI 3-6 (INTID 35-38)** with the slot swizzle
  `intid = 35 + ((slot + pin) % 4)`. `HalGetInterruptVector` only receives
  `(InterfaceType, BusNumber, BusInterruptLevel, BusInterruptVector)` — NOT the device
  slot/pin — and `interrupt_line` (PCI cfg 0x3C) is typically `0xFF`/unrouted under EDK2 on
  ARM. So the vector translation needs the PCI `_PRT` / device-tree interrupt-map (or the
  caller must pass slot+pin). Next step to light up interrupt-driven RX on arm64. Bounded
  risk: a wrong vector just means the consumer keeps polling (netstack treats IRQ as an
  optimization), not a boot regression.

---

## Tier 2 — Real-hardware platform gaps (intentional QEMU-virt scoping; the "real OS" roadmap)

These are stubbed in `ntoskrnl/ke/arm64/stubs.c` and `hal/halarm64/generic/halstubs.c`. They
don't block QEMU-virt boot (the current minimal device set dodges them) but separate a PoC
from a real ARM64 OS:

| Capability | arm64 status | Evidence | Impact |
|---|---|---|---|
| GIC interrupt controller (classic PnP path) | `KeConnectInterrupt`/`KeDisconnectInterrupt` UNIMPLEMENTED; `HalGetInterruptVector`→0 | `ke/arm64/stubs.c:174`, `halstubs.c:250` | Real PnP drivers can't connect ISRs |
| DMA | `HalAllocateAdapterChannel`→NOT_IMPLEMENTED, `HalAllocateCommonBuffer`→NULL | `halstubs.c:91/102` | Blocks real storage/net drivers (virtio-mmio currently dodges) |
| SMP / PSCI secondary-core bringup | `KeStartAllProcessors` empty (TODO PSCI/GIC), `HalStartNextProcessor`→FALSE | `ke/arm64/stubs.c:1428`, `halstubs.c:425` | UP-only; amd64 is SMP |
| Graceful reboot/shutdown | `HalReturnToFirmware` = `while(TRUE)` (FIXME PSCI) | `halstubs.c:348` | Reboot/shutdown hangs |
| Get/SetThreadContext | `PspGetOrSetContextKernelRoutine` UNIMPLEMENTED | `ke/arm64/stubs.c:1199` | arm64 usermode debugging broken |
| CPU control-state save/restore, KeRaiseUserException, hibernate, bus enum (`HalGetBusData`→0) | various stubs | `ke/arm64/stubs.c`, `halstubs.c` | Power-mgmt / exception / device-discovery on real HW |
| Boot media | UEFI ramdisk only; no installable/2-stage (rosload excluded for arm64) | `boot/freeldr/freeldr/CMakeLists.txt:236` | No install path; x86 has livecd/bootcd |

## Tier 3 — Confirmed NON-issues (don't chase)
Correct per-arch differences, not bugs: `MM_LOWEST_SYSTEM_ADDRESS` / VA layout (48-bit vs
57-bit), `MM_PAGE_FRAME_NUMBER_SIZE` (36-bit PA), `MI_HIGHEST_USER_ADDRESS` (matches amd64 at
47-bit), `MAXIMUM_PROC_PER_GROUP` (64, correct for `_WIN64`). A larger arm64
`DOUBLE_FAULT_STACK_SIZE` (0x8000) is harmless/defensive.

---

*Source: arm64-love lane; re-verified against the tree on `build-cache`/mainline 2026-06-21.
Companion to `arm64-clock-and-lpcexitthread-findings.md` and field-manual §9.11.*
