# `vibeos-lint`: the source lint suite as one Rust binary

**Opened 2026-10-16** (user's call). Worktree `/home/curi/Projects/os-playground/vibeos-lint`,
branch `lint-rs`. Read `AGENTS.md` first: every rule there applies to this lane.

## The problem, measured

`./vibeos-cli check` runs **117 source checks**, almost all of them separate Python scripts
(`tools/scan_*.py`: 102 files, ~27K lines; plus `tools/verify_*.py`, `tools/rustconv/...`).
Each one is its own process, and almost each one walks, reads and re-tokenises the same
Rust/C tree. From the command logs of 87 recent runs (`vibeos-lint-timings.tsv` beside
this file has every scanner):

- Per-scanner medians sum to **~2,574 s (43 CPU-minutes) per full check**. With the
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

**Landing:** do not use `vibeos-cli land`. Message the **integration** session (see
`docs/engineering/active-lanes/integration.md` once it lands; `ListAgents` shows it) with
`merge lint-rs @ <sha> — <what>`, and keep working.

## Measure as you go

Record in this doc, per batch: rules ported, Rust wall time for all ported rules, the Python
time they replaced (from the TSV), and the full `check` wall time before and after. The
TSV was produced from `/tmp/vibeos-cli/*-check-*/commands.jsonl` (`stage` = `lint:<name>`,
`seconds`); regenerate it the same way.

## Inventory (fill in)

| Scanner | Ported? | Parity (py / rs findings) | Notes / reason left out |
|---|---|---|---|
| | | | |
