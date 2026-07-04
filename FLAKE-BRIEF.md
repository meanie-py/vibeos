# Flake: `arm64_lpcexitthread_race` — PspExitThread → LpcExitThread exit race (arm64)

**Worktree branch:** `flake-arm64-lpcexit` · base `master` @ 4d84eee378d

## ⚠️ Read first — overlap with the nettest flake
An arm64 nettest wedge prints BOTH `[WINLOGON] nettest.exe created` AND
`LpcExitThread`, so it matches `nettest_explorer_race` FIRST (listed earlier in
`tests/qemu/cli/triage.py`) — not this entry. **This entry is the residual arm64
exit-path hang with NO nettest marker present.** Much of what looked like this
flake is actually the nettest race. Recommend coordinating with the
`flake-nettest-explorer` worktree: if that lands first, re-measure how often a
*pure* `LpcExitThread` hang (no nettest) still reproduces — it may shrink a lot.

## Signature
- **when:** hang. serial CONTAINS `LpcExitThread`, LACKS `_tWinMain entered`
  (and, for the pure form, no `nettest.exe created`). arm64 only.

## Root cause hypothesis
`PspExitThread → LpcExitThread` system-thread-exit / timer race on arm64 — a
thread exiting through the LPC exit path races something (timer/scheduler/LPC
port teardown). Intermittent on mainline.

## Repro
```bash
./vibeos-cli build --arch arm64
./vibeos-cli smoke --arch arm64 --repeat 8
# GDB path (arm64 debug):
RUN=$(qemu/start.sh arm64 --gdb)
qemu/wait-for.sh $RUN --pattern "shell ready" --timeout 540   # or catch the hang
```
arm64 TCG boots are slow (~9min timeout) — expect long iterations.

## Pointers
- `docs/engineering/archive/arm64-clock-and-lpcexitthread-findings.md` — the
  historical investigation of this exact race (periodic-clock work that made it
  deterministic, plus the exit-path analysis). Best starting context.
- `docs/engineering/translation-debugging-field-manual.md` §9.
- Retire the `arm64_lpcexitthread_race` entry in `triage.py` once fixed + re-gated.

## Land discipline
Gate smoke ×3 (all arches) before landing. arm64 boot firmware
(`qemu/QEMU_EFI.fd`, `qemu/efivars.fd`) is already copied into this worktree;
`boot.img`/`ramdisk.img` are rebuilt by the build. Never revert Rust→C.
