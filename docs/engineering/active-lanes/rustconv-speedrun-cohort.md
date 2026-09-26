# rustconv scaling cohort — 2026-09-23

This round adds an offline benchmark and integrates one compiler-translated Wine
translation unit. It made no model calls. The existing DeepSeek pilot ledger remains
$0.17093736 against the $5 cap; costs are conservative configured-price estimates.

## Results and interpretation

Twelve source files are split 6/6 between development and held-out evaluation.
Development passed all six rows. Held-out passed four, rejected one and left one
unsupported. Both measurements are valid; the held-out acceptance verdict is false.
The score is **eight unchanged candidates in this evaluation, one automatic
normalization, one manually repaired candidate, one rejection, one unsupported**.
It is not an original model first-pass rate: most C candidates were already integrated,
and their historical repair cost and effort are unknown. The corpus is deliberately
small and selected. Harness construction inspected held-out bytes, so it is not blinded.

| Source | Outcome | Scope |
|---|---|---|
| `dll/win32/jscript/decode.c` | automatic_repair | decode_source finite host corpus; guest ABI assessed separately |
| `dll/win32/shell32/CCopyAsPathMenu.cpp` | unsupported | DoCopyAsPath unsupported host grade |
| `dll/win32/shell32/CEnumIDListBase.cpp` | rejected | Skip/Reset parity only; Next rejected; other methods unsupported |
| `dll/win32/shell32/utils.cpp` | untouched_success | SheRemoveQuotesA/W only, not whole utils.cpp |
| `sdk/lib/crt/math/arm64/fmod.c` | untouched_success | selected exported functions on a finite host corpus |
| `sdk/lib/crt/mem/memmove.c` | untouched_success | selected exported functions on a finite host corpus |
| `sdk/lib/crt/stdlib/qsort.c` | untouched_success | selected exported functions on a finite host corpus |
| `sdk/lib/crt/string/strtoi64.c` | untouched_success | selected exported functions on a finite host corpus |
| `sdk/lib/cryptlib/rc4.c` | untouched_success | rc4_init and rc4_crypt finite host corpus |
| `sdk/lib/rtl/dos8dot3.c` | untouched_success | selected exported functions on a finite host corpus |
| `sdk/lib/rtl/utf8.c` | untouched_success | selected exported functions on a finite host corpus |
| `sdk/tools/txt2nls/main.cpp` | manual_repair | whole host tool on finite codepage corpus |

The final run compares JScript on 349 inputs, RC4 on 268, quote helpers on 173,
CEnum Skip/Reset on 83, txt2nls on 129 whole-tool inputs, and its bounded C++
stdlib helper on 372 inputs. A separate root run adds 256 stdlib cases.
Existing CRT/RTL harnesses report test-function counts, not randomized-input counts;
the scorecard keeps those separate. JScript and RC4 each detect two mutants;
txt2nls detects five. Eighty-three passing cursor tests do not erase CEnum Next's
actual rustc E0308 (`*rgelt = 0`), or its missing ownership/callback coverage.
CCopyAsPath lacks complete helper bodies and a faithful COM/clipboard substrate.

Final commanded stages sum to 25.217 seconds on this warm host. This includes
candidate compilation and oracle checks, not historical translation or manual repair.
It is not an extrapolation to unobserved modules or an end-to-end fleet throughput rate.

## Shipping JScript decoder

`dll/win32/jscript/decode.c` (180 lines) now links from `jscript-rust` on all three
architectures. The original C remains on disk. c2rust preserved tables, control flow
and Wine diagnostics; the automatic lowering removed obsolete stable-feature syntax
and retained Windows scalar widths using compiler measurements. No algorithm was
manually rewritten. Preparation took 0.731 seconds and transpilation 2.229 seconds.

The root's larger host corpus compared 3,267 inputs with zero status/buffer mismatches
or guard failures, and detected both mutants. The final cohort recompiles the actual
shipping Rust module rather than trusting a cached host library. All three DLLs expose
the expected C-convention symbol (`_decode_source` on i386) and retain Rust decoder
symbols. Guest boot smokes are regression checks, not execution coverage of the decoder;
its behavioral evidence is from the host differential tests.

ARM64's default boot manifest does not contain JScript. The new
`vibeos-cli build --arch arm64 --extra-target jscript` proves the DLL links while
preserving the normal boot build and guards; it does not imply JScript is on that media.
Saved raw Rust and ABI metadata are in
`tools/rustconv/.responses/compiler-jscript-decode/`; replay the normalized ABI record
rather than regenerate the translation after its C compile command disappears.

## Reproduction and failure boundaries

From the repo root, use the project Python environment and set `PYTHONPATH=tools/rustconv`:

```sh
python -m rustconv.cohort_runner --repo . \
  --staging ~/vibeos-rust-staging/rustconv-speedrun \
  --shell32-bank /path/to/shell32-worktree/tools/rustconv/.responses \
  prepare --manifest /persistent/path/cohort.json
# Same global arguments, followed by:
# run --manifest /persistent/path/cohort.json --output /persistent/path/new-run
```

The external staging input is the preserved txt2nls repair and repair manifest; bank
inputs are existing purchased artifacts. A missing artifact is a hard failure, never
permission to buy it again. The runner makes no model calls. It recompiles candidates,
retains commands/logs/tool identities, pins source/candidate/grader/fixture bytes,
and verifies those pins before and after each split. Changed compilers also invalidate
the frozen run. Its normal exit is 1 while the held-out rejection/unsupported rows remain;
inspect `valid` separately from `passed`. Malformed grader output, stale original-C
oracles, and failed mutation controls abort the measurement instead of lowering a
candidate's score. Detailed C++ verdicts come from the saved report, not console summaries.

Final evidence is under `~/vibeos-rust-staging/rustconv-speedrun/`:
`cohort-final-manifest.json`, `cohort-final-run/`, `decode-generation/`, and
`cohort-cpp/root-semantics-heldout.json`. The final pipeline fingerprint is
`d24305e53ea9a8efc113f1c65c682ac7d0379235561f60db176c666cacb0b4c7`.

## Integration verification

`vibeos-cli check` passed (`20260923-195942-4eb_tht4-check-all`). All 59
`check-tooling` checks passed; durable log: `cohort-tooling-settled.log` in staging.
The default CRT differential suite also passed (`20260923-200044-xr8s_80d-difftest-host`);
an intentionally nonexistent selected case was refused.

Cold build summaries report OK for all architectures:
`20260923-195709-rafblj7z-build-amd64`, `20260923-195710-id5x6cew-build-i386`,
and `20260923-195711-6ar2baub-build-arm64` (with `--extra-target jscript`).
Final boot stages passed in `20260923-200055-djlmezku-smoke-amd64`,
`20260923-200055-x_joxdq9-smoke-i386`, and `20260923-200055-v2qaanzv-smoke-arm64`.
Screenshots were inspected; amd64's 15 network checks also passed. Smoke summary
verdicts are WARN solely because a warm provenance refresh followed the cold builds.
All three JScript DLL hashes remained identical to the earlier verified links.
No stale-media override was used. This work has not been landed or pushed.

## What this enables next

Use compiler candidates for additional bounded C files with executable C references;
keep the route opt-in until coverage supports broader use. Use the C++ numeric/macro
mechanisms where their documented contracts fit. The next substantial C++ work is a
faithful PIDL/COM ownership and callback test substrate, followed by a new held-out
cohort. Do not count missing coverage as acceptance, or spend on retrying the already
banked Next body. Whole-pipeline repair effort, unbiased first-pass quality, and
million-line throughput remain unmeasured; these twelve rows do not establish them.
