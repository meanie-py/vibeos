# Handoff: netstack `device::bringup()` hangs at DriverEntry on i386 net boot

> **ARCHIVED 2026-07-10 — RESOLVED.** The hang was fixed on the same lane before it merged:
> `946935e042a` (2026-07-03) has the full i386 usermode winsock roundtrip GREEN
> (ws2_32→msafd→AFD→netstack, real-internet HTTP 200), which requires the very
> `device::bringup()` path this doc reports wedged; merged as `feffe213faf`.
> `i386-uefi-net` is a supported net config in `tests/qemu/test_net.py` today. See the
> field manual for the durable bug classes extracted from this lane (`adf78d014fc`).


**Status:** OPEN. Blocks all net runtime validation on `netstack-arm64-m7` (i386-uefi-net),
including the msafd.dll winsock round-trip. **Not a msafd bug** and **not stale-ABI** (both
ruled out below). Owner handing off: netstack lane (meaniepy/netstack). Target: Codex.

## TL;DR

Booting `--arch i386-uefi-net` hangs inside `netstack.sys` **DriverEntry**, in
`device::bringup()` (virtio-net init), *before* the network stack or any usermode/winsock
code runs. The boot never reaches the desktop and times out at 360s. amd64 net is unaffected
(11/11 green). The hang is newly surfaced because **this branch is the first to exercise an
i386 net boot-to-desktop after the mainline catch-up merge** — mainline's virtio/HAL layer
evolved 145 commits with *no i386 net smoke config to catch a regression there*.

## Exact symptom

Serial log tail (`qemu/i386-uefi-net_smoke_live.log`), last 4 lines before the 360s timeout:

```
[SYSDRV] loading #10: \Registry\Machine\System\CurrentControlSet\Services\netstack
[NETSTACK] DriverEntry: netstack.sys loaded
[NETSTACK] M1 self-test: probing for virtio-net
                                                  <- hangs here, no further output
```

Test failure:
```
Failed: Boot failed: Timed out waiting for marker '[EXPLORER] _tWinMain entered' after 360.0s
```

The `[NETSTACK] M1 self-test: probing for virtio-net` line is printed by
`sdk/lib/rust/netstack-rust/src/selftest.rs:25`, immediately before it calls
`device::bringup()`. The next `nsprint!` in that function is either
`FAIL: virtio-net bringup` (the `None` path) **or** the MAC-address line (success). **Neither
appears** → `device::bringup()` is spinning in an infinite loop, not returning `None`.

## Where it hangs

`sdk/lib/rust/netstack-rust/src/device.rs:27`:

```rust
pub fn bringup() -> Option<NetDev> {
    let df = find_virtio_net()?;                                  // (a) PCI enumerate
    enable_bus_master(df);                                        // (b) PCI cmd reg
    let mut root = PciRoot::new(HalPciAccess);
    let transport = PciTransport::new::<NetHal, _>(&mut root, df).ok()?;  // (c) virtio PCI transport
    VirtIONet::new(transport, BUF_LEN).ok()                       // (d) virtio-net device init
}
```

Because `bringup()` never returns `None` (no `FAIL` log), it is NOT a clean "device not found"
— one of (a)–(d) is looping forever. Ranked by likelihood:

1. **(d) `VirtIONet::new` — DMA / virtqueue setup (LEADING HYPOTHESIS).** The `virtio-drivers`
   crate resets the device, negotiates features, and allocates + hands the device the physical
   addresses of the virtqueues via the **`NetHal`** DMA HAL (`sdk/lib/rust/netstack-rust/src/hal.rs:31`,
   `unsafe impl Hal for NetHal`). If `NetHal`'s virt→phys translation (`dma_alloc` / `share` /
   `mmio_phys_to_virt`) is wrong on **i386** (different phys-addr scheme, non-identity map, or a
   HAL change from the merge), the device receives garbage queue addresses, never advances the
   used ring, and the driver's status/feature poll **spins forever**. This is the classic
   "works on amd64, hangs on i386" shape — amd64 DMA/phys works, i386 differs.
2. **(c) `PciTransport::new` — capability walk / BAR mapping.** Reads the virtio PCI capability
   list and maps config BARs via `NetHal::mmio_phys_to_virt`. Same i386-vs-amd64 mapping risk.
3. **(a)/(b)** — unlikely; simple PCI config-space reads. If `find_virtio_net` looped it would
   spin on config reads, but that path is shared with amd64.

## What's been ruled out

- **Not msafd.** msafd.dll is a usermode DLL loaded when ws2_32 first initializes — that is
  *thousands* of boot steps after this DriverEntry hang. The msafd cohab flip (this session)
  compiles + links clean on all 3 arches; it cannot affect `netstack.sys` DriverEntry.
- **Not stale-ABI / stale rlib.** Reproduced after a full §9.6b clean rebuild:
  `rm -rf sdk/lib/rust/target/i686-kernel sdk/lib/rust/target/netstack-cargo-netstack_rust-rustlib/i686-kernel`
  then `ninja ntoskrnl netstack afd msafd && rm -f livecd.iso && ninja livecd` (fresh ISO,
  no errors) — same hang. So `netstack.sys` is built against the merged IRP/IO_STACK_LOCATION
  ABI; the hang is logic, not a mismatched binary.
- **Not a file this branch edited directly.** `git diff continue-rust-port..HEAD` shows this
  branch did **not** touch `device.rs`, `hal.rs`, or `selftest.rs` (the entire bringup path).

## Why it regressed (root-cause hypothesis)

Two independent facts combine:

1. **Mainline's virtio/HAL bringup was never exercised on an i386 net boot.** The pre-merge
   `lib.rs` comment (see diff below) states verbatim: *"amd64 only — … and there is no i386 net
   smoke config."* So over 145 commits, mainline's `device.rs`/`hal.rs`/virtio path could have
   regressed on i386 (DMA/phys, BAR mapping) with no smoke to catch it.
2. **This branch is the first to boot i386 net to desktop.** The M10-i386 work (`isr.rs`)
   flipped the interrupt module — and, more importantly, added an **i386-uefi-net smoke config**
   — so i386 now runs the full virtio bringup at boot. `lib.rs` diff (`continue-rust-port..HEAD`):

   ```diff
   -#[cfg(target_arch = "x86_64")]                              // M10 isr: amd64 only
   -// … "there is no i386 net smoke config."
   +#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]    // M10 isr: i386 + amd64
   +// … "both share the working x86 HAL HalGetInterruptVector + IOAPIC routing."
    pub mod isr;
   ```

   Note the *assumption* baked into that comment — "i386 shares the working x86 HAL." If the
   merge's evolved `NetHal` DMA/interrupt path does **not** in fact work on i386, that assumption
   is the bug's origin. (The `isr` module itself connects the ISR *after* bringup, so it is not
   the direct hang — but the same "i386 HAL is fine" assumption underlies both.)

Net: **mainline's evolved virtio bringup + i386's actual HAL DMA behavior are incompatible**,
and this branch is simply the first to run that combination. Fix belongs in
`netstack-rust/src/{hal.rs,device.rs}` (i386 DMA/phys correctness), not in msafd or the merge.

## Reproduction

```bash
cd /home/curi/Projects/os-playground/vibeos-wt-netstack
# clean rebuild so netstack.sys matches the merged kernel ABI (rules out stale rlib):
rm -rf sdk/lib/rust/target/i686-kernel \
       sdk/lib/rust/target/netstack-cargo-netstack_rust-rustlib/i686-kernel
( cd build-i386 && ninja ntoskrnl netstack afd msafd && rm -f livecd.iso && ninja livecd )
# boot the i386 net config (i386 auto-spawns nettest.exe at boot; see triage.py):
PY=/home/curi/Projects/os-playground/vibeos/.venv/bin/python
PYTHONPATH=. $PY -m pytest tests/qemu/test_net.py -v --arch i386-uefi-net \
    -k "boot_with_nic or usermode_winsock_roundtrip" --timeout 600
# -> both ERROR at setup: boot timeout, serial ends at "M1 self-test: probing for virtio-net"
```

Known-good control: `--arch amd64-uefi-net` boots and passes the net suite (11/11) — use it to
diff the working (amd64) vs hanging (i386) `NetHal` DMA/transport behavior.

## Suggested debug path (GDB)

Use the kernel debug toolkit (`qemu/start.sh i386-uefi-net --gdb`, `qemu/README.md`), or the
UART-trace pattern, to bisect which of (a)–(d) spins:

1. **Cheapest first — add `nsprint!` breadcrumbs** between (a)/(b)/(c)/(d) in `device::bringup()`
   and one inside the `VirtIONet::new` feature/status poll loop. Rebuild netstack only
   (`ninja netstack && rm -f build-i386/livecd.iso && ninja -C build-i386 livecd`) and re-boot.
   This alone localizes the spin to a single call in one iteration.
2. If it's (d): dump the physical addresses `NetHal::dma_alloc`/`share` hand the device on i386
   vs amd64. A non-identity or truncated (32-bit vs 64-bit) phys address is the smoking gun.
   Check `hal.rs` for any `as u32`/`as usize` phys-addr narrowing or an identity-map assumption
   that holds on amd64 but not i386.
3. Confirm i386 `MmGetPhysicalAddress` / the HAL DMA the merge introduced actually returns a
   device-usable bus address (virtio wants the *physical* queue address; if the kernel hands a
   virtual or wrongly-translated one, the device silently drops it → poll spins).
4. If it's (c): the virtio PCI capability BAR mapping (`mmio_phys_to_virt`) is returning a bad
   VA on i386 → the config reads read garbage → status never matches.

## Code pointers

| File | Role |
|------|------|
| `sdk/lib/rust/netstack-rust/src/selftest.rs:25,29` | prints the last-seen marker, calls `device::bringup()` |
| `sdk/lib/rust/netstack-rust/src/device.rs:27` | `bringup()` — the 4-step hang site |
| `sdk/lib/rust/netstack-rust/src/hal.rs:31` | `NetHal` — the virtio DMA/MMIO HAL (**prime suspect for i386**) |
| `sdk/lib/rust/netstack-rust/src/lib.rs` (DriverEntry) | init order; the M10-i386 `#[cfg]` flip + "i386 HAL is fine" assumption |
| `sdk/lib/rust/netstack-rust/src/isr.rs` | M10 ISR (runs *after* bringup — not the direct hang, but shares the assumption) |

## Scope / non-goals

- Fix is i386 virtio bringup (HAL DMA/phys). Do **not** touch msafd — it is done and correct
  (cargo-check + link clean ×3, committed `96abf20d32a`, `98253c63c70`).
- Do not "fix" this by reverting the M10-i386 config or disabling i386 net — that hides a real
  i386 virtio/HAL regression that also affects any future real-hardware i386 path.
- amd64 and arm64 net are out of scope (amd64 green; arm64 stays on polling — its
  `HalGetInterruptVector` is a stub).
