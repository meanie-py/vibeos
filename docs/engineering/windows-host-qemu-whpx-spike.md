# Windows-host QEMU: WHPX acceleration spike

**Date: 2026-06-26.** Lane: `smoke-tests` (build-cache / windows-host backend).
**Status: SHELVED — whpx is wired + opt-in, but does NOT boot our x86 guests cleanly. Default accel stays `tcg`.**

## Goal

The `windows-host` QEMU backend (see `docs/engineering/windows-host-qemu-spike.md`) escapes
WSL2 nested-virt contention by running the *Windows* `qemu-system-*.exe`. Today it runs **TCG**
(pure emulation). The Windows Hypervisor Platform (WHPX) was enabled on the swarm host
(`HypervisorPresent = True`), so the hope was to move x86 guests from TCG to **whpx** hardware
virt for a large speedup — especially amd64, the slow KVM-contended arch.

WHPX is x86-only (it lives only in `qemu-system-x86_64.exe`; an i386 guest runs 32-bit under
that exe), so arm64 must stay TCG regardless.

## What was built (kept, opt-in)

A per-arch accel resolver + probe, so whpx can be turned on safely once it boots:

- `backend.resolve_accel(arch, configured, whpx_ok)` — arm64 → always `tcg`; x86 honors the
  configured value; x86+whpx degrades to `tcg` only on a *definite* "no hypervisor" probe.
- `backend.whpx_available()` — non-admin probe via `Win32_ComputerSystem.HypervisorPresent`
  (`Get-WindowsOptionalFeature -Online` needs elevation → always inconclusive, so it's not used).
  Returns True / False / None (None = trust config, don't degrade).
- `backend.rewrite_accel(..., "whpx")` — `-accel whpx,kernel-irqchip=off` and **drops `-cpu host`**
  (see finding 1).
- `vm.py` — stores the resolved accel on `self._win_accel`; `_acquire_arch_slot` re-serializes
  same-arch x86 **whpx** boots (whpx contends for the host hypervisor like KVM; tcg does not).

Opt in with `qemu.windows.accel = "whpx"` in `.vibeos-cli.local.json`, or
`VIBEOS_QEMU_WINDOWS_ACCEL=whpx`.

## Findings (boot tests on 2026-06-21/26 livecd media; tcg boots the *same* media fine)

### 1. `-cpu host` under whpx → OVMF FXSAVE fault in early firmware (BOTH arches)

With the original `-cpu host` (which `rewrite_accel` used to keep for whpx), both i386 and amd64
fault in OVMF before reaching the loader:

```
FXSAVE_STATE - 1BF54C20
!!!! Find image based on IP(0x829022) (No PDB) (ImageBase=...82..., EntryPoint=...) !!!!
```

The host CPU feature set WHPX exposes via `-cpu host` isn't what the 32/64-bit firmware expects
(FPU/XSAVE state). **Fix applied:** `rewrite_accel` now drops `-cpu host` under whpx too (falls
back to qemu's default model). This is strictly better — it gets past firmware.

### 2. Past firmware, neither x86 arch boots cleanly under whpx

With `-cpu host` dropped:

| arch | result under whpx |
|---|---|
| **i386** | through OVMF32 + freeldr init, then **hangs hard at `[freeldr] Switching stack...`** (the freeldr→kernel handoff / mode transition). 2×360s no-complete. |
| **amd64** | boots **deep into the kernel** — loads built-in drivers (scsiport, mountmgr, uniata) — then `DMA is not supported yet` → **BUGCHECK 0x00000009** during uniata init. |

Both arches boot to desktop on **tcg** with the identical media (i386 ~75s), so these are
whpx-specific (CPU mode-transition / device-DMA emulation differences), not media or OS bugs.

## Why the default stays `tcg`

A default accel that doesn't reach the desktop would red-line every x86 gate for every agent —
that violates the north-star (smoke is the source of truth). So whpx is opt-in and `tcg` remains
the committed default. The infra above is correct and tested; it's ready the moment the two boot
blockers below are solved.

## Open follow-ups (whoever resumes the spike)

1. **i386 freeldr stack-switch hang under whpx** — instrument `freeldr` around "Switching
   stack..."; likely a PAE / protected→long-mode transition or a self-modifying/segment reload
   WHPX handles differently. Possibly try a specific `-cpu` model (`-cpu max`, `Haswell`) rather
   than the bare default.
2. **amd64 uniata BUGCHECK 0x9 ("DMA not supported yet")** — whpx IDE-DMA emulation differs from
   tcg/kvm; uniata takes a path it bugchecks on. Check whether a non-`-cpu host` model or a
   different `-M`/IDE setup avoids it, or whether uniata's "DMA not supported" fallback is the
   real bug (it should degrade to PIO, not bugcheck).
3. If both are solved: enable `amd64=whpx` (high value), keep `i386=tcg` if its hang is
   intractable, `arm64=tcg` always. The resolver already supports an arch-specific default.
