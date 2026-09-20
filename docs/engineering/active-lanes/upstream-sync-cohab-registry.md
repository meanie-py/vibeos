# Cohab-flip registry (source-level approximation)

Replaces the "668 files / 1149 upstream edits" estimate, which was `<stem>.c` matched to
ANY `<stem>.rs` anywhere in the tree — a known-wrong upper bound (e.g. `shlwapi/utils.cpp`
matching an unrelated crate's `utils.rs`). Nobody should quote 668 again.

Registry: `tools/cohab-registry.json`. Generator: `tools/gen_cohab_registry.py` (read-only,
re-run any time — no build required, in particular safe to run before/after the upstream
merge).

## Headline number

**195** upstream-changed files hit a HIGH-confidence flipped TU in this registry. That is
the number that replaces 668. Derived with:

```
FP=5213cf717cff6946945922e6a7e03d87b0a58e4f
git diff --name-only $FP upstream/master   # 4519 files
```

intersected against the registry:

| confidence of the flipped TU | upstream-changed files hitting it |
|---|---|
| HIGH | **195** |
| MEDIUM | 94 |
| LOW | 2 |
| **HIGH+MEDIUM+LOW (everything the registry paired, however weakly)** | 291 |

Use 195 when you need a defensible number. 291 is the ceiling if you're willing to include
the MEDIUM/LOW tier's weaker pairings (see confidence definitions below) — still a fraction
of the old 668, and unlike 668 every entry in it has a citation you can check by hand.

## Counts, by confidence

Total cohab-flip candidates found: **1091**

| confidence | count | meaning |
|---|---|---|
| HIGH | 832 | An inline comment on the SAME LINE names the crate + `.rs` file, and that `.rs` file was confirmed to exist on disk — or a block-header comment (e.g. `# lpc/ — replaced by Rust (ntoskrnl-rust/src/lpc/)`) names a crate that exists, covering a following run of bare disabled entries. |
| MEDIUM | 255 | Either (a) an inline/block pairing as above but the specific `.rs` file could not be confirmed, or was confirmed but the same C source ALSO appears, uncommented, somewhere else in the build (flagged `also_compiled_uncommented: true` — the "disabled" premise is now in question), or (b) no textual evidence at all, paired only by guessing `<module-dir-name>-rust` and confirming that crate directory exists. |
| LOW | 4 | rust_crate/file NAMED but NOT found anywhere in that crate — the comment appears stale or wrong. Every entry with `rust_crate: null` was dropped entirely rather than kept as a fake LOW row (see "no rust signal" below) — an honest omission over an invented one. |

**LOW list (comment claims a `.rs` that a crate-wide search could not find — worth a human
look, possibly a stale comment or a rename that never got fixed):**
- `hal/halx86/generic/x86bios.c` → claims `hal-rust/src/x86/x86bios.rs`
- `ntoskrnl/config/i386/cmhardwr.c` → claims `ntoskrnl-rust/src/config/i386/cmhardwr.rs`
- `ntoskrnl/kd64/amd64/kdx64.c` → claims `ntoskrnl-rust/src/kd64/amd64/kdx64.rs`
- `ntoskrnl/kd64/i386/kdx86.c` → claims `ntoskrnl-rust/src/kd64/i386/kdx86.rs`

**"No rust signal" (158 commented-out `.c`/`.cpp` lines, NOT in the registry at all):** the
generator also finds ~158 ordinary disabled/unbuilt sources with zero nearby mention of
Rust — arch exclusions, missing third-party codecs (libtiff's `tif_jbig.c`/`tif_lzma.c`/
etc.), disabled experimental app code. These are deliberately excluded from the registry:
counting every commented-out `.c` in the tree as a "cohab flip candidate" is exactly the
kind of noise this tool exists to filter out. Re-derive the list with
`python3 -c "import sys; sys.path.insert(0,'tools'); import gen_cohab_registry as g; [print(w) for w in g.scan()['warnings']]"`.

## Method (re-derive with `python3 tools/gen_cohab_registry.py`)

1. Walk every `CMakeLists.txt` / `*.cmake`.
2. For each commented-out line matching a bare `<path>.c`/`.cpp` (optionally prefixed
   `${REACTOS_SOURCE_DIR}/`), require it to be either BARE (nothing after the path) or
   followed immediately by annotation punctuation (`(...)`, `— ...`, `-> `, `→ `). A path
   token that starts an ordinary prose sentence (`# smss_rust_stub.c was an empty TU...`)
   matches the same shape and is excluded by this gate — without it, that sentence alone
   inflated the count by hundreds of false HIGH/MEDIUM entries during development.
3. Resolve the path: `${REACTOS_SOURCE_DIR}/x` → repo-root-relative; a bare relative name →
   relative to the CMake file's OWN directory (CMake's own rule) — conflating the two
   produced a `ntoskrnl/ntoskrnl/...` doubled-prefix bug during development, fixed by
   tracking which prefix form matched, not by string-testing after the regex had already
   stripped it.
4. Find the Rust owner, in this preference order: (a) inline same-line annotation, (b) the
   most recent "block hint" comment in the same unbroken run of `#` lines (resets on a
   blank line or a non-comment line), (c) `<parent-dir-name>-rust` if that crate directory
   exists. Anything none of these can pair gets NO ENTRY (see "no rust signal" above), not
   a null-filled row.
5. Confirm the crate exists under `sdk/lib/rust/`, and try to confirm the named `.rs` file:
   first a literal join, then (because some crates like `ws2_32-rust` mirror the upstream
   module's own `src/` subdir a second level deep — `src/src/addrconv.rs`, not
   `src/addrconv.rs`) a basename search scoped to that one crate directory.
6. Cross-check "actually disabled": index every UNCOMMENTED `.c`/`.cpp` reference tree-wide
   by its FULLY RESOLVED path (not bare basename — `api.c`/`util.c`/`hash.c`/`event.c`/
   `rpcserver.c` are common enough across unrelated modules, e.g. dhcpcsvc vs advapi32 vs
   umpnpmgr, that a basename-only index gave 443 false "also compiled elsewhere" flags on
   fully-flipped modules during development; the full-path index gives 14, all real, e.g.
   `dll/win32/advapi32/reg/reg.c` really is separately compiled by
   `advapi32_vista/CMakeLists.txt` via a `../advapi32/reg/reg.c` relative reference).

## Blind spots (inherited from `verify_cohab_coverage.py`'s docstring — read it in full)

This is a **source-level approximation**, not the real oracle. The real oracle is the
OBJECT FILE (`llvm-nm --defined-only` on the compiled `.c.obj`, compared against everything
that links ahead of it), which needs a build this task was not allowed to run. Concretely,
this registry:

- **Cannot see a wrong-name flip.** A "paired" HIGH entry here does not prove the Rust side
  actually resolves the C symbols at link time. msvcrt's `math.c` shipped three symbols
  under the wrong name (`_rotl64`/`_rotr64` bare instead of `MSVCRT_` prefixed,
  `math_error` needing `-Dmath_error=_invoke_matherr`, `__stdcall` FDIV stubs needing
  `#[export_name = "...@4"]`) and none of that is visible from source text — the flip
  would have been a silent no-op and this tool would still call it HIGH.
- **Cannot see a sibling-file or C-shim provider.** A symbol a `.c` file needs can be
  defined by an unrelated file in the same archive (`cnt-rust`'s `cabi/math.rs`) or by a
  C shim compiled into the module target itself (`rust_file_varargs_shim.c`). This
  registry's pairing is "which `.rs` plausibly owns this `.c`", never "what actually
  resolves it at link time" — `verify_cohab_coverage.py --arch <a>` is the tool that answers
  that, once a build exists.
- **`file(GLOB ...)` source lists are invisible to this method entirely** — you cannot
  disable one entry with a comment in a glob-driven list, so a module built that way (if any
  do a cohab flip this way) is neither counted as flipped nor as unpaired; it's just not
  scanned.
- **A disabled `.c` with a stale/wrong Rust comment reads as paired.** The LOW list above
  is the direct evidence of this: the comment text is confident and specific and still
  wrong (or at least, not currently backed by an existing file).

## Re-deriving everything in this doc

```
python3 tools/gen_cohab_registry.py > tools/cohab-registry.json   # regenerate the registry
FP=5213cf717cff6946945922e6a7e03d87b0a58e4f
git diff --name-only $FP upstream/master > /tmp/upstream_changed.txt
python3 - <<'EOF'
import json
d = json.load(open("tools/cohab-registry.json"))
changed = set(l.strip() for l in open("/tmp/upstream_changed.txt"))
hits = {k: v for k, v in d.items() if k in changed}
from collections import Counter
print(Counter(e["confidence"] for e in hits.values()))
EOF
```
