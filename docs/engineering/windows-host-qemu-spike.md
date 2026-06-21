# Spike: hosting the smoke QEMU on the Windows host (escape WSL2 nested virt)

Status: **spike complete, 2026-06-21** (build-cache lane). Goal: learn whether running QEMU on
the *Windows host* (instead of nested KVM inside WSL2) is feasible, to escape the nested-virt
timing contention that produces false `HANG_TIMEOUT`s (see the contention analysis below). This
doc records what the spike proved, the one blocker, and a sketch for a dual-backend smoke runner.

## Why (the problem being escaped)

Smoke boots use `-accel kvm` inside WSL2. WSL2 is itself a Hyper-V VM, so that KVM is **nested**
(L2-on-L1). Boots are 1-vCPU on a 32-core host, so the observed 3.5× same-arch stall
(103s→361s) is **not** CPU starvation — it's nested-virt **timing/VM-exit amplification**: timer
reads / interrupt injection / I/O exits trap up through L1 Hyper-V, and under contention the
guest's *sense of time* jitters, so timer-driven boot waits balloon past the hang timeout. The
per-arch QEMU cap fixes correctness but not the structural cost, and does NOT cover build↔boot
contention (a 32-core `ninja` starving a concurrent boot's timing).

A Windows-host QEMU with `-accel whpx` (Windows Hypervisor Platform) runs as an **L1 sibling
partition on Hyper-V — not nested** → cheap exits + stable timing. That's the structural fix.

## What the spike PROVED (integration plumbing — the hard part)

Ran the *installed* Windows QEMU (`C:\Program Files\qemu\qemu-system-i386.exe`, v11.0.50) on our
real i386 `livecd.iso`, driven from WSL2 via interop. **End-to-end success:**

- **WSL→Windows path translation works.** Staged `livecd.iso` + `OVMF32_CODE.fd` + a writable
  `OVMF32_VARS` copy to `/mnt/c/temp/vibeos-spike` (437 MB ISO copy ≈ 12 s), launched QEMU with
  `C:/temp/...` forward-slash paths.
- **The OS actually boots on the Windows-host QEMU.** UEFI (OVMF32) → found the DVD → freeldr ran
  every init stage → `Switching stack...` → ntoskrnl → smss → csrss → win32k → **user32 usermode
  init** (mpr/user32 DllMain). 49 KB of serial — essentially the whole boot short of explorer.
- **Serial capture across the boundary works.** `-serial file:C:/temp/.../serial.log`, polled
  from WSL via `/mnt/c/...` — clean, complete stream. (TCP serial would be the cleaner production
  transport; file-poll is proven.)
- **Process control works.** Launch via WSL interop; `powershell.exe Stop-Process -Name
  qemu-system-i386 -Force` to kill. No stray procs after.

This was under **pure TCG** (slow) and it *still* reached usermode — so the plumbing is sound
independent of the accelerator. The accelerator is a one-flag swap.

## The ONE blocker (the speed payoff)

The installed QEMU build is **TCG-only — WHPX is not compiled in** (`-accel whpx: invalid
accelerator`; `-accel help` lists only `tcg`). To get the L1-acceleration win we need:

1. **A WHPX-capable Windows QEMU build.** The official builds at <https://qemu.weilnetz.de/>
   include WHPX. Install (admin): `winget install qemu` may pull a whpx build, or download the
   weilnetz installer. Verify with `qemu-system-i386.exe -accel help` → must list `whpx`.
2. **The "Windows Hypervisor Platform" optional feature enabled** (admin; separate from Hyper-V):
   `Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform` (couldn't query state
   in the spike — needs elevation). WHPX needs this even though Hyper-V is on for WSL2.

Both are **host/admin actions** the agent can't self-serve. Once done, re-run the exact spike with
`-accel whpx` and **measure boot time vs WSL-KVM, single and concurrent** — that number decides
whether the architecture work is worth it. (Caveat: WHPX is less per-exit-optimized than KVM, and
WHPX↔WSL2 coexistence should be confirmed; structurally it should still beat nested-KVM under
contention, but measure — don't assume.)

## Architecture sketch — dual-backend smoke runner (keep the simple cases simple)

Principle: **the default path stays exactly as today** (native `qemu-system-*` + KVM — works on
real Linux AND in WSL2-nested). Windows-host QEMU is an **opt-in backend** so a plain Linux/WSL
user gains nothing to configure.

Seam: a QEMU *backend* in `tests/qemu/vm.py` parameterizing the differences:

| concern | Linux/KVM (default) | Windows/WHPX (opt-in) |
|---|---|---|
| binary | `qemu-system-<arch>` (PATH) | `/mnt/c/Program Files/qemu/qemu-system-<arch>.exe` (interop) |
| media paths | native | stage to `/mnt/c/...`, pass `C:/...` (stage-once + reuse) |
| accel | `-accel kvm -cpu host` | `-accel whpx` (different `-cpu` handling) |
| serial | Unix socket (`serial_client`) | TCP (`-serial tcp:...`) — or proven file-poll fallback |
| QMP/monitor | Unix socket (`qmp_client`) | TCP (`-qmp tcp:127.0.0.1:PORT`) over WSL2 localhost |
| screenshot | QMP screendump → native path | screendump → Windows path, read via `/mnt/c` |
| process kill | PID/signal | `powershell Stop-Process` |

The bulk of the work is the **transport abstraction** (Unix-socket → TCP) in
`serial_client.py`/`qmp_client.py`; path-staging, accel flags, and process control are
straightforward. Select via `VIBEOS_QEMU_BACKEND=windows` / `--host-qemu`. arm64 stays
TCG-native (no accel either way).

## Decision / next step

- **Plumbing: proven feasible.** No architectural blocker.
- **Gate on a number, not faith:** install a WHPX QEMU + enable HypervisorPlatform (admin), re-run
  the spike with `-accel whpx`, and measure single + *concurrent* boot times vs WSL-KVM. If WHPX
  holds steady under concurrent boots (the whole point), build the dual backend; if not, the
  cheaper win is the harness **contention-discriminator + auto-retry-serialized** (kills the
  token-burn pain regardless of venue).
