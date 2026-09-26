# qemu-perf — make the boot/smoke cycle fast

**Lane:** `qemu-perf` (worktree `vibeos-qemu-perf`), opened 2026-09-19.
**Scope:** the QEMU execution layer only — backend, accelerator, media attach, boot
concurrency. Not the guest. A guest bug found here gets handed to the lane that owns it.

## Execution update

See [measured results](../qemu-perf-results.md) for this lane's implementation and
validation. The original handoff below is retained as the starting hypothesis.
Corrections established during execution:

- `smoke --repeat N` runs sequentially; farm runs one worker per architecture.
- WHPX was resolved **after** admission, so its intended per-arch cap was bypassed.
  The lane resolves it first and normalizes net/UEFI variants to the same cap.
- TCG was faster in the desktop trials. WHPX remains opt-in; the media cache and
  Windows cleanup are default behavior for every Windows harness launch.
- Windows Event 1000 and minidumps confirm the all-setup-errors incident was a
  QEMU process crash. Later WHPX runs reached 13/15: listeners announced, but
  the two inbound data tests timed out. That differs from the older spike's
  missing-listener diagnosis.

## The situation you are inheriting

**KVM inside WSL2 is not worth using.** Nested virt under WSL2 contends badly enough that
it lost to running QEMU on the Windows host outright. So the default is *not* local QEMU:
`tests/qemu/cli/config.py` sets `qemu.backend = "windows-host"`, which launches
`/mnt/c/Program Files/qemu/qemu-system-*.exe` through WSL interop, stages media to a
Windows-visible dir (`/mnt/c/temp/vibeos-qemu`), and carries QMP + serial over TCP
(WSL listens, QEMU.exe connects back to `127.0.0.1`). The recorded comparison in that
file is **~1m35s windows-host vs ~6min contended native** — treat as inherited, not
re-measured.

Consequence worth internalising: **the guest runs off-host.** Media is staged, not shared;
paths are `C:/...` inside the QEMU argv; the serial and QMP transports are sockets over the
WSL/Windows boundary. Several problems in this area are *transport* problems wearing a
guest costume (see the open bug below).

### Two accelerators, and the default is the slow one

| | |
|---|---|
| `tcg` | pure emulation. **The current default.** No shared accelerator. |
| `whpx` | Windows Hypervisor Platform — real hardware virt. **x86 only.** Opt-in. |

Flip it with `VIBEOS_QEMU_WINDOWS_ACCEL=whpx`. arm64 always resolves to TCG (`backend.resolve_accel`) — WHPX cannot run an aarch64 guest, so arm64 is
permanently out of scope for the accelerator work.

**WHPX is available on this box** (`backend.whpx_available()` → `True`, verified
2026-09-19). It is opt-in not because it fails to boot, but because it never earned
gate-grade repeat reliability. The bring-up war story — and it is a *long* one, six stacked
guest-side blockers behind the AHCI switch — is
[`../windows-host-qemu-whpx-spike.md`](../windows-host-qemu-whpx-spike.md) and field manual
§9.46. **Read the spike doc before touching `backend.py`**; most of the obvious ideas in
this area have already been tried and have scars.

## Measured, 2026-09-19, idle box (24 CPU / 47 GB), build cap 3x6

Today's gate, the default **windows-host + TCG**:

| stage | wall |
|---|---|
| `smoke:i386` | 78.6s |
| `smoke:amd64` | 54.9s |
| `smoke:arm64` | 93.9s |
| `smoke:net` | 99.8s |

One **WHPX** amd64 run tonight (`VIBEOS_QEMU_WINDOWS_ACCEL=whpx ./vibeos-cli smoke --arch
amd64`, run dir `/tmp/vibeos-cli/20260919-010159-uxowgsfu-smoke-amd64`):

- argv confirms it really ran hardware virt: `-accel whpx,kernel-irqchip=on`, and the CD
  attached via `ich9-ahci` + `ide-cd` as `rewrite_accel` intends.
- **Boot reached `[EXPLORER] _tWinMain entered`** — the real desktop marker. WHPX boots.
- Verdict still FAIL, on the net phase (below).

**No honest boot-time A/B exists yet, and I did not fabricate one.** The 76.3s total for
that run includes a net suite that died at 17.8s, and the gate's `smoke:amd64` is a
different stage shape from a manual `smoke`. **Task 1 is a clean A/B** — same tree, boot
phase only, several repeats, TCG vs WHPX, i386 and amd64.

## The tension that makes this non-obvious

A faster boot is not automatically a faster gate.

`vm.py:per_arch_cap` serializes **same-arch** boots to 1 for backends that share an
accelerator — native+KVM and **windows-host+WHPX**. The current TCG default shares nothing,
so `_acquire_arch_slot()` early-returns *before* the cap and same-arch boots run in
parallel. Turning WHPX on therefore **re-arms serialization**.

So a 3x faster boot that serializes could be net-slower for exactly the workloads we care
most about: `gate`, `smoke --repeat N`, and `farm`. Any recommendation to flip the default
has to be measured at the *gate/farm* level, not per boot. Quantifying that trade is the
core question of this lane.

## Open bugs in this area

1. **Net suite under WHPX regressed below its documented state.** The spike doc records
   13/15 with two inbound-listen failures. Tonight all **15 errored in 17.8s, at fixture
   setup**, with `ConnectionError: Serial socket closed`
   (`tests/qemu/serial_client.py:219`). That is the transport dropping, not fifteen guest
   failures — a different and probably easier bug than the documented one. Classified
   `NET_FLAKE`. Start here; it is cheap and it currently blocks any WHPX gate story.
2. **WHPX inbound-listen (documented, still open).** Guest-side TDI/echo listeners never
   announce under WHPX (`MARK_ECHO_READY` absent, `LISTEN did not complete 0xc00000b5`)
   while the same tests pass on TCG. Suspect the netstack listen/accept bridge against
   amd64 APIC timing under `kernel-irqchip=on`. Guest-side — hand off if confirmed.
3. **Stale doc pointer.** `config.py` and `vm.py` both cite
   `docs/engineering/windows-host-qemu-spike.md`, which **does not exist**. The real file is
   `windows-host-qemu-whpx-spike.md`. Fix the references or add the missing doc.

## Where the code is

| | |
|---|---|
| `tests/qemu/backend.py` (301 ln) | argv rewriting, `resolve_accel`, `rewrite_accel`, `windows_qemu_bin`, `whpx_available`, staging, uid-scoped kill |
| `tests/qemu/vm.py` | `QemuVM`, `per_arch_cap`, `_acquire_arch_slot`, timeouts, gdbstub wiring, guestfwd echo servers |
| `tests/qemu/cli/config.py` | the 3-layer default/file/env config; backend + accel defaults live here |
| `qemu/README.md`, `qemu/lib/common.sh` | the shell path and `qemu_per_arch_cap`, which `per_arch_cap` mirrors |

Boot concurrency is *also* bounded by the host broker's separate `bootcpu` pool (8 leases,
`./vibeos-cli host-status`), which is independent of the build cap — don't confuse the two
when reading contention.

## Suggested order

1. Clean TCG-vs-WHPX boot A/B, i386 + amd64, repeats, one tree. Publish numbers.
2. Fix bug 1 (serial socket under WHPX) — it gates everything else.
3. Measure at **gate and farm** level with the serialization cap armed. This is the number
   that decides the default, not the per-boot number.
4. Only then propose flipping `qemu.windows.accel`. It is a fleet-wide default; land it
   like the build-capacity trial — measured, reversible, documented.

## House rules that bite here specifically

- **Never broad-kill QEMUs** (`pkill -f qemu` matches your own ancestors and has killed
  agents mid-run). Use `./vibeos-cli kill` — it refuses your own ancestors.
- **Don't switch the smoke backend to native KVM to "repro faster."** It hits the per-arch
  cap and just queues.
- **Never weaken a smoke test or lower a boot marker.** The marker is the product.
- An empty taskbar in the screenshot is **not** a failure — the shot is captured *at* the
  marker, before the tray renders. Judge by `verdict` + duration.
