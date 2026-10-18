# `vibeos-lint`: the source lint suite as one Rust binary

**Opened 2026-10-16** (user's call). Worktree `/home/curi/Projects/os-playground/vibeos-lint`,
branch `lint-rs`. Read `AGENTS.md` first: every rule there applies to this lane.

## The problem, measured

`./vibeos-cli check` currently registers **120 source checks**, including four added
on master during this rewrite. The original suite registered 116 checks; its historical
logs contain 117 scanner names. Almost all began as separate Python scripts
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

The final coverage batch expands the original source-only scope at the user's request:
all source-registry checks run through Rust plugins, including the checks that invoke
external tools or inspect `build-<arch>/`. Shellcheck, clang, binutils and the Python
compiler remain their respective authorities; Rust owns their invocation and check logic.
Post-link `run_binary_lints` and `check-tooling` remain separate commands; the final artifact cutover below ports the post-link checker implementations too.

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
  `rayon`, `memchr`, `aho-corasick`, `syn` 2, `serde`/`serde_json`. Tree-sitter and
  `yaml-rust` were approved separately; the user subsequently authorized introducing
  dependencies as needed for the remaining tooling migration.
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
| `scan_atl_slot_cast` | Yes | 0 / 0; older 0 / 0 | Final batch: native scanner controls. |
| `scan_fnclass_cc` | Yes | 0 / 0; older 4 / 4 | Final batch: tracked header and KMDF class authority; native controls. |
| `scan_cmake_linked_rust` | Yes | 25 / 25 current | Final batch: full read-only cargo-cell model, three-arch fixtures; reads build graphs/PE files. |
| `scan_list_walk_cursor_after_loop` | Yes | 0 / 0; older 1 / 1 | Final batch: native scanner controls. |
| `shellcheck` | Yes | 8 / 8 on both trees | Rust driver invokes the existing shellcheck binary; missing-tool coverage stays strict. |
| `py-syntax` | Yes | 0 / 0 on both trees | Rust driver invokes the CLI Python interpreter as the syntax compiler. |
| `verify_ffi` | Yes | 157 / 157; older 157 / 157 | Batch 7: exact parity on both trees; Rust fixtures. |
| `verify_constants` | Yes | 0 / 0; older 0 / 0 | Batch 6: exact parity on both trees; Rust fixtures. |
| `verify_constants_enums` | Yes | 3 / 3; older 3 / 3 | Batch 6: exact parity on both trees; Rust fixtures. |
| `lint_kernel` | Yes | 1229 / 1229; older 1231 / 1231 | Batch 7: exact parity on both trees; Rust fixtures. |
| `scan_nightly_surface` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `strip_msvc_cmake` | Yes | 0 / 0; older 0 / 0 | Batch 6: exact parity on both trees; Rust fixtures. |
| `shell32_tail_inventory` | Yes | 0 / 0; older 0 / 0 | Batch 6: exact parity on both trees; Rust fixtures. |
| `verify_tables` | Yes | 3 / 3; older 3 / 3 | Batch 8: exact parity on both trees; Rust fixtures. |
| `verify_match_consts` | Yes | 0 / 0; older 0 / 0 | Batch 8: exact parity on both trees; Rust fixtures. |
| `verify_signatures` | Yes | 15 / 15 current | Native parser/AST/Unicode fixtures; configured headers required. |
| `verify_conv_macros` | Yes | 0 / 0; older 0 / 0 | Batch 6: exact parity on both trees; Rust fixtures. |
| `verify_ucrtbase_cohab` | Yes | 0 / 0 on both trees | Rust graph/archive/takeover/import checks; Python reporting helpers retained. |
| `vendored_registry` | Yes | 0 / 0 on both trees | Approved YAML parser; metadata/marker snapshot and exact notices regeneration. |
| `verify_translation_literals` | Yes | 130 / 130; older 130 / 130 | Batch 6: exact parity on both trees; Rust fixtures. |
| `gen_ctype_tables` | Yes | 0 / 0 on both trees | Native derivation and object proof; Python --emit generator retained. |
| `verify_lost_exports` | Yes | 0 / 0; older 0 / 0 | Batch 7: exact parity on both trees; Rust fixtures. |
| `verify_c_signatures` | Yes | 10 / 10; older 10 / 10 | Batch 6: exact parity on both trees; Rust fixtures. |
| `scan_inline_flag_literals` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_extern_arity` | Yes | 13 / 13 | Exact on both source revisions; batch 3. |
| `scan_orphan_modules` | Yes | 6 / 6 (current); 6 / 6 (older) | Batch 4; exact finding hashes on both trees. |
| `cohab_census` | Yes | 0 / 0; older 0 / 0 | Batch 7: exact parity on both trees; Rust fixtures. |
| `scan_bugcheck_stubs` | Yes | 3 / 3 (current); 3 / 3 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_silent_failures` | Yes | 1798 / 1798; older 1805 / 1805 | Batch 6: exact parity on both trees; Rust fixtures. |
| `scan_magic_offsets` | Yes | 75 / 75; older 75 / 75 | Batch 6: exact parity on both trees; Rust fixtures. |
| `scan_doc_sections` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_boot_media_drift` | Yes | 0 / 0; older 0 / 0 | Batch 7: exact parity on both trees; Rust fixtures. |
| `scan_raw_uart_writes` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_bugcheck_codes` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_assert_to_ub` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_bitwise_not_bool` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_variadic_array_arg` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_ptr_byte_diff` | Yes | 0 / 0 | Exact on both source revisions; batch 3. |
| `scan_misaligned_cast` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_dropped_null_guard` | Yes | 0 / 0 | Exact on both source revisions; batch 3. |
| `scan_store_on_failure` | Yes | 0 / 0 | Exact on both source revisions; batch 3. |
| `scan_bool_width` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_cfg_vs_c` | Yes | 0 / 0; older 0 / 0 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_constant_placement` | Yes | 0 / 0 | Exact on both source revisions; batch 3. |
| `scan_const_contradictions` | Yes | 209 / 209; older 209 / 209 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_const_vs_island` | Yes | 1 / 1; older 1 / 1 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_const_vs_own_island` | Yes | 0 / 0; older 0 / 0 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_const_within_crate` | Yes | 37 / 37; older 37 / 37 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_invented_constants` | Yes | 8 / 8; older 8 / 8 | Batch 6: exact parity on both trees; Rust fixtures. |
| `scan_hresult_literals` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_byteptr_stride` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_gmem_flags` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_let_shadow` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_raw_bitfield` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_winerror_consts` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_case_bitor_pattern` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_list_walk_from_head` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_dowhile_continue` | Yes | 55 / 55 on both revisions | Rust plugin; Python helper retained for rustconv.c_body_checks. |
| `scan_mask_drift` | Yes | 47 / 47; older 48 / 48 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_dropped_prints` | Yes | 1427 / 1427 (current); 1442 / 1442 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_dropped_probes` | Yes | 2 / 2 (current); 2 / 2 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_ptr_typedef_indirection` | Yes | 0 / 0; older 0 / 0 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_dropped_seh_pairs` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_raising_probes` | Yes | 0 / 0 | Exact on both source revisions; batch 3. |
| `scan_raw_user_access` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_double_fetch` | Yes | 3 / 3 on both revisions | Rust plugin; Python helper retained for scan_safe_probe_deref. |
| `scan_safe_probe_deref` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_macro_fn_shadow` | Yes | 0 / 0 (current); 30 / 30 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_type_alias_contradictions` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_varargs_shim_parity` | Yes | 0 / 0; older 0 / 0 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_wide_literals` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_dangling_transmute` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_opaque_ptr_arith` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_fnptr_abi_transmute` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_cdecl_alias` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_wide_byte_copy` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_typed_byte_copy` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_noop_goto` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_formatless_printf` | Yes | 0 / 0 | Exact on both source revisions; batch 3. |
| `scan_byte_step_one` | Yes | 4 / 4 | Exact on both source revisions; batch 2. |
| `scan_ptr_typedef_cast_read` | Yes | 0 / 0 | Exact on both source revisions; batch 3. |
| `scan_info_class_literals` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_const_narrowing_cast` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_comment_swallowed_code` | Yes | 0 / 0 | Exact on both source revisions; batch 3. |
| `scan_call_arg_swap` | Yes | 3 / 3 | Exact on both source revisions; batch 3. |
| `scan_narrow_vs_wide_literal` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_sse_helpers` | Yes | 0 / 0 | Exact on both source revisions; batch 3. |
| `scan_read_after_clear` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_init_then_move` | Yes | 0 / 0 on both revisions | Rust plugin; positive/negative controls exercise the shipping binary. |
| `scan_containing_record_null` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_call_literal_vs_c` | Yes | 139 / 139; older 139 / 139 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_pack_drift` | Yes | 3 / 3 (current); 2 / 2 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_unaligned_wide` | Yes | 0 / 0; older 0 / 0 | Batch 5: exact parity on both trees; Rust fixtures. |
| `verify_svc_numbering` | Yes | 0 / 0 on both trees | Rust driver invokes objdump; table/stub controls cover unavailable builds. |
| `marshalgen_audit` | Yes | 0 / 0 on both trees | Source-derived tables and clang layout checks; stale artifact/positive divergence controls. |
| `scan_cc_mismatch` | Yes | 14 / 14 (current); 14 / 14 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_rawdylib_linkname` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_import_cc` | Yes | 6 / 6 (current); 4 / 4 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_assert_polarity` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_fourcc_literals` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_match_arm_break` | Yes | 19 / 19 | Exact on both source revisions; batch 2. |
| `scan_calloc_elem_type` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_varargs_spec_arity` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_octal_literals` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_ptr_highword_mask` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_arg_side_effects` | Yes | 5 / 5 (current); 5 / 5 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_res_named_types` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_bindgen_padding_index` | Yes | 0 / 0 | Exact on both source revisions; batch 2. |
| `scan_extern_static_width` | Yes | 0 / 0 (current); 0 / 0 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_duplicate_symbols` | Yes | 0 / 0; older 0 / 0 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_export_shadows` | Yes | 38 / 38; older 38 / 38 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_irql_balance` | Yes | 9 / 9; older 9 / 9 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_ob_refcount` | Yes | 8 / 8; older 8 / 8 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_com_double_release` | Yes | 0 / 0; older 0 / 0 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_unpruned_walk` | Yes | 0 / 0; older 0 / 0 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_extern_decl_drift` | Yes | 31 / 31; older 31 / 31 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_dropped_asserts` | Yes | 89 / 89 (current); 89 / 89 (older) | Batch 4; exact finding hashes on both trees. |
| `scan_override_self_dispatch` | Yes | 0 / 0; older 36 / 36 | Batch 5: exact parity on both trees; Rust fixtures. |
| `scan_version_vs_c` | Yes | 0 / 0; older 12 / 12 | Batch 5: exact parity on both trees; Rust fixtures. |
| `null_offset_to_offset_of` | Yes | 0 / 0; older 15 / 15 | Batch 7: exact parity on both trees; Rust fixtures. |

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

The full rewrite remains owed: 78 registry entries still need ports, and five
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

Syncing current master adds `scan_override_self_dispatch`, `scan_version_vs_c` and
`null_offset_to_offset_of`, bringing the registry to 116 checks. All three remain
Python entries tracked in the inventory; 33 Rust plugins and five exclusions leave
78 source checks owed. The tooling merge preserves the new version-scanner tests
and keeps the retired scanner tests removed.

Post-merge validation: all **116 source stages** pass in 256.12 seconds
(`20261016-233006-xh6a60u7-check-all`), while full tooling ran concurrently.
All **141 tooling checks** pass (`.cache/vibeos-lint/check-tooling-batch02-merged.log`).
A subsequent warm run of all 33 Rust rules over the merged tree yields 81 findings
in **7.61 seconds wall / 46.11 seconds user CPU**, with 2.58 GiB peak RSS.


## Batch 3 — shared C twins and module reachability (2026-10-17)

Eleven more plugins bring the binary to **44 rules**: constant placement, dropped
null guards, stores on failure, pointer byte differences, extern arity, raising
probes, swapped call arguments, SSE helpers, swallowed comment code, formatless
printfs, and pointer typedef reads. They replace **374.6 seconds** of summed
historical scanner medians; the 44-rule total is 1,491.1 seconds. **67 source
checks remain owed**, alongside the five explicit exclusions.

The shared snapshot now includes tracked C/C++ sources, Cargo manifests and
rule-declared auxiliary headers. C definitions, local callee lookups, Rust
function bodies and crate reachability are shared across plugins. Cross-file
indexes initialize before rule fan-out to avoid worker-pool deadlock; a shipping
binary fixture exercises one, two and four workers. Dynamic regex compilation
is bounded and cached, and suppression comments are indexed once per file.

`parity/batch-03.json` compares the original Python rules at `c442ed9fc6db`
against both that source revision and `cc53d01b091`: **16 findings on each**,
with identical hashes and no differences. Thirteen are extern-arity findings
and three are swapped-call-argument findings. The complete 44-rule run yields
97 findings. No baseline changed. The original eleven scanner scripts and two
dedicated pytest files are retired; the shared Python C/module/placement helpers
remain for their other callers and the translation pipeline.

The module walker was also checked independently on the full current tree:
for each Cargo manifest under `sdk/lib/rust`, compare the crate directory plus
every resolved reachable file from Python `crate_live_files` and Rust
`ModuleTree::live_files`, as sorted sets of repository-relative paths (absolute
for targets outside the repository). Both produced **7,362 entries**, with zero
differences. Rust fixtures retain nested modules, path attributes, includes,
opaque generated includes and symlink identities.

A warm 44-rule run under concurrent Python parity load took **18.04 seconds
wall / 108.39 seconds user CPU**, peaking at **3.95 GiB RSS**. This is an expanded
corpus and rule set, not a controlled comparison against batch 2's 7.61-second
33-rule run. The new cross-file roots remain candidates for further tuning.
The Rust suite passes **197 tests**, and all **186 scanner-control/runner tests**
pass. The full **116-stage source check passes in 103.65 seconds**
(`20261017-003306-ledhjs7v-check-all`); its shared 44-rule invocation took 33.47
seconds under concurrent scanner load. The preceding merged batch's full source
check took 256.12 seconds under concurrent tooling load; these are observed runs
under different load, not a controlled speedup. Full tooling validation is
recorded with the batch commit. No OS source changed, so this batch needs no OS
build or smoke run.


## Batch 4 — syntax, catalogs and cross-file conventions (2026-10-17)

Twenty-seven more rules have exact Python parity on both comparison trees. The CLI
now routes **71 checks** through the binary; **41 source checks remain owed**,
alongside four exclusions. The committed resource-object rule is included because
its input bytes can be captured in the shared snapshot.

`parity/batch-04.json` records **1,555 findings on the current tree and 1,597
on `cc53d01b091`**, with identical Python/Rust hashes for every rule. The oracle
is the original registry and helper code at `e5ffbfc1e578`. The current report
combines a full run with targeted reruns after corrections; the older report
combines 26 rules with a separate inline-constant run. These component runs are
parity evidence, not a combined performance benchmark.

The new rules replace **362.7 seconds** of summed historical scanner medians.
The user approved pinned tree-sitter crates (0.25.2 and Rust grammar 0.24.1)
to preserve syntax-based decisions. Header catalog ordering, i386 directory
exclusions, signed constants, Unicode masking and multiline wide arrays retain
the original scanners' behavior. No baseline or OS source changed.

Python scanner entrypoints and their dedicated pytest files are retired. The
translation pipeline keeps pure case-pattern helpers in `tools/case_patterns.py`;
the separate declaration fixer remains in `tools/fix_cc_mismatch.py`, with its
write-path controls. Those utilities do not run as source lint stages.

Additional candidate plugins and shared compiler/history/module facts are
checkpointed on this branch, but are not routed until their own parity is proven.
The **116-stage source check passed** in 101.60 seconds
(`20261017-025615-jf_k4_7w-check-all`), and **all 124 tooling checks passed**
(`.cache/vibeos-lint/check-tooling-checkpoint06.log`). The Rust suite passes
**486 unit tests and two integration tests**; all **186 scanner-control/runner
tests** pass. Early-stage fixtures now run the Rust plugins and observe native
file opens through Linux inotify, including a positive test that deliberately
opens forbidden build output; all eight early-stage tests pass.

A subsequent warm 71-rule run took **101.55 seconds wall / 260.16 seconds user
CPU**, with **9.76 GiB peak RSS**, under concurrent parity and fleet build load.
The source snapshot took 10.78 seconds. The seconds target remains unfinished:
syntax retention, repeated scans and cross-file indexes still need tuning.
These measurements are not a controlled comparison with earlier batches.
Profiling already removed repeated prefix newline counts in dropped-assert
checking; later candidate optimizations remain separate work.

## Batches 5–8 — source-suite coverage (2026-10-17)

The binary now routes **108 of 116 checks**. The vendor-registry audit remains
owed; its Rust core is checkpointed but YAML decoding still needs approval for
an additional dependency. Seven external checks remain in Python: shellcheck,
compileall, clang signatures, built UCRT coverage, built ctype extraction, built
SVC numbering, and clang marshalling layouts. The latter three were previously
marked owed; their actual registry commands read build outputs or invoke compiler
tools, so they fall under the design's existing exclusions.

The committed batch reports record exact Python/Rust finding sets on this tree
and `cc53d01b091`: batch 5 has **519 / 568** findings across 19 rules; batch 6 has
**2,024 / 2,031** across ten; batch 7 has **1,386 / 1,403** across six; batch 8 has
**3 / 3** across two. These 37 rules replace more than **564 seconds** of summed
historical scanner medians. Counts and hashes come from the original registry,
argv, streams and helpers at `e5ffbfc1e578`, with Python fact caches disabled.
Reports assembled from reruns are parity evidence, not combined benchmarks.

Retired Python lint entrypoints are removed. Separate write/report APIs remain:
`strip_msvc_cmake.py --write`, `null_offset_to_offset_of.py --apply`, cohab census,
`rustconv.fix_constants --fix`, and `rustconv.shell32_inventory`. The translation
pipeline retains pure loop checks in `tools/loop_patterns.py`. Their dedicated
utility tests remain; source controls exercise the shipping Rust plugins.

Cross-file source sets now declare standalone Cargo-feature inputs. Scoped
fixtures retain global version catalogs, and the early invented-constant check
prunes build directories before opening them. Bootstrap publication checks its
source digest again after compilation; a concurrent source edit cannot publish
a binary under the wrong digest. Finding chunks are normalized and tagged one
physical line at a time. The replay harness validates historical Python commands
before doing expensive work and handles nested Rustconv script roots explicitly.

The 89-rule source check passed all 116 stages in **359.86 seconds**
(`20261017-032249-edypn1un-check-all`). The 107-rule check also passed all stages
in **541.95 seconds** (`20261017-033612-heutrck0-check-all`), under concurrent
parity and fleet load. These runs demonstrate baseline compatibility; they do
not demonstrate a speedup. The seconds target remains owed. Profiling found
repeated whole-corpus path maps in crate scanning and serialized syntax parsing;
performance changes and full tooling validation follow this checkpoint.

## Performance checkpoint 1 (2026-10-17)

All 116 source stages passed in **360.88 seconds**
(`20261017-041258-44ci78_h-check-all`); the shared Rust invocation took
337.84 seconds, including its startup work. All **112 tooling checks**, **507 Rust
unit tests**, and **two integration tests** passed. Concurrent fleet/tooling load
makes this an observed runtime, not a controlled speedup comparison.

The corpus now builds exact and normalized path indexes once. Link scans reuse
those indexes and run outside the global cache lock. Export checks reject
non-kernel names before inspecting source suppressions; suppression lookups use
the line index. Duplicate-symbol architecture tracking uses compact masks instead
of cloning string sets per character, and memoizes condition results. Fixed,
fully pinned conditions evaluate without cloning the feature map. Syntax walking
uses cursors for wide child lists. Mask variants initialize independently, IRQL
syntax parsing follows relevant call frontiers, and StoreOnFailure memoizes C
success analysis. Immutable Git history caches are keyed by repository identity,
full HEAD and schema, atomically published, and recomputed when invalid.

`tools/vibeos-lint/parity/performance-01.json` records exact original-Python
findings on both source revisions for the changed cross-file analyses, including
38 export shadows, nine IRQL findings and zero duplicate-symbol/store findings
per tree. Component timings are separate replays. The seconds target remains
owed: this run still spent 248.1 seconds in extern-declaration drift, 197.7 in
invented constants, and about 133 each in enum and match-constant checks. The next
work is reducing repeated header/name parsing and regex-cache contention.

## Performance checkpoint 2 (2026-10-17)

The next frozen tree passed all **116 source stages** in **467.84 seconds**
(`20261017-042159-2gyhdvtp-check-all`) and all **112 tooling checks**. Its Rust
invocation took 358.67 seconds; bootstrap delayed its start by 108.24 seconds.
Two parity replays and tooling ran concurrently, so these observations are not
a controlled speed comparison. The historical Python replay is still running
at this checkpoint.

Regex lookups now have a bounded thread-local front cache, while fixed constant
parser patterns retain their search state. Constant-universe and glob resolution
reuse per-file names and Cargo dependency indexes. Version checks index and cache
directory overrides; invented constants harvest common SDK headers once per run.
Extern drift indexes sibling attributes and inherited foreign-block context
instead of repeatedly looking up previous siblings and parents.

The remaining measured hotspots in this run were constant placement (242.7s),
extern drift (192.0s), and invented constants (114.0s). Follow-up candidates reduce
symbol-by-symbol body scans, per-node syntax cursor allocations, repeated SDK
expression evaluation, and serial cold-history queries. The seconds target
remains owed.

## Performance checkpoint 3 (2026-10-17)

The second checkpoint's six changed rules now have exact two-tree Python parity:
**42 / 54 findings**, recorded in `parity/performance-02.json`. The same report
proves the new constant-placement identifier scan on both trees (zero findings).
Its separate Rust replays took **14.44 / 16.92 seconds**. Non-identifier symbolic
facts, including invalid C octal spellings, retain their original exact searches.

This checkpoint passed **116 source stages** in **283.51 seconds**
(`20261017-043324-8ci736i8-check-all`), **112 tooling checks**, **512 Rust unit
tests** and **two integration tests**. Bootstrap delayed the shared invocation by
78.65 seconds; that invocation took 204.13 seconds under concurrent replay and
tooling load. This remains an observed runtime rather than a controlled speedup.

Extern drift now collects nodes, sibling attributes and foreign-block context
with one cursor. Invented constants retain successful common-header evaluations
in a shared catalog and retry unresolved sites after adding module definitions.
Cold history queries use eight bounded workers; timings separately expose that
shared initialization phase. Historical replay for the cursor/evaluation changes
is still running at this checkpoint. Extern drift remains the largest measured
rule (162.64s), followed by IRQL (68.34s) and match constants (59.61s); independent
per-file extern fact extraction is the next candidate. The seconds target and
vendor YAML integration remain owed.

## Performance checkpoint 4 (2026-10-17)

The cursor/evaluation checkpoint is now parity-proven on both trees: **39 / 51
findings** across three rules. The bounded parallel extern extraction and shared
preorder traversal also match exactly: **1,524 / 1,540 findings** across six rules.
`parity/performance-03.json` records the finding hashes and finalized measurements.

Extern facts are extracted in at most eight chunks on the existing Rayon pool,
then merged in original file order. Set facts union; vector facts retain their
append order. IRQL, object-reference, dropped-probe, dropped-print and mask checks
reuse a single-cursor preorder iterator that preserves unnamed nodes and subtree
boundaries. These changes passed **513 unit tests**, **two integration tests**,
all **112 tooling checks**, and all **116 source stages**.

The overlapping source run took **243.70 seconds**
(`20261017-044112-1ogfr654-check-all`), including 49.47 seconds before the shared
invocation started. A subsequent warm run with no other lint/tooling jobs from
this lane passed in **89.77 seconds**, with **89.56 seconds** in the shared binary
(`20261017-044724-qflcws0k-check-all`). Other fleet activity continued; this is an
observed warm runtime, not a controlled comparison with historical Python runs.
The report preserves each run's corpus file/byte counts because the inputs differ.

The single-digit target remains owed. The warm run's largest rules were extern
drift (72.44s), IRQL (38.39s), match constants (33.19s), and double fetch (32.14s).
Vendor YAML decoding is the only remaining source port, pending its separate
dependency decision; the seven external checks retain their documented exclusions.

## Master reconciliation (2026-10-17)

Merged `996c6ff8957615baccfd3a51286b1f8d7ec27c9d` from `origin/master`.
The Rust plugins include master's standard-library IRQL call exclusion (`6cab479`),
CRT return authority (`c56aab4`), dead-branch constant decisions (`55d5f39`), and
same-module C-twin weak-write-probe pass (`b126827`). Their new regression fixtures
are retained in native controls or ported to Rust, including the CRT kernel skip.
Every master control-test function survives the union. All four new scanner
commands remain registered in Python; the source registry now has 120 checks,
108 routed to Rust. The four changed Python scanners were retired again only after exact replay against
master `996c6ff` on both this merged tree and `cc53d01b091`: **34 / 58 findings**
across the four rules. `parity/master-sync-996c6ff.json` preserves the counts and
hashes. All 522 Rust unit tests and two integration tests passed; the focused
IRQL/new-scanner controls passed eight tests. Full source/tooling gates follow
the retirement.

The first merged source gate exposed a fifth drift: `verify_conv_macros` had copied
`KW_TO_CONV`, so it missed master's `MI_CALL` and `MI_MAIN_CALL`. The binary now
embeds the canonical `rustconv/conventions.py` table, and bootstrap fingerprints
that extra compile-time input. Unsupported table syntax fails loudly. Regression
tests cover alias additions/changes and bootstrap invalidation both before and
during a build. The original four-rule replay remains exact. Convention replay against the same
master oracle also matches on both trees (zero findings); its reports are included
in `parity/master-sync-996c6ff.json`. Three focused bootstrap regression tests
passed. Both full gates are rerun on the retired implementations and corrected catalog.

## Complete source-registry cutover (2026-10-17)

All **120 source checks** now have registered Rust plugins and are routed through
one shared invocation by `./vibeos-cli check`. The final twelve include the three
new source scanners, vendor registry, and all eight formerly excluded external/build
checks. Shellcheck, clang/binutils and Python's compiler remain external authorities;
no standalone Python scanner command runs in the source registry. Missing shellcheck
and an empty shell input set retain their original coverage verdicts.

`parity/final-coverage.json` records the method and limits. Against master `996c6ff`,
all twelve match on this tree (**48 findings**); ten also match on `cc53d01b091`
(**13 findings**). The older tree has no configured headers/build graph, so signature
and cargo-cell checks additionally use native parser and three-arch model fixtures.
The latter compare accepted/refused labels with the real converter survey on the same
synthetic build inputs. Build-dependent SVC and marshalling success on these trees is
an original skip, not evidence of an executed disassembly/layout probe; separate
controls exercise divergence, repair and stale-table failure.

The signature port fixes Unicode character windows and crate-wide deduplication.
Vendor parity caught form feeds in GPL grants: notice rendering now follows Python's
full line-break semantics, with a byte-exact committed-notices control. Vendor grant
policy lives once in `tools/vendor-grants.json`; cargo-cell engine dictionaries and
calling-convention aliases are embedded from their canonical tables. Bootstrap
fingerprints all three inputs and rejects a build whose inputs changed underneath it.

Six standalone Python scanners and the dedicated signature pytest file are retired.
Writer/report libraries remain for vendored notices, ctype generation, UCRT burndown
and marshalling generation; their old lint CLI modes now direct callers to the native
front door. The cargo-cell ratchet tests and all scanner controls exercise the shipping
binary. A registry test requires every source stage to belong to the native routed set,
so a future Python scanner cannot silently reopen the split. External/registry
plugins declare their own inputs rather than requiring an unrelated default Rust
workspace. The vendor fixture driver uses its temporary registry as the repository;
project-specific tolerated grants apply only to project checkouts carrying the
canonical vendored writer. Follow-up vendor-only parity remains exact on both trees.
Native no-SDK fixtures also prove missing-build skips and a guest syscall dispatched
to its neighbor failing loudly.

The parity subsets ran concurrently: **92.69 seconds** for the current twelve and
**71.92 seconds** for the older ten. These are coverage checks, not a controlled full-suite
performance comparison. Performance tuning and the single-digit target remain next.
Full source/tooling gates run after retirement; their artifact paths are recorded in
the checkpoint commit message. No push or landing is performed by this lane.

## Artifact and standalone checks (final linter cutover)

The binary now also owns the post-link suite, cohab coverage, committed-artifact drift,
layout-output comparisons, i386 compiler ABI tripwire, optional wide-byte-string and
bind-all checks, LLVM IR audits, and ARM64 unwind verification. Gate and the smoke static
assertions route to it. `./vibeos-cli lint --list-rules` lists the artifact/standalone
plugins; source baseline checks remain the existing 120-stage registry. See
`tools/vibeos-lint/README.md` for invocation and ownership boundaries.

Artifact records carry each check's stdout, stderr and exit code in a validated JSON envelope.
The Python bridge refuses incomplete or malformed coverage. Disassembly/symbol extraction
is content-addressed against input/tool/implementation bytes and relevant environment,
locked across invocations, and revalidated before publishing. Records are compressed with
zstd; raw extraction text is released between plugins. Baselines and allowlists are read
again on every invocation, including cache hits. SHA-256 and zstd dependencies were added
under the user's subsequent blanket dependency approval.

The first nine artifact checks matched historical Python/shell stdout, stderr and exits
exactly on the same existing amd64 build tree (`parity/binary-amd64.json`). This establishes
scanner parity, not build freshness or boot coverage. Positive/negative fixtures cover the
other architectures and incomplete inputs. `lint_parity.py` can replay retired scanner and
helper bytes from Git; binary-cache implementation fingerprints are materialized from those
same historical bytes while preserving the original path identities; the original
cache verifies binary/tool content and record integrity before reuse.

Generators remain Python authorities for now, as requested for the next migration. They
emit into temporary output directories or return read-only regeneration recipes; Rust owns
the drift comparison. Migration targeting reports (`scan_arch_cfg`, sibling drift, probe
leads) and the warning census's reporting interface are also separate from lint verdicts.
The native bind-all audit deliberately refuses any failed Cargo check instead of accepting
partial diagnostics; it uses stable force-warn flags without `RUSTC_BOOTSTRAP`.

Performance tuning remains owed. The historical Python timings and the source-suite target
above remain the reference; successful migration or warm cache parity is not proof of a
single-digit source suite.

Regroup before starting the next tooling migration. The user's order is migration/reporting
utilities, code generators, build infrastructure, QEMU/testing infrastructure, hardware/payload
tools, `vibeos-cli`, and finally C-to-Rust tooling.


## Performance checkpoint 5: vendor audit and CLI lifetime (2026-10-17)

Vendor directory lookups now use the existing sorted inventory instead of scanning every
file for every directory. Fixed prose-license regexes compile once, and the filesystem walk
uses directory-entry types, checking symlink targets only when necessary. The first selected
comparison dropped from 119.95s to 9.81s with identical empty output. A later current-tree
observation was 2.23s (2.33s external wall), including 1.29s to inventory 62,360 files. Cache
state and the master merge differ between observations; these are component measurements.

Extern drift retains a large bindgen tree across both existing extraction passes. Original
Python at `996c6ff8957` still matches exactly: 23 extern findings and zero vendor findings.
The one-shot source CLI also leaves its read-only corpus for OS reclamation at process exit,
avoiding a destructor walk through potentially swapped-out cached trees and source views.
Library callers retain normal ownership and cleanup. `TIMING source_rules` now measures the
aggregate execution phase, separately from snapshot loading and process teardown.

Both source gates passed all 120 stages with the same 5,638 finding lines and SHA-256. The
first overlapping native run took 453.7s; a subsequent run without other lint/tooling jobs
from this lane took 182.9s: 29.49s snapshot loading and 150.37s rule execution. Build/history
caches differ and fleet activity was not controlled, so this is not a controlled speedup.
`parity/performance-04.json` preserves the method, hashes, timings and Python replay.
The single-digit target remains owed; retained source-view/AST memory and repeated fact
extraction are the next performance work.


## Performance checkpoint 6: bounded syntax and cheaper snapshots (2026-10-17)

Master `91640c25fb1` is reconciled, including its linear vendor directory index and Rust
scanner fixes. The source suite preserves all 5,550 finding lines byte for byte across
the reconciled baseline and every cache experiment (`parity/performance-05.json`).

The default source pool now caps workers at eight. A shared weighted LRU retains at most
four million Rust syntax nodes, admits a file after its second request, and keeps sources
over 256 KiB weakly cached as before. Active callers retain their Arcs through eviction;
library ownership and cleanup remain normal. The small already-cached `lru-slab` dependency
provides constant-time recency tracking. This bounds retention, not total process memory.
Raw bytes share decoded text when identical (528.2 MB of extra raw storage becomes 1.8 MB);
CRLF and invalid UTF-8 still preserve original bytes. The lexer masks its character buffer
in place, matching the old implementation on 20,000 generated inputs.

Traversal rejects irrelevant names before root checks and carries only eligible source sets
into each subtree, preserving each set's pruning and explicit-input rules. Extern drift
extracts island facts once, keeps only relevant AST nodes, and reads lines from the shared
index with exactly `str::lines()` semantics. New timings separate inventory, materialization,
shared indexes, parallel rule execution and optional cache statistics. Statistics inspect
initialized metadata without parsing or walking the compiler's entire JSON AST.

Sequential old/new native scans on the same checkout observed 152.90s / 149.13s wall,
568.99s / 556.92s user+system CPU, and 14.40 / 9.76 GiB maximum RSS. The final normal source
gate observed 109.6s in the native process: 2.19s inventory, 1.75s materialization, 7.84s
shared initialization and 97.26s parallel rules. Host activity was not controlled, so the
paired results establish a memory improvement and similar observed latency, not a universal
speedup. Extern drift remains the largest rule (80.81s in that final gate). The single-digit
whole-suite target remains owed; further CPU and scheduling work follows this checkpoint.


## Performance checkpoint 7: transient allocations (2026-10-17)

Merged master `50c7c7907f2`, including the compile-command cache, unsupported-regex,
dangling-symlink and formatting fixes. Raw-user-access now borrows cached function facts
and source lines instead of copying their bodies and raw line strings. Masked views share
an allocation when an absent cutoff marker and ASCII character/byte masking give identical
input and output; Unicode and real cutoffs retain their distinct behavior.

The CLI uses `mimalloc` 0.1.52. Its Linux C allocator override covers tree-sitter too;
the global allocator declaration lives only in the executable. `--no-default-features`
builds retain the system allocator for comparisons. The new dependency is authorized by
the user's general dependency approval. Usage and override support come from the
[allocator crate](https://docs.rs/mimalloc/0.1.52/mimalloc/).

The merged baseline observed 9.02 GiB maximum RSS / 121.30s. Rust allocations alone with
mimalloc observed 8.22 GiB / 106.72s. With borrowing/view sharing, the system-allocator
source gate observed 8.08 GiB (169s including compilation); the native full-override
experiment observed 6.97 GiB / 110.83s. Every completed scan emitted the same 5,840 tagged
finding lines. A two-arena glibc experiment was rejected: 7.83 GiB / 487.46s.
Host activity and build caches were not controlled; these are observations, not a universal
speedup or a complete allocation attribution. The retained corpus and live rule working
sets still consume several GiB. `parity/performance-06.json` records methods and evidence;
the single-digit latency target and further memory reduction remain open.

The final default-allocator source gate passed all 123 stages with the same finding hash
at 6.81 GiB peak RSS / 121.22s including bootstrap, alongside 692 Rust tests and all 106
tooling checks. That is about 25% less peak RSS than the observed merged baseline, with
similar elapsed time. Inspection of ELF symbols confirms the executable overrides malloc,
calloc, realloc and free; the library test executable has no allocator override symbols.
