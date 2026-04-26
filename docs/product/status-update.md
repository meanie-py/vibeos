
# ntoskrnl Translation Status (updated 2026-04-26)

## Smoke Test Status

| Arch | Result | Notes |
|------|--------|-------|
| **i386** | **7/7 PASS** | Full graphical desktop with start menu/taskbar |
| **amd64** | **7/7 PASS** | Explorer regression fixed today by fstub wide-string fix |
| **ARM64** | 4/7 | 2 skipped (objdump path), 1 failing — `[SMSS] _main` never fires (silent failure inside `LdrInitializeThunk` or CRT init) |

ARM64 reaches `[PS] PspUserThreadStartup: returning` reliably (kernel + user-mode dispatch work after this session's MMPTE soft-PTE bit-layout fix), but `_main` doesn't fire — likely a separate user-mode loader bug.

## Boot-Milestone Harness (new this session)

`tests/qemu/test_smoke.py::test_milestones` validates that each arch's `required_milestones` list (per `tests/qemu/vm.py`) appears in serial output. Catches silent regressions where `shell ready` fires without explorer actually starting (the amd64 case earlier this session).

User-mode markers added:
- `[SMSS] _main entered` (`base/system/smss/smss.c`)
- `[CSRSS] _main entered` (`subsystems/csr/csrss/csrss.c`)
- `[WINLOGON] WinMain entered` (existed already)
- `[EXPLORER] _tWinMain entered` (`base/shell/explorer/explorer.cpp`)
- Plus existing kernel `[KERNEL] KiSystemStartup entered`, `IoInitSystem returning TRUE`

## Translation Coverage (this session)

### FSTUB — fully Rust on all 3 arches (this session)

| File | C lines | Rust lines | Status |
|------|--------:|-----------:|--------|
| disksup.rs | 2,758 | 6,020 | per-function rustconv translate + subagent + Gemini cleanup |
| fstubex.rs | 2,566 | 4,641 | per-function rustconv translate + subagent + Gemini cleanup |
| halstub.rs | 307 | 969 | hand-rewritten by subagent (HAL_DISPATCH global init) |
| translate.rs | 186 | 430 | hand-rewritten by subagent (CM_PARTIAL_RESOURCE_DESCRIPTOR) |

**Net: 5,817 lines C → 12,060 lines Rust.** All 4 files in build, all arches green.

The wide-string bug found here — `L"\\Device\\Harddisk%d\\Partition0"` translated as bytes-cast-to-`*const u16` produced garbage paths every other character — was a translation pattern that's likely present in other subsystems too. Worth a project-wide grep.

### IOMGR & PNPMGR — unchanged from prior status

(See git history for prior session work — iofunc.rs, arcname.rs, devaction.rs, arb/*.rs all complete.)

## Key Fixes This Session

- **ARM64 BUGCHECK 0x6A fixed** — `MMPTE_SOFTWARE.Protection` is at bit 1 on ARM64 (not bit 5 like amd64/i386). Rust `soft_protection()` accessor was hardcoded to shift=5; demand-zero PTEs never matched the check. ARM64 SMSS now gets dispatched to user mode for the first time since March.
- **amd64 explorer regression fixed** (incidental) — fstub wide-string bug above. `shell ready` was firing from winlogon's own desktop creation, masking that explorer never started. New milestone harness caught it; the wide-string fix resolved it.
- **fstub fully Rust** — disksup + fstubex + halstub + translate. Bulk-translate-then-subagent-cleanup workflow validated; per-function `rustconv translate` (vs `submit`) avoids token-truncation issues that affect large files.
- **README rewritten** to be accurate per-arch (no more "all 3 boot to desktop" overclaim).
- **AGENTS.md** strengthened with translation philosophy: never revert .rs to C, never delete .rs files, never re-run paid translations without approval, never exclude from build to make smoke pass.

## What's Next (priority order)

1. **ARM64 SMSS reaches `_main`** — debug why `LdrInitializeThunk` or CRT init silently fails. Highest-leverage if we want ARM64 desktop back.
2. **Translate config/** (~19K lines, registry CM) — last big C subsystem in ntoskrnl. Per-function translate + subagent cleanup pattern proven this session.
3. **Translate fstub/'s upstream patterns project-wide** — search for `*const u16` bytes-cast-to-wide bugs in other already-translated `.rs` files.
4. **pnpmgr on i386** — extern "C" → extern "system" audit, ~36 blocks across 12 files. Mechanical.

## Open Architectural Issues

- **CGU=1 on i386/amd64** — ~15 unexpected ud2s due to LLVM interprocedural noreturn contagion. Workaround `codegen-units=16` + `opaque_ptr` barriers on 2,449 fns shipped, root not yet fixed.
- **kd64/ disabled** — translated but disabled because of recursive exception flood on amd64 (`KdpTrap → KdpReport → KdSendPacket` infinite nesting). Needs the bug debugged, not retranslated.
- **`--allow-multiple-definition` linker flag** — masks dual C/Rust symbol definitions (caused multiple "wait, which version is being linked?" debug sessions this month including the trap.rs / trap.c shadow bug). Should be removed once final C holdouts are translated.
