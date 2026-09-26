# Vendored FEX-Emu snapshot (vibeos)

**Upstream:** https://github.com/FEX-Emu/FEX, tag **FEX-2607**
(commit `1cc4b93e7a71c883ec021b71359f136394dc1f3c`, snapshot 2026-07-08). **License: MIT** (`LICENSE`).

**Unmodified.** No vibeos patches are applied to this tree — it is a pruned
byte-identical snapshot of the upstream tag. If a local change ever becomes
unavoidable, record it here as a diff and mark the touched files.

The one addition is `vendor/fex/VENDORED`, a generated marker written by
`tools/vendored.py --write-stamps` (see `vendor.yaml`). It adds a file rather than
touching one, so every upstream byte is still upstream's; a re-vendor drops it and
`--write-stamps` puts it back.

**Pruned** (build-irrelevant for the vibeos wow64/arm64ec targets — huge test
corpora and gated-off externals; see `tools/fex/build-fex.sh` for the exact
configure that this subset was proven against):

- `External/fex-gvisor-tests-bins` (727M), `External/fex-posixtest-bins` (38M),
  `External/fex-gcc-target-tests-bins` (33M) — test binaries
- `External/vixl` (412M) — replaced by FEX's own CodeEmitter; only needed for
  `ENABLE_VIXL_DISASSEMBLER`/`ENABLE_VIXL_SIMULATOR` (both OFF)
- `External/tracy`, `External/Vulkan-Headers`, `External/zydis`,
  `External/Catch2`, `External/jemalloc_glibc`, `External/drm-headers`,
  `unittests/` — profiler / thunks / disassembler / tests / glibc allocator,
  all gated off in our configure
- `.git*`

**Build:** `tools/fex/build-fex.sh` (out-of-band — deliberately NOT part of the
ReactOS cmake graph; see `docs/engineering/fex-bringup-plan.md` rung 1).
Products land in `<builddir>/Bin/`:

- `libwow64fex.dll` — the 32-bit-x86-guest WoW64 BTCpu plugin
  (imports **ntdll.dll + wow64.dll only**)
- `FEXOfflineCompiler.exe` — standalone JIT driver (imports KERNEL32/ADVAPI32 +
  UCRT api-sets → needs `ucrtbase.dll` + apiset resolution on vibeos)

To re-vendor a newer tag: shallow-clone upstream with submodules, re-apply the
prune list above, re-run the build proof, update this file.
