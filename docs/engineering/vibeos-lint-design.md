# `vibeos-lint`: the source lint suite as one Rust binary

**Opened 2026-10-16** (user's call). Worktree `/home/curi/Projects/os-playground/vibeos-lint`,
branch `lint-rs`. Read `AGENTS.md` first: every rule there applies to this lane.

## The problem, measured

`./vibeos-cli check` currently registers **113 source checks**; the historical logs
contain 117 scanner names. Almost all began as separate Python scripts
(`tools/scan_*.py`: 102 files, ~27K lines; plus `tools/verify_*.py`, `tools/rustconv/...`).
Each one is its own process, and almost each one walks, reads and re-tokenises the same
Rust/C tree. From the command logs of 87 recent runs (`vibeos-lint-timings.tsv` beside
this file has every scanner):

- Per-scanner medians sum to **~2,574 s (43 scanner-minutes) per full check**. With the
  scanners run concurrently that is ~170 s of wall time (p90 ~370 s) on a 24-core host.
- The worst single scanners are 1.5-2.5 minutes each: `scan_list_walk_from_head` 156 s,
  `scan_variadic_array_arg` 120 s, `scan_dowhile_continue` 100 s,
  `scan_constant_placement` 92 s, `scan_ptr_typedef_indirection` 90 s,
  `scan_double_fetch` 84 s, `scan_let_shadow` 76 s.
- A single landing runs the suite up to four times (the lane's `check`, `land` on the tip
  and on the merged tree, the gate), across a fleet of ~10 lanes.
- Two earlier patches (`ceb43db14c6` one shared pruning walk, `b0bdd67316e` a cached scanner)
  did not change the shape; the suite kept growing. The fix is structural.

## Goal

One host binary, `vibeos-lint`, that walks the tree **once**, reads and tokenises each file
**once**, and runs every ported rule over that shared representation in parallel.
Target: **the whole source suite in seconds** (single digits on this host), so the check a
lane runs before landing and the check the gate runs are the same fast command.

Non-goals for now: the post-link binary lints (`run_binary_lints`: scan_ud2,
scan_flex_array_panic, … read built objects), `check-tooling` (ruff/pyright/pytest), and
checks that drive external tools (`shellcheck`, `py-syntax`/compileall, anything that runs
clang, bindgen or cargo, or reads `build-<arch>/`). Those keep running as today; list each
you leave out, with the reason, in this doc's inventory.

## The contract it must keep (do not change behaviour)

The current system, so the switch is invisible to every lane:

- **Registry:** `tests/qemu/cli/lints.py::source_lint_specs` — `(name, cmd,
  findings_from)` for every check. `source_partition_specs` splits them into `source-early`
  / `source-deferred` for the gate; `EARLY_SOURCE_CHECKS` names the early set.
- **Findings** are the scanner's output lines (`_finding_lines`: non-empty, minus lines
  matching `_SUMMARY_LINE`). Some read stdout only, some stdout+stderr (`findings_from`).
- **Baselines:** `tools/lint-baseline/<name>.txt`, one accepted finding line per line. The
  gate fails only on NEW lines (`_lint_stage`). A rule's output lines must therefore be
  **byte-identical** to the Python scanner's, or every baseline turns into fresh findings.
- **Verdicts:** a crash or a nonzero exit with no finding lines is `SCANNER_ERROR`
  (`_scanner_error`), never a pass. Plain `check` fails on a NEW finding; `--strict` adds
  coverage and per-arch guards (`debugcmd.check`).
- **Source corpus:** `tools/source_walk.py` (`source_files()`, the pruning walk: never enter
  `target/`, build output, etc.). Reproduce its include/exclude decisions exactly.
- **Tests:** `tools/test_scan_*.py` (48 files) hold each scanner's fixtures and expectations.

## Architecture

- **Crate:** `tools/vibeos-lint/` as its own Cargo workspace (like `tools/host-jobs`), built
  for the host with the pinned stable toolchain (`rust-toolchain.toml`). Build it on demand
  the way `tests/qemu/cli/hostjobs.py::ensure` builds host-jobs: into
  `.cache/vibeos-lint/`, keyed by a digest of its sources, under a lock. Commit
  `Cargo.lock`; build `--offline`. Available offline in `~/.cargo/registry`: `regex`,
  `rayon`, `memchr`, `aho-corasick`, `syn` 2, `serde`/`serde_json`. Nothing else without
  asking.
- **One pass:** walk once (same pruning as `source_walk.py`), read each file once, and build
  a per-file representation that rules share: raw bytes, line index, and for `.rs` a
  comment/string-masked view (most Python scanners strip comments and strings themselves,
  each slightly differently: find the variants and keep each rule's own semantics).
  Rules that need a real parse can use `syn` on demand; cache the parse per file.
- **Rules:** one module per ported scanner, a `Rule` trait over the shared file set,
  run with `rayon`. Cross-file rules (a rule that indexes the whole tree, then checks) get a
  two-phase interface: collect, then report.
- **Output:** `vibeos-lint --rules a,b,c` (or `--all-ported`) prints each finding as
  `<rule>\t<finding line>` in a deterministic order, exactly the Python line after the tab.
  Exit 0 on success whether or not there are findings (baselines decide); nonzero only for
  an internal error, with a `FATAL:` line, which `_scanner_error` already classifies.
- **Wiring:** `lints.py` runs the binary **once** for all ported rules, splits the tagged
  output per rule, and feeds each rule's lines through the existing `_lint_stage` (same
  stage names, same baselines, same strict/new-finding logic). Unported rules keep their
  Python command. `source_lint_specs` stays the single registry; a ported entry just points
  at the shared run. `vibeos-cli check` and the gate pick it up with no other change.

## Porting rule-by-rule, proven by parity

For each scanner, in **cost order** (top of `vibeos-lint-timings.tsv` first; the top 15 are
over half the total):

1. Port it as a Rust rule. Preserve behaviour exactly, including quirks. A rule that is
   wrong is fixed in a separate, deliberate change after parity, with its baseline updated
   in the same commit, never as a side effect of the port.
2. Port its fixtures from `tools/test_scan_<name>.py` into Rust tests.
3. **Parity on the real tree:** a small harness (`tools/lint_parity.py`, yours to write)
   runs the Python scanner and the Rust rule on this worktree and diffs the finding sets.
   Zero diff is required. Run it on at least one other commit too (`git worktree add
   --detach` a scratch tree at an older master, remove it after).
4. Switch the registry entry to the shared run, delete the Python scanner and its pytest
   file, and keep the parity result in the commit message (counts on both sides).

Land in batches of ported rules; each batch has to pass the full `./vibeos-cli check` and
`./vibeos-cli check-tooling` with identical baselines.

**Handoff:** commit each verified batch on `lint-rs` and give the user its hash and branch.
The user relays it to the Claude integration session. Do not push or call `vibeos-cli land`.

## Measure as you go

Record in this doc, per batch: rules ported, Rust wall time for all ported rules, the Python
time they replaced (from the TSV), and the full `check` wall time before and after. The
TSV was produced from `/tmp/vibeos-cli/*-check-*/commands.jsonl` (`stage` = `lint:<name>`,
`seconds`); regenerate it the same way.

## Inventory

| Scanner | Ported? | Parity (py / rs findings) | Notes / reason left out |
|---|---|---|---|
| `shellcheck` | No | — | Excluded: invokes shellcheck. |
| `py-syntax` | No | — | Excluded: invokes Python compileall. |
| `verify_ffi` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_constants` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_constants_enums` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `lint_kernel` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_nightly_surface` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `strip_msvc_cmake` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `shell32_tail_inventory` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_tables` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_match_consts` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_signatures` | No | — | Excluded: invokes clang against this checkout’s generated build headers. |
| `verify_conv_macros` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_ucrtbase_cohab` | No | — | Excluded: reads built archives/DLLs with nm/objdump. |
| `vendored_registry` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_translation_literals` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `gen_ctype_tables` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_lost_exports` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_c_signatures` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_inline_flag_literals` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_extern_arity` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_orphan_modules` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `cohab_census` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_bugcheck_stubs` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_silent_failures` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_magic_offsets` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_doc_sections` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_boot_media_drift` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_raw_uart_writes` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_bugcheck_codes` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_assert_to_ub` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_bitwise_not_bool` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_variadic_array_arg` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_ptr_byte_diff` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_misaligned_cast` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_dropped_null_guard` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_store_on_failure` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_bool_width` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_cfg_vs_c` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_constant_placement` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_const_contradictions` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_const_vs_island` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_const_vs_own_island` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_const_within_crate` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_invented_constants` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_hresult_literals` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_byteptr_stride` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_gmem_flags` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_let_shadow` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_raw_bitfield` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_winerror_consts` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_case_bitor_pattern` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_list_walk_from_head` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_dowhile_continue` | Yes | 55 / 55 on both revisions | Rust plugin; Python helper retained for rustconv.c_body_checks. |
| `scan_mask_drift` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_dropped_prints` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_dropped_probes` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_ptr_typedef_indirection` | No | — | Owed: shared CMake compiled-TU/module pairing and tree-sitter definition equivalence. |
| `scan_dropped_seh_pairs` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_raising_probes` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_raw_user_access` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_double_fetch` | Yes | 3 / 3 on both revisions | Rust plugin; Python helper retained for scan_safe_probe_deref. |
| `scan_safe_probe_deref` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_macro_fn_shadow` | No | — | Owed: tracked header/spec inventory and tree-sitter definition equivalence. |
| `scan_type_alias_contradictions` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_varargs_shim_parity` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_wide_literals` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_dangling_transmute` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_opaque_ptr_arith` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_fnptr_abi_transmute` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_cdecl_alias` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_wide_byte_copy` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_typed_byte_copy` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_noop_goto` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_formatless_printf` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_byte_step_one` | Yes | 4 / 4 | Exact on both source revisions; batch 2. |
| `scan_ptr_typedef_cast_read` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_info_class_literals` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_const_narrowing_cast` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_comment_swallowed_code` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_call_arg_swap` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_narrow_vs_wide_literal` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_sse_helpers` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_read_after_clear` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_init_then_move` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_containing_record_null` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_call_literal_vs_c` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_pack_drift` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_unaligned_wide` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `verify_svc_numbering` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `marshalgen_audit` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_cc_mismatch` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_rawdylib_linkname` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_import_cc` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_assert_polarity` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_fourcc_literals` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_match_arm_break` | Yes | 19 / 19 | Exact on both source revisions; batch 2. |
| `scan_calloc_elem_type` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_varargs_spec_arity` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_octal_literals` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_ptr_highword_mask` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_arg_side_effects` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_res_named_types` | No | — | Excluded: reads committed PE-COFF resource objects, not source text. |
| `scan_bindgen_padding_index` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_extern_static_width` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_duplicate_symbols` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_export_shadows` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_irql_balance` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_ob_refcount` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_com_double_release` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_unpruned_walk` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_extern_decl_drift` | No | — | Owed: remains in the Python registry until ported and parity-proven. |
| `scan_dropped_asserts` | No | — | Owed: remains in the Python registry until ported and parity-proven. |


## Batch 1 — host runner and twelve rules (2026-10-16)

The first batch installs the stable, offline Cargo workspace, locked on-demand
bootstrap, shared immutable source snapshot and line index, cached lexical views,
parallel rule execution, tagged output, and one shared invocation in both source
checks and baseline capture. The registry still has all 113 original stage names;
every unported command and findings stream is unchanged. Early/deferred selection
still applies before execution. Scanner/bootstrap/protocol failures fail every
selected Rust stage; no partial output is accepted.

The twelve plugins replace **861.1 seconds of summed historical per-scanner medians**
from the TSV (33.5% of its 2,573.6-second total). That sum is scanner elapsed time,
not a measurement of CPU time or the parallel suite’s wall time. A warm execution
of all twelve, including snapshot loading, took **5.75 seconds wall / 34.12 seconds
user CPU**, with about 2.0 GiB peak RSS. Under simultaneous full-check, tooling and
Python parity load the same set took 30–38 seconds; these are observed runs, not
a single-digit guarantee under arbitrary fleet contention.

All twelve match the original Python finding sets byte-for-byte on the lane’s
source revision `34d3d7e1941` and older revision `cc53d01b091` (271 changed `.rs` files among 297 workspace
files between them). Each yields 58 unique findings: 55 do-while findings and
3 double-fetch findings, zero from the other ten. No baseline was changed.
The committed parity report records counts, finding-set SHA-256 digests and
Python elapsed times; `tools/lint_parity.py` replays the original scanner bytes
from git against either source tree without restoring deleted scanners.

Ten independent Python scanners and four corresponding pytest files are removed.
`scan_dowhile_continue.py` and `scan_double_fetch.py` remain import libraries for
the paid translation audit and `scan_safe_probe_deref`, respectively; source lint
execution for both is Rust. Their library tests remain. The shipping Rust plugins
also run the original positive/negative scanner controls, and `check-tooling` runs
the Cargo tests.

After batch 1, 96 registry entries awaited ports, and five external
or binary checks remain intentionally outside this source-only binary. The next
shared roots are tracked C/header inventory, C-twin/function indexing, and CMake
module pairing; the pointer-typedef and macro-shadow rules keep using Python until
those contracts have faithful replacements. Those roots remain owed; batch 2 below adds further source rules.

Validation: full `vibeos-cli check` passed all 113 stages in 103 seconds
(run `20261016-224400-kmksrwzt-check-all`); `check-tooling` passed all 150 checks
(including 57 Rust tests). The initial full source check took 352 seconds under
concurrent parity/tooling load and failed only because this fresh lane lacked
`build-amd64` headers. The amd64 build provisioned those headers and produced
`build-amd64/livecd.iso`. The full-suite historical wall median was about 170
seconds; these differently loaded runs are not a controlled before/after wall
comparison. No OS boot-path source changed, so no smoke run was required for
this host-tool batch.

## Batch 2 — twenty-one more rules (2026-10-16)

Thirty-three rules now run in the host binary. The 21 new rules replace 255.4 seconds
of summed historical scanner medians (1,116.5 seconds across all 33). These are
scanner elapsed times, not CPU measurements. This batch adds byte/element-size,
allocation-flag, bitfield, pointer, literal, alias and control-flow rules listed above.
Input scopes preserve the original workspace Rust, SDK Rust and C/assembly walks;
overlapping files are read once. Pointer-mask and UART rules share a cached Rust
lexer, including raw strings, nested comments and lifetimes.

The 21 new rules exactly match the original Python finding sets on the lane and
`cc53d01b091`: four byte-step findings, 19 match-arm findings, zero for the other
19. No baseline changed. `parity/batch-02.json` records counts and digests.
The parity harness now uses the original registry's arguments and streams, restores
Python's script-directory import path, loads imported tool helpers from the oracle
revision too, and disables persistent Python fact caches.
A warm run of all 33 rules took **6.82 seconds wall / 48.33 seconds user CPU**,
with 2.58 GiB peak RSS. This includes the shared source snapshot.
Twenty independent Python scanners and ten pytest files are removed; their fixtures
run in Rust. `scan_wide_byte_copy.py` remains a helper library for unported rules.

The full rewrite remains owed: 75 registry entries still need ports, and five
external/binary checks remain excluded. C-twin/function indexing and CMake pairing
are still needed by the remaining cross-language rules.

Validation: all 134 Rust tests and 31 selected CLI controls pass. Full source check
passed all 113 stages (`20261016-232120-v3e5p7hs-check-all`), and full tooling check
passed all 140 checks (`.cache/vibeos-lint/check-tooling-batch02.log`). The source
check took 208.59 seconds while two uncached Python parity suites and tooling ran
concurrently, versus batch 1's 103.84-second run under different load; this is not
a controlled before/after suite timing. Both final parity reports cover all 33
plugins and yield the same 81 findings: the prior 58 plus this batch's 23.
No baseline or OS source changed; no OS build or smoke was needed for this batch.
