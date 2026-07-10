# Wall-2 handoff: arm64 USB HID — ReopenPipe REMOVE spin (task #18)

> **RESOLVED 2026-07-10 (landed `b22168c8acc`, gate ×3 green).** Root cause was one
> level below the localized spin: the EHCI completion **DPC could never drain** —
> the arm64 IRQ-exit device-DPC retire path skipped the idle thread (pinned at
> DISPATCH), and DPC bodies ran with PSTATE.I open, letting the level-triggered
> EHCI line re-enter its ISR and starve the DPC quiescing it. Fix (codex lane):
> allow idle-at-DISPATCH to retire device DPCs; keep IRQs masked during IRQ-exit
> DPC bodies; EHCI ISR accumulates status bits + returns FALSE for rollover-only
> interrupts; bounded 2ms stall in ReopenPipe post-remove. arm64 now delivers
> `[KBDCLASS] first key delivered`. The evidence chain below is kept as a worked
> example of the localization method.

**Status 2026-07-09 (superseded):** root cause localized to a **polling loop that never exits inside
the EHCI miniport's `SetEndpointState(REMOVE)` path on arm64**. One more probe should
name the exact loop; the fix is likely restoring a dropped timeout-vs-C comparison.
amd64 delivers end-to-end (`USB-HID-KBD-OK`) with the identical driver code.

## Repro (arm64)

```bash
./vibeos-cli build --arch arm64
printf 'fs0:\\efi\\boot\\BOOTAA64.EFI\r\n' > /tmp/startup.nsh   # boot.img is regenerated per build;
mcopy -o -i qemu/boot.img /tmp/startup.nsh ::/startup.nsh       # EDK2 falls to UEFI shell with extra PCI devs
RUN=$(VIBEOS_QEMU_EXTRA="-device usb-ehci,id=xhid -device usb-kbd,bus=xhid.0 -device usb-mouse,bus=xhid.0" qemu/start.sh arm64)
qemu/wait-for.sh $RUN --pattern "W2-SES.*state=4" --timeout 300
# serial: qemu/runs/$RUN/serial.log — system WEDGES (serial stops) right after state=4
```

amd64 comparison (delivers, prints golden tape): `scratchpad verify_usb_hid.py amd64-uefi`
with `VIBEOS_QEMU_EXTRA_ARGS` (note `_ARGS` for vm.py; start.sh uses `VIBEOS_QEMU_EXTRA`).

## The evidence chain (each theory retired with data)

1. **usbport → EHCI parameter flow is CLEAN.** `[W2-QT]/[W2-QA]/[W2-SUB]/[W2-EHCI]`
   prints (queue.rs / endpoint.rs / usbehci.rs, all marked `TEMP Wall-2`) show pointer
   identity + correct `len=0x40` + setup bytes `80 06 00 01 00 00 40 00` end-to-end.
   Old "TransferParameters zero / MapTransfer never called" lore is DEAD.
2. **DMA common buffers are physically contiguous** (`[W2-CB] ... noncontig=0`,
   hal-rust arm64 dma.rs self-check).
3. **Interrupt delivery works.** `[W2-ISR]` fires with sts=1 (completion), 0x20 (IAA),
   4 (port change), 8 (rollover). GICD_ISENABLER1 bit4 (INTID36) set, PMR=0xF0,
   PSTATE.I clear. pci-rust INTx synthesis (line 36 = dev2 pinD) matches QEMU.
4. **Enumeration progresses:** addr-0 GET_DESCRIPTOR(64) completes; SET_ADDRESS(1)
   completes; endpoint state change ACTIVE confirms via EpStateChangeList
   (`[W2-EPSC] ... ready=1`, hcfn>epfn — frame counter fine).
5. **The wedge:** `USBPORT_ReopenPipe` (usbport-rust/endpoint.rs:1284; recycles EP0
   for the new address) calls miniport `SetEndpointState(REMOVE)` directly
   (`[W2-SES] state=4` prints, from EHCI_SetEndpointState entry) — and the very next
   line `[W2-RP] pre-wait` (endpoint.rs, right after the call returns + 2 spinlock
   releases) NEVER prints. The hang is INSIDE
   `EHCI_SetAsyncEndpointState(REMOVE)` → `EHCI_RemoveQhFromAsyncList` (+ its
   helpers `EHCI_LockQH` / `EHCI_FlushAsyncCache`, usbehci.rs ~2200-2450).
6. It spins at DISPATCH **holding EndpointSpinLock + MiniportSpinLock**
   (ReopenPipe takes both around the call, endpoint.rs:1317-1327). GDB backtrace of
   the wedge: `KeAcquireSpinLockAtDpcLevel` spinning, caller
   `USBPORT_WorkerThreadHandler` (blocked on MiniportSpinLock) — single CPU → total
   livelock, serial dies. Matches sample-pc histogram (KfRaiseIrql + usbport loop).

## Prime suspect (verify before fixing)

`EHCI_FlushAsyncCache` (usbehci.rs:~2228-2322) vs C original
(`drivers/usb/usbehci/usbehci.c:1670`): the C has `if (CurrentTime > EndTime) break`
timeouts in its polling loops; the Rust translation reads `CurrentTime`/`EndTime` in
loop 2 (wait for ASYNCSS==1, ~2276) and the doorbell loop (~2298) but **never
compares them** — dropped-timeout translation bug class. Loop 1 (~2248) kept its
timeout (BugCheck). BUT: the wedged boot's registers read ASE=1/ASYNCSS=1/IAAD=0,
which satisfies none of those loops' spin conditions *at dump time* — so either the
register state evolved after the spin began, or the actual spin is in
`EHCI_LockQH`'s FrameIndex-poll or `EHCI_RemoveQhFromAsyncList`. **Next probe:**
one DbgPrint at entry/exit of each of RemoveQhFromAsyncList / LockQH /
FlushAsyncCache (or GDB `bt` at wedge with usbehci symbols) names the loop in one
boot. Then restore the C's timeout semantics in that loop (and audit the sibling
loops for the same dropped comparison — likely all of them should get the fix).

Why arm64-only: timing. amd64/QEMU satisfies the polled condition before the missing
timeout matters; arm64 TCG ordering leaves the condition unmet forever. The bug is
arch-independent C-fidelity; arm64 is just the arch that trips it.

## Instrumentation in tree (all marked `// TEMP Wall-2 instrumentation (task #18)`)

- usbport-rust: queue.rs (QT/QA), endpoint.rs (SUB, DEW rate-limited, CP loop, RP
  brackets), usbport.rs (EPSC in IsrDpcHandler walk).
- usbehci-rust: usbehci.rs (ISR, SES, EHCI-recv in SubmitTransfer).
- hal-rust: arm64/dma.rs (CB contiguity self-check).

Remove all of these before landing the fix. Success marker:
`[KBDCLASS] first key delivered` on the arm64 probe boot. Then gate ×3 + land.

## Also note

- `qemu/symbols-arm64.tsv` lacks Rust symbols past `orgHalInitPnpDriver` (cargo-linked
  kernel `ntoskrnl-cbd27bfd4bbb639a.exe`, ImageBase 0x400000, symbols DWARF-only —
  llvm-symbolizer hits `$R00000E` pseudo-reloc junk). Symbolicate drivers by objdump
  pattern-match (worked: found USBPORT_WorkerThreadHandler by matching insn bytes).
- Boot media: build regenerates boot.img — re-add startup.nsh after every build.
