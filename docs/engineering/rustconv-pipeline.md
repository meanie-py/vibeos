# rustconv — Pipeline Internals

Deep reference for the `tools/rustconv/` C-to-Rust translation pipeline: the
pipeline stages, key files, post-processor rules, the probe-migration and
Stage-3 design, and the Codex cross-model path. The high-level "what it is",
the CLI command list, the cost table, and the static-analysis commands live in
`docs/engineering/agent-reference.md`; the money rules and the ordered plain-C loop
live in `AGENTS.md`.

## Cost-efficient translation (read before spending API $)

The paid `translate`/`submit` path is function-by-function with prompt-caching + a
resume cache. The rules that keep the bill down (validated on the win32k frontier: 18
files for $18.98, caching saving ~$18 ≈ half the bill):

- **Dry-run first (FREE):** `rustconv --arch <a> translate <file.c> --dry-run` prints the
  function count + first-prompt size so you can set a `--max-cost` cap before spending.
- **Function-by-function, not whole-file `batch`:** `translate` chunks per function so a
  single giant file doesn't blow the context or silently stub lazy functions (a known
  `batch` failure mode for >1000-LOC files).
- **Prompt-caching is the big lever:** the ~5K-token system prompt + shared context are
  cached across the per-function calls in a run — that's the ~50% saving. It's why the
  synchronous per-function path can beat the Batch API here despite no 50% batch discount:
  Batch loses the warm cache and can't resume mid-file.
- **Durability — never lose paid work:** output streams to `tools/rustconv/.responses/<ts>-<file>/`
  (gitignored, in-repo, survives kills/reboots); each function lands the instant it returns,
  the in-flight one as `.partial`. That dir is the canonical copy — never `-o /tmp/...`.
  **NEVER Ctrl-C mid-run** (you re-pay every completed function on retry); let it finish.
- **Integrate later (the free half):** assembling the `.responses/` output into the crate +
  `cargo check` fixes cost nothing — do that manually (AGENTS.md § Notes).

Full playbook + the rtl-specific recipe: `archive/rustconv-cost-efficient-translation.md`.

## Pipeline

```
C source → [scanner] tree-sitter parse → [resolver] match vs ntkrnl-sys catalog
→ [prompt_builder] system prompt + C + imports + exports → [batch_runner] LLM API
→ [postprocess] safe mechanical fixes → [assembler] write crate → [checker] cargo check ×3
```

After integration, logic that smoke never reaches is graded by `rustconv difftest-contract`:
shipping Rust vs the genuine C on the host, with generated corpora and required mutants
(`tools/rustconv/data/difftest/README.md`). Treat each bug it finds as a CLASS: the next
step is a $0 static check that screens every translated body for it, so the main pass
stops shipping that class — not just a fix at the one site.

`rustconv translate` accepts HAL paths and is HAL-aware (separate
`system_prompt_hal.md`, hal_ddk imports, auto-CMake edit). Output quality is
~70% useful for HAL — review before integrating (memory
`feedback_rustconv_hal_target.md`).

## The deterministic-first stages for plain C (2026-07-24)

Three $0 stages wrap the paid call. They exist because each catches a class the others
structurally cannot, and because everything deterministic must be closed *before* payment
(the substrate-closure protocol). AGENTS.md carries the short recipe; this is the detail.

**Stage 0 — `python -m rustconv.profile_derive <module>` (`profile_derive.py`).** Derives the
bindgen island from the CMake source list plus each TU's real include closure, rather than a
hand-authored allowlist. Key behaviours:
- Only TUs **actually compiled** count; on-disk-but-unlisted `.c` is `DEAD_SOURCE` (a previous
  batch paid to translate a file nobody compiles).
- The allowlist is tiered — an SDK header reached *only* through other SDK headers is excluded
  (`<winbase.h>` alone reaches all of Win32) — but a header that **owns a `case`-label
  constant is force-added**, because an omitted constant becomes a silent catch-all match arm
  (§9.112). On rtl8139 that single rule is what keeps all 38 `OID_*` labels resolving.
- §9.108 header conflicts are detected by **co-reachability** (a type bodied by two headers
  that no single TU includes together), which is the false-positive filter a naive
  "these headers conflict" claim lacks. A real conflict emits one `TYPES_STEM` island per TU
  group, each with its own wrapper — a union wrapper would rebuild the wall.
- Validated by rediscovering the committed `afd` and `disk` profiles and by independently
  finding wldap32's real 2-island split.
- **The emitted `-I` set is what `gen_driver_types.sh` LACKS, not what we searched.** The two
  are separate constants (`_SDK_SEARCH` vs `_GENERATOR_BASE_INC`) because they genuinely
  differ — `sdk/include/wine` and `sdk/include/winrt` are ours only. Keep the latter in sync
  with the bindgen invocation. Dirs are bucketed EARLY (`EXTRA_INC`) or LATE (`LATE_INC`) by
  the real command's own ordering, since an alternate surface hoisted above the SDK changes
  which header a bare `#include <x.h>` resolves to: wldap32 needs wine EARLY (it owns the only
  `config.h`), samlib needs it LATE (its `winioctl.h` collides with `ndk/iotypes.h`). Observed
  `build-<arch>/sdk/...` dirs are dropped entirely — the generator passes those per-arch as
  `${BUILD_DIR}`, and echoing one back would bake i386's generated headers into all three.
- **Prologue guards are evaluated where they are decidable.** Wine's `config.h` is an
  authoritative build-configuration oracle (autoconf's `/* #undef HAVE_LDAP_H */` asserts this
  build), so a guarded include under one of its ~330 undefs is dropped rather than mirrored.
  Previously the `#ifdef` was discarded and the `#include` kept, promoting it to unconditional
  — wldap32 then asked for OpenLDAP's `<ldap.h>`, which this tree does not vendor.
  Undecidable guards still keep both arms and emit `CONDITIONAL_PROLOGUE`.
- **`add_pch()`'s SKIP argument is honoured.** Those TUs do not compile with the PCH, and are
  skipped precisely because it conflicts with their own headers — wldap32's `ber.c` is both
  the PCH-skipped TU and the reason the module needs two islands.
- `--emit profile` prints a paste-ready `gen_driver_types.sh` arm; `--scaffold-dir` writes the
  crate skeleton. ERROR findings are blocking; exit code is 1 when any fires.
- **A module whose `.c` live in a sibling static-lib target needs `--target`.** msvcrt is the
  case: the `msvcrt` MODULE target compiles only generated stubs and `.rc` while
  `msvcrt_static` owns all 32 `.c`, so aiming at the module name used to report "compiles no
  in-tree .c … an already-Rust-flipped module looks exactly like this". That inference is
  wrong for every whole-archive **cohab** module, whose `.c` stay compiled forever and are
  beaten at link by `#[no_mangle]` Rust. It now names the sibling and the exact re-run.
- **A static-lib target has no `set_module_type()`**, so kernel-vs-usermode is unknowable from
  the CMakeLists — pass `--usermode` / `--kernel`. (Those flags are new: the error had named
  them for a while without the argparser defining them. A fail-loud whose remedy does not
  exist is worse than no message; see `test_usermode_override_exists_for_a_target_with_no_set_module_type`.)
- **BLOCKLIST_FUNCS is MEASURED and reported, never picked.** `--emit report` now carries the
  external-prototype survey: how many functions the module calls that the include closure
  really declares (the surface `BLOCKLIST_FUNCS=1` makes you hand-write), how many it calls
  that no closure header declares at all, and every name. The emitted profile keeps
  `BLOCKLIST_FUNCS=1` and puts the measurement in a comment BESIDE it. It does not choose,
  and the calibration is why. Raw `called - defined` over the twelve hand-set modules nearly
  separates them (samlib 73 vs tcpsvcs 66 — a 7-unit margin on twelve samples); refined to
  closure-declared prototypes it tightens to `intl 69 | desk 198 | sysdm 130 | syssetup 205 |
  aclui 97 | samlib 60` for the modules that chose 0 against `tcpsvcs 47 | wlanapi 16 |
  snmpapi 7 | wsock32 0` for those that kept 1. A real gap, still not a rule: ten samples, and
  the boundary pair is exactly the pair whose reasoning was never about a count — samlib chose
  0 because a hand-written declaration of an RPC-shaped SAM entrypoint wrong by one pointer
  level *links* and then corrupts at run time. Decide it the MACRO_SIGNED way: island it both
  ways, one cargo check each. There is also an ordering constraint that makes 1 the only safe
  unattended answer — `BLOCKLIST_FUNCS_FROM_CRATE=1` greps OUT_CRATE for `#[no_mangle]` names,
  so on a FIRST derive against an empty scaffold it blocklists nothing.
- **An already-FLIPPED module refuses rather than reporting empty sections.** The empty-TU case
  always hard-failed; the one that went quiet is subtler. A cohab flip comments the real `.c`
  out but leaves the `rust_*.c` scaffolding compiled, so the TU list is *not* empty — it is one
  shim with no switch statements, no enum members and no dead bodies in it. Every audit then
  printed an empty section that read as a pass. sysdm reported "(no case labels with
  identifiers)" that way while carrying the exact defect the case-label audit exists to catch.
  `is_shim_tu` recognises the scaffolding (the tree spells all 12 of them `rust_*.c`), and a
  shim-only source list exits 2. **`--flipped-sources` re-enables the list the flip commented
  out and audits against THAT** — a flip does not delete the source list, so the pre-flip
  translation surface is still in the CMakeLists and a flipped module can be re-audited without
  going to git (sysdm: 1 TU / 0 switch constants → 13 TUs / 77). It is deliberately narrow —
  the commented block must be a `list(APPEND X …)`/`set(X …)` whose every argument looks like a
  source file, and its parens must balance inside the commented run, so a `# set(WITH_DEBUG
  TRUE)` somebody disabled on purpose is never resurrected. 37 of the tree's CMakeLists have a
  restorable list, and every one is a module we flipped. The run says so
  (`FLIPPED_SOURCES_RESTORED`), because the report then describes C the build does not compile.

**Stage 1 — the paid call, with both envelopes.** See the sealing note below; always
`--seal-facts --seal-calls`. In the same run, `trivial_c.py` lowers the provable trivial band
(empty / constant-return / param-return / forwarder / forwarder-with-null / getter / setter,
plus debug-only bodies) for **$0**, and each lowering re-verifies itself against an independent
tree-sitter oracle — identical ordered identifier sequence and equal constant multisets —
**raising** rather than shipping if the two disagree, because nobody reviews a free
translation. Measured 9.0% of 94,323 repo-wide functions; 41% on stub-heavy modules but ~0% on
real driver code, so do not plan capacity around it.

`forwarder_with_null` (2026-07-24) admits the CRT's dominant idiom — `X(args)` delegating to
`X_l(args, NULL)` — by accepting literal `NULL` arguments *only* where the **same-TU** callee's
already-parsed signature declares a pointer at that position. Arity must match the callee
exactly (so a variadic or unknown callee is refused) and `0`/`FALSE` are not accepted, because
`0` at an `int` position is just zero. The oracle rewrites both `NULL` and
`core::ptr::null_mut()` to one sentinel before comparing, so checks 2–4 stay token-for-token
rather than being relaxed; a `NULL` that moved, vanished, or displaced a real argument still
perturbs the identifier sequence. (What the sentinel cannot distinguish is `null()` from
`null_mut()` — a *type* error rustc catches at candidate-check, not a silent miscompile.)

**Its real reach is smaller than the C looks, and the gap is the interesting part.** A text
match for `return Callee(<params>, NULL);` finds 146 msvcrt functions (76 in mbcs.c alone).
The shape lowers **11**. The difference is that the CRT spells the locale parameter
`_locale_t` — a *typedef'd* pointer — so neither the type spelling (`"*" in type`) nor
`c_type_to_rust` can prove pointerness, and `msvcrt-sys`'s bindgen island does not declare the
alias either. Refusing is correct: admitting it would assume the very fact the shape exists to
prove. **The lever, when someone wants those ~135: give the pipeline a typedef authority** —
extend the module's bindgen island to carry its pointer typedefs, then have
`_is_pointer_c_type` consult the island rather than the spelling. `test_trivial_c_forwarder_null.py::
test_typedefd_pointer_is_refused_because_nothing_proves_it` pins the current refusal so that
becomes a deliberate decision, not a silent behaviour change.

**Stage 1.5 — `rustconv crate-repair <crate>-rust` (`crate_repair.py`).** Bounded deterministic
repair over the whole-crate `cargo check --message-format=json`, run before paying anyone to
fix compile errors. Eleven rules, each with a closed authority — diagnostic-driven
(`width_alias_cast`, `truncating_numeric_cast`, `match_scrutinee_cast`, `anon_union_field`,
`packed_field_ptr`, `index_usize`, `no_std_path`, `raw_pointer_cast`, `array_decay`) and
source-driven (`flex_array_index`, `panicking_cast`); everything else stays RED and bucketed by
shape for triage. Three properties worth knowing:
- It **refuses rustc's own `.try_into().unwrap()` suggestion** — that converts C's silent
  truncation into a panic in a live service and trips `tools/scan_silent_failures.py`.
- Span repairs are driven off the `expected X, found Y` **labels**, not the primary span: in
  the "arguments to this function are incorrect" shape the primary span is the *callee* and
  the fixable value spans are secondary, so a primary-span implementation silently misses
  every multi-bad-argument call. **But labels alone are not enough either** (found 2026-08-07):
  once a call has TWO or more bad arguments rustc changes shape again — the top-level spans lose
  their labels entirely and each mismatch moves into a CHILD note whose *message* carries the
  types and whose single span is the argument. `_mismatch_sites` therefore reads both, deduped
  by span. Until it did, every rule in the module was blind to those calls: on intl it was an
  ordinary two-argument `wcscpy(dst, src)`, 7 sites per target sitting in RED for no other
  reason. If a whole class of call looks stubbornly unrepairable, check the rendering first.
- **Pass all three arches.** A cast whose target width equals the analysing target's pointer
  width is applied only on cross-target agreement; otherwise it is withheld with both
  proposals, because an `as u32` inferred from i386 is wrong on amd64/arm64 wherever the real
  parameter is `SIZE_T`/`ULONG_PTR`. The `flex_array_index` authority reads
  `types_<arch>.rs` — `types.rs` is only a `#[cfg]` dispatch stub.
- **It will not cast a `match` PATTERN** (`_in_match_pattern`). Rust forbids expressions in
  patterns, so `(RPC_S_INVALID_BINDING) as i32 =>` is `expected a pattern, found an expression`
  — and rustc reports a mistyped arm exactly like any other integer mismatch, so the rule aimed
  straight at `match`-on-status-code, which is where narrow-vs-wide mismatches actually surface.
  Two lanes hit it in one week: wlanapi's `WlanRpcStatusToWinError` lost all 5 arms, msvcrt's
  `create_mbcinfo` lost its dispatch (bindgen split a four-macro `#define` family into `u32`/`i32` on
  sign). The correct repair is to **widen the scrutinee** (`match Status as u32`) or fix the type
  where the constant is DECLARED — never at the arm — which also keeps the arms auditable as
  names under the catch-all-binding rule (field manual §9.112). Detection is not line-local: a
  wrapped or-pattern puts the `=>` on a later line, and `foo(A, b)` vs `(A, 0) => v` are
  indistinguishable at the comma until you step out of the enclosing bracket group.
  **`match_scrutinee_cast` now performs that widening** rather than leaving it as the largest
  decline bucket (293 sites on desk.cpl alone). It walks out to the enclosing `match` and casts
  the SCRUTINEE, cross-target-agreed like any width bake, and drops the whole match when the
  arms want different types — two expectations mean the arms are not one constant family.
- **`anon_union_field` routes a flattened anonymous-union field** through bindgen's
  `__bindgen_anon_N`. C injects an anonymous union's members into the enclosing struct's scope
  (`psh.nStartPage`), Rust has no anonymous members, and every property-sheet module hits it.
  The map is derived from the crate's own islands, never guessed: a field an outer struct
  declares itself is left alone (C shadows the same way), and a field the three arches route
  differently — or that one arch declares directly — is dropped rather than guessed. It
  deliberately declines the E0560 struct-LITERAL spelling: `S { a.b: v }` is not legal Rust, so
  the repair there is a zeroed value plus field stores, which is a rewrite and not a splice.
- **`packed_field_ptr`** (E0793) rewrites `packed.array.as_ptr()` to
  `core::ptr::addr_of!(packed.array).cast()`. `as_ptr()` goes through `&[T; N]`, and forming a
  reference to a field of a `#[repr(packed)]` struct is UB even if never read — `addr_of!` takes
  the address without ever forming one, and `.cast()` is C's array-to-pointer decay.
  `SP_DRVINFO_DETAIL_DATA` and `SHFILEINFO` are both packed in the SDK.
- **`index_usize`** (E0277) casts a non-`usize` array index. rustc reports this as a missing
  `Index<u32>` impl, NOT as a type mismatch, so there is no `expected X, found Y` label and
  `truncating_numeric_cast` never sees the site. `usize` is the width-varying alias itself, so
  the cast is portable and needs no cross-target agreement.
- **`array_decay`** answers C's most basic idiom: `WCHAR szBuf[80]` passed where an `LPWSTR` is
  wanted. C decays an array to a pointer for free and Rust does not, so this arrives as
  `expected *mut u16, found [u16; 80]` in **argument** position (E0308) and as
  `non-primitive cast: [u16; 80] as i32` in **cast** position (E0605) — C's `(LPARAM)szBuf`.
  Measured on intl by removing all 106 decays from the shipped crate and re-running: **105 came
  back deterministically**, the one miss being an artifact of the removal itself. Both shapes
  matter — 52 argument sites against 39 cast sites there, so handling only the first mechanises
  barely half the class.
  - The **guard is element-type identity**, and only in argument position: `[u16; N]` behind a
    `*mut u8` is a reinterpret, not a decay, and stays RED because nothing in the source claims
    it — the mismatch is evidence a *declaration* is wrong.
  - In **cast** position that guard is deliberately dropped. `(BYTE *)szBuf` is the author
    stating the reinterpret, and reproducing it is faithfulness; an array cast to a raw pointer
    can only ever mean its address, since an array is not pointer-sized.
  - Decay to an **integer** (`as LPARAM`) is accepted only where that integer is pointer-wide on
    the analysing target, and is cross-target agreed. That agreement is doing real work: a
    genuine byte-reinterpret like `[u8; 4] as u32` is pointer-wide on i386 *only*, so it fails to
    earn agreement instead of silently becoming an address. The replacement keeps the target type
    **exactly as the source spells it** — rustc reports the resolved `i32`, and writing that back
    would bake i386's width into a site whose source correctly says `LPARAM`.
  - The pointer-shaped edits need no agreement at all: `as_ptr` names no type and bakes no width.
    This is the one deliberate exception to the module's cross-target discipline.
- **It cannot repair a `#[cfg]`-dual'd body — KNOWN GAP.** `abi_enforce` renders an internal
  `system` fn as an x86 `stdcall` copy plus a non-x86 `C` copy (rustc will not unify
  `extern "system"` with `extern "stdcall"` at the *type* level on i686, so a `system` fn
  cannot sit in a `stdcall` vtbl/callback slot — the dual is correct and must not be collapsed
  to `extern "system"`). But each copy compiles on exactly ONE target, so the cross-target
  agreement rule can never be satisfied and every cast in it is withheld forever. Inside an
  arch-gated branch, baking that arch's width is correct *by construction* — that is the
  premise the rule is missing. netid hit this on 38 casts across its three dialog procs, and
  the two copies had already DIVERGED (the non-x86 one carrying `(TRUE) as i64` from an
  earlier pass while the x86 one was untouched), which is the shape to watch for: an
  error count that differs wildly per arch on the *same* function.

**Stage 4 — `python -m rustconv.cfg_gate` (`cfg_gate.py`, 2026-07-26).** For a module whose C
branches on a build-configuration `#ifdef` that is OFF in this tree. Run it after translation
and before `crate-repair`: it is what makes the crate compile at all, and the casts
`crate-repair` would otherwise chase live in code that this stage removes from the build.
- **Guards are derived, not named.** `guards_for_module` intersects the macros the module tests
  with the `/* #undef X */` set in wine's `config.h` — the same oracle Stage 0 uses. wldap32
  branches on EIGHT, so a single hand-passed `--guard` would have left seven silently wrong.
  Each maps to its own lowercase feature. `#elif defined(X)` arms are not in the `#ifdef` scan
  and still need declaring by hand.
- **The classification refuses more than it transforms, on purpose.** `#else` arms (the *other*
  arm is what this build compiles) and guards nested inside a `switch` are reported, never
  rewritten. That refusal is load-bearing: it is what surfaced wldap32's live §9.112 catch-all
  in `map_error`.
- **The off-arm's return value is LIFTED from the translated Rust**, not re-derived from the C
  constant — fact sealing already audited that rendering, and a second opinion could disagree
  silently. The twin copies the original's attributes so `#[no_mangle]` survives.
- Interacts with `crate-repair`: prefer the semantic alias (`usize`) over a bindgen width alias
  (`SIZE_T`) in any FFI signature the guarded code calls, or the repair pass is forced to bake
  `as u32` on i386 and `as u64` on amd64/arm64 for one expression and correctly withholds both.

## `rustconv port`: one module, stage by stage (2026-10-05)

`rustconv port <stage> …` drives one plain-C module through the loop above and records every
stage's time and spend in `~/vibeos-rust-staging/port/<stem>/state.json` (`port status <stem>`
prints it). Stages, in order: `derive <module-dir> [--target T]` (profile → the
`gen_driver_types.sh` arm), `island`, `scaffold`, `translate --cost-cap $`, `integrate`,
`resolve`, `macros`, `fix --cost-cap $`, `check`, and `replay` (all of them from the banks), plus
`hand` (bank a hand-written function). Every
stage is idempotent; re-run any of them after a fix.

- **`translate --escalate`** buys with DeepSeek thinking OFF (~1/40 of the output tokens on
  small functions), then re-buys with thinking ON only the bodies a deterministic check
  rejected. One `--cost-cap` covers both passes, and the second revisits only the TUs the first
  ran. `--dry-run` prints the functions each TU would buy.
- **`resolve`** fills the prelude between generated markers from what the tree already
  agrees on: bare enumerators aliased to the island's prefixed constant (`FreeLdr` →
  `_BOOT_STORE_TYPE_FreeLdr`, exact, all three islands must agree), constants that agree by
  VALUE, extern fns that agree by ABI class. Types are never copied (allowlist the header).
  It also refreshes the prelude's `pub use win32macros_rust::{…}` list from the shared crate.
- **`macros`** ports the module's OWN function-like C macros the crate calls (filesup.h's
  `DoesFileExist`, ntverrsrc.c's `VersionInfo32_*`) plus the local macros their bodies use,
  in one thinking-off call, banked by a hash of the definitions. The reply must be exactly
  the requested `macro_rules!`. Then `integrate` turns each `NAME(` call site into `NAME!(`.
  SDK header macros go in `win32macros-rust` instead, once, for every crate.
- **`check`** counts errors per arch, separates modules not yet translated, and warns
  `UNDERCOUNT` when a failed GLOB import is present: rustc stops reporting unresolved names
  once one fails (an invented `use ntdll_sys::nt::tier1::*;` took setuplib from 40-odd E0425
  to zero while fixing nothing).

Acceptance and seal fixes from the same run:

- **A body must PARSE.** `c_body_checks.parse_findings` runs rustc's own parser (rustfmt) on
  every paid body and rejects one that fails — one unparseable body hides every resolution
  and type error in the crate (setuplib read "94 errors" over ~1,800). Banked bodies are
  re-judged on replay, so `port translate --escalate` re-buys exactly the old ones that fail.
- **`L"AddReg.NT" INF_ARCH` is sealed.** A wide literal pasted against a macro used to be
  skipped, and the model copied the C concatenation verbatim. `macro_authority.
  string_literal_macros` reads each arch's `-dM`; a macro that is string literals on every
  built arch is folded into the run, and one whose value differs per arch (INF_ARCH is
  `L"x86"`/`L"amd64"`/`L"arm64"`) becomes one name with a `#[cfg(target_arch)]` static per
  arch (pointer positions only). 318 such sites in the tree's C.
- **`--rebuy-fn NAME`** (translate and `port translate`): a banked body that stopped replaying
  because a seal got stricter is refused (the guard against silent re-buys), and the refusal
  now names this flag. It re-buys only the named bodies. A body whose C itself CHANGED (the
  SEH scanner fix) is re-bought without it, and says so.
- **`port macros` runs with thinking ON**: macros get none of the per-function audits, and the
  thinking-off reply compiled `VersionInfo32_Children` as a struct-scaled pointer add.

crate-repair rules the setuplib residue added (each is C's own semantics, nothing guessed):
`c_truthiness` (an integer in a condition is `!= 0`, C's `!x` is `== 0`, a `0` compared
with a bool-returning call is `false`, a condition stored into BOOLEAN is `as u8`),
`option_fn_call` (a call through a bindgen `Option<fn>` is `.unwrap_unchecked()` — calling
NULL is undefined in both languages; `.unwrap()` stays forbidden), `uninit_c_local` (`let x:
T;` read or field-filled first gets `= core::mem::zeroed()`), `c_mutable_binding` (rustc's
own `mut ` suggestion: C bindings are assignable), and `anon_union_field` now rewrites a
struct LITERAL naming an anonymous-union field into a zeroed value plus field stores. The
width and anon-union authorities read ntdll-sys through the island's public glob. And the
pass loop now also reverts a pass that adds a parse/expansion error, because one of those
stops type checking and makes the total FALL (546 -> 16 on a pass that broke 3 macros).

The stitcher bug that one found: `stitch_file` located a function's start by matching
STRIPPED lines and did not know `unsafe extern "C" fn` without `pub`, so an indented
`#[cfg(..)]` inside the body was taken as the start; everything above it was hoisted as a
"standalone item" and 20 partlist functions ended up nested in `UpdateDiskLayout`. Only
column-0 lines count now. `port translate --replay-only` reassembles every TU from the bank
for $0 after a fix like that.

ntstrsafe.h's `RtlString*` family (NTSTATUS twins of strsafe, `static __inline`, exported by
nothing) now lives in `strsafe-rust::ntstrsafe`, transcribed rather than aliased — the
oversized-destination and overflow behaviour differ from the HRESULT family. `port resolve`
re-exports win32macros-rust and strsafe-rust by name through a registry, adding the Cargo
dependency, and never copies a tree-consensus `extern` for a name a shared crate provides.

### `port fix`: the compiler in the loop (2026-10-05)

Per-function translation is bought blind -- the model never sees rustc -- and crate-repair then
needs one hand-written rule per error class. On setuplib that plateaued near 300 errors per
arch: the last four repair passes moved the count by 11. `port fix` closes the loop the way an
agent with `cargo check` does: every function rustc still rejects goes back to the model with
its own rendered errors (tagged by arch, so a width error says "only on i386"), the C it came
from, and the crate's declarations of every name the errors mention. Thinking off, ~$0.002 a
function. Its first round on setuplib took 295 to 131 errors per arch for $0.33.

That round also shipped semantic bugs, which is what the guards are for. Names rustc could not
find had been exempt from "keep every constant and call", and the model filled them in itself:
`ERROR_SYSTEM_PARTITION_NOT_FOUND` became `ERROR_SYSTEM_DEVICE_NOT_FOUND`,
`STATUS_NOT_SAME_DEVICE` the Win32 `ERROR_NOT_SAME_DEVICE`, two NTSTATUS names became WRONG hex
(`0xC000017D` for `0xC0000172`), `InstallFat12BootCode` became `InstallFat32BootCode`, three
`_SEH2_LEAVE`s were deleted, `ASSERT(c)` became `if !c { return … }`, and a hand-inlined
`RTL_CONSTANT_STRING` counted the NUL in `Length`. So (`crate_fix.reply_findings`):

- **Every capitalised name and every lowercase callee in the input must survive.** A rename is
  allowed only for a TYPE or FIELD that rustc itself suggests (E0412/E0422/E0573/E0609...);
  never for a value or function (E0425) -- rustc's "similar name" for `iswspace` is `isspace`.
  Method names, `min`/`max` and type names in casts may change.
- **No new integer literal >= 0x100**, no added item (unless the C function defines that type
  locally), no `.unwrap()`/`try_into`, signature unchanged, and the reply must parse.
- **Verdicts are never banked**: replies are re-judged on every replay, so fixing a wrong guard
  recovers every reply it refused for $0 (the key salt is not a prompt version).
- A fix leaving its function with more errors on any arch is reverted; a round that adds a
  parse/expansion error is bisected for the culprit (one check per fix took most of a run).
- **A refused function's retry carries the refusal**, thinking off. Escalating to thinking ON
  bought the same wrong answer at ~10x the price (Fat12->Fat32 and iswspace->isspace, twice
  each); `--thinking escalate` still exists.

What a fix may not invent, resolve now supplies. `resolve` evaluates every still-undefined
ALL_CAPS name with clang through `sdk_facts` -- per arch, agreement required, the C TYPE kept
(`STATUS_PARTITION_FAILURE: NTSTATUS`, not `i32`) -- into a `port facts` prelude block. A C TU is
probed in C (a duplicated `case 0 + (long long)(X)` prints the value, an incompatible struct
initializer prints the typedef). `win32macros-rust` gained `ASSERT` (inert, type-checked),
`C_ASSERT`, `RTL_CONSTANT_STRING`, `UlongToPtr` and `PAGE_ROUND_UP`.

**`port replay <stem> [--fix-cap $]`** rebuilds the crate from every bank in order for $0:
translate, resolve, macros, integrate, crate-repair, resolve AGAIN, crate-repair, fix, check.
(Until 2026-10-06 it did not: `port replay` passes cap $0 and the translate loop stopped at the
cap before the first TU, so every replay re-ran the later stages over whatever the files already
held. A replay-only run now ignores the cap, and never takes the buying retry path.)
resolve runs twice because rustc reports unresolved names only once nothing halts it: before
crate-repair setuplib's resolve saw 5, after it 34. The three arches' `cargo check`s now run at
once in separate target dirs (crate-repair and port alike); cargo locks a build dir whole, so
sharing one serialised them. And crate-repair refuses an edit outside the crate it repairs: the
truthiness rule followed a span into win32macros-rust and rewrote `IsRecognizedPartition` to
suit setuplib.

**Expand the module's own macros in the C, don't port them** (`port translate --only X.c
--expand-macros`, recorded per TU in the port state; `translate --expand-local-macros DIR`).
A C macro is untyped text; a `macro_rules!` body has one typing for every caller, and C's
silent promotions differ per call site (`GetPartEntryOffsetInBytes` is `i64 * u32`). rustc then
reports the error inside the macro body, away from the function that could fix it. Expanding
before the seals means the model translates what the compiler sees, and crate-repair and
`port fix` see each use in its context. Function-like macros and object-like aliases of a
member or function expand; constants stay names for the seal. On partlist.c every macro-body
error went away (11 -> 8, the rest being an island header and fn-pointer ABIs). Ten bodies
re-bought, $0.20 with escalation. The C rule that matters: a macro is not re-expanded inside its
own expansion (`#define SpInfFindFirstLine (SpInfExports.SpInfFindFirstLine)`).

**`goto` has two sanctioned lowerings, and the dodge is refused.** Told "no goto", a
thinking-on reply invented `goto_NoSystemPartition!()` shims, which nothing parsed or checked.
The mechanical prompt now spells out both lowerings (a labeled block for a forward jump to a
shared tail; a state variable plus `loop { match }` for jumps between several labels), and
`c_body_checks` rejects a `goto_*` macro, call or nested fn at purchase, so the body escalates
instead of shipping.

### The setuplib residue, closed by tooling (2026-10-06)

After `port fix`, setuplib's errors were a dozen distinct classes, few of them macros, so each
went into a deterministic stage rather than a re-buy. Every one is general:

- **Island:** a pack PUSH in a TU's prologue (`#include <pshpack1.h>`, popped after its own
  typedefs) was copied into the merged wrapper and packed every later header -- 62 structs,
  `USETUP_DATA` among them. `scan_preamble` drops packing includes. A TYPE the C reaches only
  through an SDK header's own includes (`MOUNTDEV_NAME`, ddk/mountmgr.h via `<mountdev.h>`) is
  now found by resolve in the include closure, added to the arm (kept in the port state across
  re-derives as `island_headers`) and the island regenerated.
- **resolve is namespace-aware.** A missing TYPE was satisfied by a same-named CONSTANT from
  tree consensus (`FILE_SYSTEM` became bugcodes.h's `0x22`); `prelude_resolve.unresolved`
  records type vs value, and E0573 "expected type, found constant" counts as a missing type.
- **Calling conventions of DEFINED functions** come from clang (`align_definition_abis`): seven
  NTAPI callbacks were defined `extern "C"`, a stack-imbalance on i386 that only showed as a
  cast error at the install site.
- **c2rust startup initialisers fold into their statics.** A static C cannot initialise at
  compile time (`RTL_CONSTANT_STRING`) is zero-filled by c2rust and assigned in a
  `.CRT$XIB` constructor; the preamble kept the zeros and dropped the constructor, so two
  UNICODE_STRINGs shipped empty. `fold_static_initializers` moves each `NAME = EXPR;` into
  NAME's initialiser (Rust evaluates it at compile time) and refuses any other store.
- **crate-repair rules**, each C's own semantics: `option_fn_is_null` (`.is_null()` on an
  `Option<fn>` is `.is_none()`), `local_shadows_constant` (C lets a local shadow an enumerator,
  E0530; the local is renamed for the rest of its block), `sizeof_constant` (`size_of::<CONST>()`
  -> `size_of_val(&CONST)`), `fn_item_cast` (drop c2rust's `f as extern "C" fn(..)` and let the
  function coerce), `some_fn_item` (a function into an `Option<fn>` slot is `Some(f)`),
  `option_fn_as_pointer` (an `Option<fn>` passed as PVOID maps None to NULL),
  `variadic_promotion` (rustc's `as c_int`/`as c_uint` for a narrow argument to `...`),
  `c_local_static_mut` (a C `static` holding a pointer), `pointer_double_negation` (`!!p`),
  `zeroed_not_default` (`{0}` on a union struct), `raw_pointer_field` (C's `p->f` written `p.f`,
  receiver found by hand because rustc's suggestion points into `CONTAINING_RECORD!`),
  `unsigned_negative_literal` (`(ULONG)-1`), and `packed_field_ptr` also answers
  `size_of_val(&packed.field)` without forming a reference. `option_fn_call` no longer needs
  `fn(` in the message: rustc abbreviates long types to `Option<...>`. A repair pass that
  applies nothing (every edit refused at write) now ends the session.
- **resolve: c2rust's local types replace the model's twins.** The model collapses `typedef
  struct _X {..} X;` into one `struct X` while c2rust's tables spell `_X`; the module's copy
  shadowed the prelude's and they never unified. Resolve drops a module's definition of every
  name it takes from c2rust, and fetches the typedef `X` with a tag `_X`.
- **An invented success value is refused:** `c_body_checks.invented_flag_assignments` (purchase
  and `port fix`) -- see the field manual's invented-values entry.
- **CRT function-like macros** (`iswspace`..., `min`, `max`) are functions in win32macros-rust;
  resolve skips function-like macros as externs, so nothing supplied them.
- **`port fix` and dead C.** The fix model sees only the lines the build compiles, and a name
  that appears only on never-compiled lines may be dropped (osdetect.c `#define`s NDEBUG and
  then calls `DumpNTOSInstalls` under `#ifndef NDEBUG`).
- **do-while + continue** joins goto: a prompt recipe (`loop { 'body: { .. break 'body; .. }
  if !(c) { break; } }`) and a purchase-time reject using the lint's own detector.
- **C-variadic definitions** have no deterministic stage yet; finish them by hand with
  varargs-rust's `va_entry!` (a naked thunk defines the `...` export, no C file) and the
  `rust-pullers` feature for `va_arg`, and bank them with `port hand`. The C-shim template is
  the older pattern (syssetup retired its shim for this one).
- **`port hand <stem> file.rs:Name[=Bought]`** banks a hand-written function so `replay` puts
  it back; replay rebuilds every module from the banks and would otherwise erase the residue
  a human finished. A banked function whose target vanished fails the replay, loudly, and fix
never buys for a banked function.

`vibeos-cli difftest` (and so the gate) now also runs a shared crate's own known-answer tests
when its manifest sets `[package.metadata.difftest] host-tests = true` (win32macros-rust,
strsafe-rust): nothing ran them, and the `as u8` crate-repair once wrote into
`IsRecognizedPartition!` reached master with two of them failing.

c2rust anchoring fixes the setuplib run forced, all in `c2rust_anchor.py`:
`--preserve-unused-functions` (c2rust prunes an unused `static`, and liveness is per arch —
`IsAcpiComputer` is only called under `#ifndef _M_AMD64`); a body missing from the requested
arch is anchored from another, and one NO built arch's preprocessor keeps is dropped from the
bill as compiled out (`DumpPartitionTable`); attached `-isystem/path` flags are kept (dropping
arm64's CRT dir under `-nostdinc` was what made every arm64 c2rust run "panic"); and a
file-scope `L"…"` in a table, which c2rust spells as a pointer into a transmuted temporary
(rejected as a dangling pointer in a static), is hoisted into a named `RUSTCONV_C2R_WSTR_*`
static.

## `body-audit`: did a pre-seal body keep its C's calls? (2026-09-26)

Call sealing makes a dropped call fail at translation time; crates translated before it
existed (fastfat and cdrom were the first DeepSeek lanes) never had that check. `rustconv
body-audit <module dir> | --all` pairs each flipped-out C function with its same-named Rust
and lists the C callees the Rust no longer performs, $0, about a minute tree-wide. A callee
counts as kept through a crate-local helper or live C glue the body delegates to, a known
lowering (`RtlZeroMemory` -> `write_bytes`, COM `IFoo_Bar` -> `lpVtbl).Bar`, A/W variants,
intrinsics -> `asm!`), or a macro whose expansion's calls are all kept. Dead `#if` arms,
DUMMY_PSEH except handlers and `#if DBG` code are optional; `_SEH2_FINALLY` is not, since it
runs. cfg-split twins of one fn are judged as a union.

It is a LEAD GENERATOR, ranked `stub-silent` > `high` (state-changing callee) > `medium` >
`low` > `stub-loud`. Read each lead against the C before fixing or re-buying: macro state it
cannot resolve (`NTDDI_VERSION` arms) and hand-restructured bodies still produce false leads.
Motivating case: fastfat's `FatCreateNewDirent` lost `RtlSetBits`, so every new dirent stayed
free in memory. Re-translate only what the audit confirms broken.

**The ledger and provenance (2026-10-04).** `tools/rustconv/data/body_audit_reviewed.toml`
records a verdict per `(module, function)` over the exact callees a human read: `false` and
`benign` are hidden (counted in the header), `owed` stays listed with where the work is
tracked, and a drop nobody reviewed brings the lead back. Several entries for one function
merge. `--provenance` asks git whether each dropped callee was in that C function's body when
the Rust file was first committed; one that was not is upstream work landed on C already
flipped (a re-port for upstream-sync), not something the translator lost. A false lead that a
rule could recognise belongs in `body_audit.py`, not the ledger; `--show-reviewed` lists the
hidden ones.

The 2026-10-04 burn-down read every high and medium lead (198 + 951): about 70 real bugs
fixed, 214 owed upstream re-ports, the rest false or benign, and the false shapes became
rules (C from another binary under the module dir, `#if 0 // comment`, module `-D` names and
the forced `_USE_DUMMY_PSEH`, a four-char pool tag swallowing 460 lines in the masker,
`#[link_name]` aliases, member-access macro aliases, printf-family equivalence). Medium
fell from 951 to 474 on rules alone before anyone read a lead.

## `--target`: proven from the tree, or refused (2026-07-24)

`--target <name>` for a `dll/win32/*` module not in `TARGETS` builds a Target via
`config.make_user_dll_target`. That used to **assume** two things — output crate `<name>-rust`,
substrate `ntdll-sys` — which are right for the winsock/net fleet and silently wrong for
anything else. On msvcrt both were wrong at once: the crate is `msvcrt-crt-rust`, and it does
not depend on ntdll-sys *at all*, so every `ntdll_sys::` path the model emitted was
unresolvable by construction. Per the `crate_bindgen_files` contract that is not a no-op — the
model invents a forward declaration that compiles, links, and is silently wrong; here that
means a **guessed `threadmbcinfo`**, whose `mbctype[257]`/`mbcasemap[256]` every mbcs predicate
indexes.

Both are now derived, and raise `TargetResolveError` rather than guessing:

- **Output crate.** `<name>-rust` if it exists. If it does not but crates matching `*<name>*-rust`
  do, refuse and list them (`--output-crate`). If nothing resembles it, the module is genuinely
  new and the conventional name is correct.
- **Substrate.** Read the `*-sys` path dependency the output crate actually declares. Exactly
  one → use it; none or several → refuse (`--sys-crate`). A new module with no Cargo.toml yet
  keeps the ntdll-sys default. Every pre-existing user-DLL crate (ws2_32, iphlpapi, netapi32,
  dnsapi, wkssvc) declares exactly `ntdll-sys`, so this returns their historical value
  unchanged — `test_target_resolve.py` pins that.
- **Umbrella.** Detected from the substrate's `lib.rs`: one `pub mod <x> { … }` (ntdll-sys →
  `ntdll_sys::nt::*`) or a flat root `pub use bindgen_<arch>::*;` (msvcrt-sys →
  `msvcrt_sys::*`). Flat is represented as `""`, which is **meaningfully different from `None`**:
  `None` means "kernel target" and skips user-mode post-processing entirely, so `postprocess`
  gates on `sys_umbrella is not None`, not on truthiness. Anything ambiguous raises.
- **Prompt addendum.** `templates/system_prompt_<name>.md` when present, else the generic
  `system_prompt_user.md`. `system_prompt_msvcrt.md` carries the CRT rules: substrate is
  `msvcrt_sys` flat; never redeclare `threadlocinfo`/`threadmbcinfo` (arch-varying offsets —
  `pcumap` is at 208 on i386, 336 elsewhere); read the real `pclmap`/`pcumap` rather than
  hardcoding an ASCII fold; `_invalid_parameter` + `*_errno()` instead of `STATUS_*`; exports
  are `cdecl` on every arch including i386.

## Translating an ntoskrnl file (2026-10-03)

ntoskrnl.exe is cargo-linked (`add_cargo_module(... KIND kernelexe)`) and every one of its
`.c` is a commented `#${REACTOS_SOURCE_DIR}/ntoskrnl/...` entry in `ntos.cmake`. So no kernel
file had a CMake owner, an include path or a compile command, and a sealed translate refused
outright. That is now handled:
- `profile_derive` treats a kernelexe target as a `kernel` module compiling `${SOURCE}`.
- It restores repo-rooted flipped entries.
- It follows linked targets' `target_include_directories(... INTERFACE ...)`.
- It audits a kernel file as itself, so other, uncompilable TUs (mm/marea.c and the deleted
  cache/ headers) do not block it.
- `macro_authority._derived_kernel_entry` synthesizes the compile command from that derived
  profile plus a real kernel-mode TU's compiler flags. The c2rust anchor reuses it.
- c2rust gets `-D_InterlockedX=InterlockedX` renames, because it otherwise silently drops every
  function that calls an MSVC interlocked intrinsic.

Two things the run will NOT do for you. Kernel code lives in split crates (`ps-rust` etc.),
but the import header is stitched for `ntoskrnl-rust`, so rewrite it to the crate's own
substrate. And the SEH macros come back literally. Route every user access through
`ntkrnl_sys::probe` (`probe_write_user`, `safe_copy_raw`, ...), and check each `_SEH2_TRY`
in the C has survived: `job.c`'s handle write-backs did not.

## An SDK enum's width is an ABI fact — prove it from all three islands (2026-08-04)

The `--cpp-com` scaffold refuses any state field whose type it cannot prove is scalar,
pointer, or a source-visible POD. `FOLDERVIEWMODE m_FVM` (CDefViewUtil's
`CShellFolderViewCBWrapper`) is none of those from the header's point of view: a C
`typedef enum { … } NAME;` has **no fixed underlying type**, so reading shobjidl.h cannot
settle whether it is 4 bytes on every target. Reading it and assuming `int` would have been
the guess the rest of this pipeline exists to refuse.

`cpp_com.island_integer_aliases` settles it from evidence instead. bindgen renders each
per-arch island against the real target, so `island_<arch>.rs` already *contains* the answer —
`pub type __WIDL_shobjidl_generated_name_00000025 = ::core::ffi::c_int;` plus
`pub use self::__WIDL_… as FOLDERVIEWMODE;`. The prover follows that re-export chain in
**i386, amd64 and arm64** and accepts the name only when all three resolve to the *same*
primitive. 176 SDK names qualify; `SIZE_T`, `ULONG_PTR` and `LPARAM` deliberately do not,
because their widths diverge — accepting one arch's answer is exactly how a cast inferred from
i386 has shipped a wrong width before.

Two consequences worth keeping straight:

- **The scaffold never re-declares a proven name.** It emits `pub m_FVM: FOLDERVIEWMODE`, which
  resolves through the prelude's `pub use crate::island::*` to the island's own alias. A local
  `pub type FOLDERVIEWMODE = INT;` would compile (a local item shadows a glob) and would be a
  second source of truth for a width — the thing `scan_type_alias_contradictions` exists to catch.
- **No islands means no proof, not a default.** `island_files=()` yields an empty set and the
  field stays a loud preflight failure. `cpp_tu` resolves the island paths from the caller's
  `repo`, never from config's install-time root, because `pip install -e` bakes the *main*
  checkout into site-packages and a sibling worktree's island would prove this tree's layout
  against another lane's headers.

## Key Files

```
tools/rustconv/
├── rustconv/
│   ├── scanner.py         # tree-sitter C parser (functions, globals, SEH, arch ifdefs)
│   ├── catalog.py         # Index ntkrnl-sys symbols (~9,700 per arch)
│   ├── resolver.py        # Match identifiers → imports + gap warnings
│   ├── spec_parser.py     # Parse ntoskrnl.spec / hal.spec
│   ├── prompt_builder.py  # Assemble system + per-file prompts
│   ├── batch_runner.py    # Anthropic + Gemini API client
│   ├── postprocess.py     # Safe mechanical fixes (type names, imports, enums)
│   ├── assembler.py       # Write to ntoskrnl-rust/src/{subsys}/{file}.rs
│   ├── checker.py         # cargo check ×3 targets → structured errors
│   ├── probe_migrate.py   # Stage 2: ProbeFor{Read,Write} → Result<T>
│   ├── verify_constants.py / verify_abi.py / verify_ffi.py  # Static-analysis verifiers
│   └── config.py          # Paths, model config, token budgets, Target dataclass (kernel/hal)
├── templates/
│   ├── system_prompt.md       # Kernel translation rules (~5K tokens, prompt-cached)
│   ├── system_prompt_hal.md   # HAL-flavor translation rules
│   ├── function_prompt.md     # Per-function prompt template
│   └── probe_migrate_prompt.md  # Stage 2 migration rules
└── data/
    ├── gap_functions.toml         # FORCEINLINE/macro functions not in bindgen
    └── trace_catalog_{arch}.json  # debugcon byte-marker decoder catalogs
```

## Post-Processor Safety

`postprocess.py` only does context-free transforms that are always correct:
- Type name underscore-prefixing (`OBJECT_HEADER` → `_OBJECT_HEADER`)
- Import fixes (`ddk::*` warning, pool type dedup)
- Enum variant name prefixing (`ObCreateHandle` → `_OB_OPEN_REASON_ObCreateHandle`)

Union field access (`__bindgen_anon_N`) is NOT done by the post-processor — it's
context-dependent and handled by the system prompt teaching the LLM to generate
correct accessors directly.

## Probe Migration Stage (`probe-migrate`)

Second pipeline stage that migrates bare `ProbeForRead`/`ProbeForWrite` calls in
already-translated Rust code to use the Rust-native probe API
(`ntkrnl-sys/src/probe.rs`). Operates file-by-file with
preprocess→LLM→postprocess pattern.

```bash
python -m rustconv probe-migrate se/sqos.rs           # Single file
python -m rustconv probe-migrate se/ -m haiku          # Batch subsystem
python -m rustconv probe-migrate se/sqos.rs --scan-only  # Show probe sites
python -m rustconv probe-migrate se/sqos.rs --dry-run    # Preview without writing
```

Key files:
- `rustconv/probe_migrate.py` — scanner, preprocessor, postprocessor, runner
- `templates/probe_migrate_prompt.md` — 10 migration rules for LLM
- `sdk/lib/rust/ntkrnl-sys/src/probe.rs` — Rust-native probe API

The probe API returns `Result<T, NTSTATUS>` instead of raising exceptions:
```rust
probe_read_user::<T>(ptr, mode) -> Result<T, NTSTATUS>
probe_write_user::<T>(ptr, value, mode) -> Result<(), NTSTATUS>
probe_user_buffer(buf, len, align, mode) -> Result<(), NTSTATUS>
probe_capture_unicode_string_header(ptr, mode) -> Result<UNICODE_STRING, NTSTATUS>
```

**Status:** SE and OB subsystems fully migrated to Rust-native probe API. All 3
arches build + smoke tests pass. Other subsystems still use bare
`ProbeForRead`/`ProbeForWrite` calls — see memory
`feedback_translation_philosophy.md` for staging.

## Multi-Stage Pipeline Architecture

The translation pipeline has multiple stages, each refining the previous output:

```
Stage 1: C → Rust        (rustconv submit/batch)   — bulk transliteration
Stage 2: Probe migration (rustconv probe-migrate)  — ProbeForRead/Write → Result<T>
Stage 3: Result+Drop-RAII idiomation (planned)     — de-transliterate to idiomatic Rust
```

**Stage 3 is NOT "SEH idiomation."** There is no SEH to convert: `USE_DUMMY_PSEH`
makes `_SEH2_EXCEPT` handlers dead code on all 3 arches (see DUMMY_PSEH section
in AGENTS.md), and the kernel is `panic=abort` with no unwinding. Stage 1 already
produced Rust where the exception path doesn't execute. Stage 3's job is to
de-transliterate the *normal* path: replace the C goto-cleanup cascades and
manual lock/unlock pairs with `Drop`-based RAII (`Drop` *is* the `finally`, and
it is sound here precisely because every scope exit is an explicit `return` — no
unwind), bare `NTSTATUS` i32 with `Result<T, NTSTATUS>`, and magic integer
constants with `enum`/newtype/`bitflags!`.

The shape per function: an idiomatic `<snake_name>_rs(...) -> Result<T, NTSTATUS>`
core (suffix `_rs`; distinct from the existing `_rs_<CName>` inline-shim
*prefix*), with the existing `#[no_mangle] extern "system"` C symbol collapsed to
a thin trampoline (`Ok(v)` → `write_volatile` outparams + `STATUS_SUCCESS`,
`Err(s)` → `s`). The trampoline MUST keep `write_volatile` on every
output-pointer write (a naive `*out = v` reintroduces the captures(none)/ud2
miscompile) and keep the `extern "system"` ABI. Adoptable one function at a time
so smoke stays green throughout.

Stage 3 will operate **function-by-function** (not file-by-file) using paired
C+Rust function context, a dependency graph (SQLite cache), and Rust-native API
surface for RAII guards, safe user-mode copies, and Result patterns. The LLM sees
the original C alongside the current Rust translation, plus callees and type
context.

## Codex CLI (Cross-Model Translation)

OpenAI Codex CLI (`codex`) can be used as an alternative to rustconv for
C-to-Rust translations. Cross-model translation catches different blind spots
than same-model review.

```bash
# Non-interactive translation
codex exec --full-auto "Translate these C functions to Rust: ..."

# Default model works (o3 requires paid OpenAI API, not ChatGPT account)
codex exec --full-auto "$(cat prompt.md)"
```

**Key prompt rules to include:**
- `#[no_mangle] pub unsafe extern "system" fn` for all exported functions
- Pool tags: `u32::from_le_bytes(*b"XXXX")` (C multi-char literals are big-endian)
- `NT_ERROR(x)` = `((x as u32) >> 30) == 3`, NOT `x < 0`
- `extern "system"` for FFI (stdcall on i386)
- DUMMY_PSEH: skip `_SEH2_EXCEPT` handlers entirely
- `core::ptr::addr_of_mut!` for KeWaitForSingleObject event pointers
- `IO_STATUS_BLOCK.Information`: use `as _` for cross-arch portability

**Post-integration checklist:**
1. Run `verify_constants.py` on the output to catch wrong constant values
2. Run `verify_ffi.py` to check ABI mismatches and C macro usage
3. Use RAII guards (`PfnLockGuard`, `SpinLockGuard`, `IrqlGuard`) instead of manual lock pairs
4. Use `drop(guard)` at the exact original release point — do NOT let guards drop at function end
5. Prefer `Result<(T, NTSTATUS), NTSTATUS>` returns over output pointers for internal Nt* functions — preserve exact NTSTATUS, don't hardcode STATUS_SUCCESS
6. Use `core::ptr::write_volatile` for ALL writes to output pointer parameters — prevents LLVM captures(none) elimination
7. Run `python3 tools/scan_ud2.py --check build-i386` after building — catches LLVM miscompilation (ud2 in named functions)
8. Run `python3 tools/lint_kernel.py` to find unprotected output pointer writes
9. Fix type casts (`PVOID`, `as _` for arch-dependent sizes)
10. Build all 3 arches + run smoke tests (use `--skip-clean-build` for fast iteration, omit for final verification)

## Developing the pipeline itself (lint / types / tests)

`rustconv` is a maintained Python package, not a scratch script — it carries its
own dev tooling in `tools/rustconv/pyproject.toml`:

- **ruff** (`[tool.ruff]`): lint (`E/F/I/UP/B/SIM/RUF`) + format. Same ruleset as
  the cli and `agent-swarm` so the whole tool surface lints under one standard.
- **pyright** (`[tool.pyright]`, basic mode): run it pointed at the venv
  interpreter so deps resolve — `pyright --pythonpath .venv/bin/python rustconv`.
- **pytest** (`tests/`): pure, no-network unit tests. `test_semdiff_pure` pins the
  semdiff Rust-function/JSON extraction; `test_config_cost` the cost/model math;
  `test_cli_invariants` guards the import hygiene (every submodule imports,
  `__main__` has zero inline imports, and the analysis subcommands that share
  symbol names — `run_oracle`, `extract_file`, `print_summary`, … — stay
  module-qualified so they can't collide on a naive re-hoist).

Run the whole surface (rustconv + cli + `tools/*.py` + agent-swarm) in one shot:

```bash
./vibeos-cli check-tooling            # ruff + ruff format --check + pyright + pytest
./vibeos-cli check-tooling --no-tests # fast lint+types only
```

**Import policy:** hoist imports to module top. The only imports kept inside
functions / guarded by `try/except` are *genuinely optional* third-party deps —
`google.genai` (the `gemini` extra), `PIL`, `shellcheck_py`, and the
`tomllib`→`tomli` 3.10 backport — where a top-level import would break
`pip install` without the extra. That guard is the correct idiom, not a smell.

## Stage 3 — Result + Drop-RAII de-transliteration

Stage 1 output is a faithful C *transliteration*: the borrow checker, type system, and ownership model are essentially unused (the fattest subsystems ke/mm/ps/ob/ex score ~2–3/10 idiomatic across ~447K LOC — a few hundred `Option`/`Result`, thousands of whole-function `unsafe extern` blocks, ~5000 1:1 `#[no_mangle]` symbols). Stage 3 de-transliterates the *normal* path into idiomatic Rust while smoke stays green throughout.

**Stage 3 is NOT "SEH idiomation."** `USE_DUMMY_PSEH` makes `_SEH2_EXCEPT` handlers dead code on all arches and the kernel is `panic=abort` with no unwinding — there is no exception control flow to idiomate. Drop-as-`finally` is sound *precisely because* of this: every scope exit is an explicit `return`, so guards fire deterministically.

**The seven moves (incremental, per-function, smoke-green throughout):** (1) magic values → `enum`/newtype/`bitflags!`, run every new constant through `verify_constants.py`; (2) remove C control flow (labeled-`loop`-as-goto, goto-cleanup cascades); (3) break up 300–600-line files — safe *within* `ntoskrnl-rust` (`#[no_mangle]` is flat namespace); spinning a fn into a *new rlib* triggers the `lto=false`/`cgu=256` firewall — don't do casually; (4) tighten `unsafe` scope; (5) introduce `<snake_name>_rs(...) -> Result<T, NTSTATUS>` cores (suffix `_rs`, don't collide with the existing `_rs_<CName>` inline-shim *prefix*); (6) collapse the C `#[no_mangle] extern "system"` symbol into a thin trampoline over the `_rs` core; (7) new structures (locks, rundown, refs) implement `Drop` → `Drop` **is** the `finally`.

**Trampoline-template rule (load-bearing):** every out-param write in the trampoline MUST be `core::ptr::write_volatile`. A naive `*out = v` reintroduces the LLVM `captures(none)`/`ud2` miscompile the whole kernel-Rust surface is defended against. Keep the `extern "system"` ABI on the exported symbol (never `extern "C"`). Prefer `Result<(T, NTSTATUS), NTSTATUS>` over hardcoding `STATUS_SUCCESS`. Drop guards at the *exact original release point* — don't let a guard drift to function end.

**Frozen pilot / template:** `ob::ObReferenceProcessHandleTable` — a self-contained, survey-flagged leak rebuilt as `RundownGuard: Drop` + an `_rs` core + a thin trampoline, validated with smoke ×3 + `scan_ud2` + `verify_abi`, **frozen as the canonical template**. Copy its shape.

**Bulk ordering:** `ob` first (most self-contained; pilot lives here); `ke` last/heaviest (~55K LOC); algorithm sibling rlibs (`slist`/`avltable`/`splaytree`) are **already idiomatic** — leave them alone. The stage operates function-by-function (never file-by-file) with paired C+Rust context.

## c2rust behavior-anchor path (`translate --c2rust`)

`--c2rust` anchors each function's translation on its **c2rust transpilation** — a compiler-exact, behavior-preserving Rust body — injected into the prompt as a `## Behavior Anchor`. This stops the mechanical LLM pass from silently *stubbing* hard functions or turning function-like macros into phantom externs (an anchor-less pass emitted only 7 of 13 functions in a driver file, dropping 4 exports, invisible until link). The anchor preserves the **algorithm layer** (deref chains, IRQL checks, exact `DbgPrint` format strings, `wrapping_add`, GF/crypto constants); the LLM still introduces shell/constant bugs *around* it, so the Claude review tier stays **mandatory**.

Pieces: `tools/rustconv/rustconv/c2rust_anchor.py` (`build_anchor_map`, `build_preamble`); the `## Behavior Anchor` rule in `templates/function_prompt_mechanical.md`; the `--c2rust` flag. Paid plain-C `translate` enables this path by default. A missing binary, c2rust failure, empty map, or missing selected function **hard-fails before payment**. The requested architecture pins the compilation database; there is no silent cross-architecture fallback. `--no-c2rust` explicitly opts out. Library callers may request best-effort behavior, but the CLI uses strict mode. A compiler anchor constrains the prompt; it does not prove the model preserved its behavior.

**A THIRD blocker — the one that looked like "c2rust refuses the SDK include closure" — is FIXED, and it was never the closure.** c2rust 0.22.1's Clang AST exporter cannot map three function ATTRIBUTES, and each drops the TYPE of the declaration it sits on (`Missing type <id> for node: AstNode { tag: TagFunctionDecl … }` then `Exported Clang AST was invalid`). One such decl anywhere in the include closure kills the whole transpile, and all three live in msvcrt's PRIVATE headers, so every msvcrt TU inherited them:
`__WINE_MALLOC` → `__attribute__((malloc))`, and `__WINE_CRT_PRINTF_ATTR`/`__WINE_CRT_SCANF_ATTR` → `__attribute__((format(printf|scanf,…)))`.
Measured on `dll/win32/msvcrt` (2026-09-13): 32 invalid-AST sites → 26 with `__WINE_MALLOC` neutralised → **0** with the two format attrs as well, and the TU transpiled. Module-wide: **2/37 TUs anchored before, 35/37 after.** `rustconv` now passes the three `-D` neutralisers automatically (`c2rust_anchor._ATTR_NEUTRALIZERS`), ahead of the db flags so a db `-D` of the same name still wins. This is safe for an ANCHOR because none of the three carries semantics we translate — they are optimiser and `-Wformat` hints, not ABI, not body — and the real build never sees the `-D`. The one C change it needed is two `#ifndef` guards around those macros in `dll/win32/msvcrt/include/corecrt.h` (they were unguarded, so the command line could not override them); the guards change no build, since nothing else defines the names.
**Do not diagnose this class from the panic.** The panic names neither file nor construct; the actionable line is the `--> <header>:<line>` under each `Exported Clang AST was invalid`. Preprocess with the pinned clang (`clang -E`) and read what the macro actually expanded to before theorising — `__WINE_DEALLOC` and `__WINE_ALLOC_SIZE` LOOK like the culprits and are already empty under clang, whose `__GNUC__` is 4.

**Two known c2rust 0.22.1 ICEs — reproducers + root causes in `docs/engineering/c2rust-upstream-bugs.md`.** Read that before assuming a `--c2rust` failure is your fault; "fix the C side first" is **not** available for either:
* **Any file using `_SEH2_TRY` (260 `.c` files) — or wine's `__TRY`/`__EXCEPT`/`__EXCEPT_CTX`, which is the same class under a different spelling (it is what still blocks msvcrt's `except.c` and `cpp.c`, the only 2 of 37 that do not anchor) — cannot be anchored.** Under `_USE_DUMMY_PSEH=1` (our default) the macro expands to a block opening with `__label__ __pseh_try_end;`, and c2rust does not support GCC local-label declarations — one `Missing child … TagDeclStmt` warning per label, then a panic in the *comment locator* that names neither the file nor the construct. The actionable message is the `Exported Clang AST was invalid` line **above** the panic; the panic itself is noise.
* **Taking the address of any `__stdcall`/`WINAPI`/`CALLBACK` function ICEs** (`convert_type.rs:518`, `Attributed` not unwrapped in the K&R path). One-line fix, verified locally, not yet vendored.

**Installing c2rust (the LLVM footgun):** `cargo install c2rust` needs **`llvm-config` on `PATH`** (not just `LLVM_LIB_DIR`) — `build.rs` calls it for `--version`/`--libs`. `libclang-*-dev` alone is insufficient:
```bash
sudo apt-get install -y llvm-14-dev              # provides /usr/bin/llvm-config-14
ln -sf /usr/bin/llvm-config-14 ~/.local/bin/llvm-config
cargo install c2rust --version 0.22.1
```
**The transpile invocation** (two dead ends: pointing at `compile_commands.json` treats it as a dir → flag-less → `ntdef.h not found`; a one-entry db dir → `expected exactly one compiler job`). ✅ **Direct-source form:** `c2rust transpile --output-dir DIR SOURCE.c -- <FLAGS>` where `<FLAGS>` = `--target`/`-D`/`-I`/`-isystem`/`-include` from that file's `compile_commands.json` entry, **dropping `-o`/`-c`/`-M*`**. Output at `DIR/src/<stem>.rs`; c2rust may warn/panic but **still writes the `.rs`** — check for the output *file*, not the exit code. Note: **i386's cmake has `CMAKE_EXPORT_COMPILE_COMMANDS` OFF** (no db); already-translated files are `REMOVE_ITEM`'d so absent from the db — anchors only work for **still-C** targets (which is exactly where they're needed).

**Preamble (file-scope data) comes from c2rust, not a second LLM call.** Take ONLY `static*` and aggregate `const` (contains `[`/`{`); EXCLUDE scalar `const NAME: _ENUM = 5;` (c2rust floods header enum-variants that collide with bindgen). The path prepends these exact tables and **dedups** — dropping any LLM-emitted `static`/`const` of the same name (the LLM copy is fabrication-prone). Verify the agent actually spliced the big tables (a silent AES-S-box drop is invisible until `cargo check`).

## Shrinking giant C files: the `#include`-back split

rustconv's giant-file problem is a *batch* problem, not a fidelity one (`translate` already chunks function-by-function), so physical splitting is only to keep the *output* `.rs` sane. When you split, use **`#include`-back, never real TU-splitting.** Keep `<base>.c` as the **single** TU: it retains the prologue + first function group and appends `#include "<base>.split2.c" …`; fragments hold the rest **verbatim**. The preprocessor reassembles the identical TU → **zero CMake change, zero linkage/static/global hazard.** Safety guards: (1) per-file byte-exact reconstruction assertion (`base[:cut] + fragments == original`); (2) confirm the source list is explicit `list(APPEND SOURCE …)`, **not** `file(GLOB)` (a glob double-compiles fragments); (3) `nm --defined-only` identical pre/post; (4) smoke gate all arches. **Splitting invalidates per-file sidecar analysis** (Pass-0/2/3 `*.json`) — wipe all sidecar globs and regenerate (deterministic, no API cost).

## LLM mechanical transforms as a "remainder closer"

When a bulk mechanical C→C (or Rust→Rust) normalization is deterministic for *most* sites but a minority need a local judgement call, build the LLM stage as the **remainder closer** with the deterministic tool's output shapes as the contract — not a from-scratch rewriter. (This closed a 64-site win32k SEH-lowering ledger the deterministic `seh_rewrite.py` correctly refused, ×3 green.) The shape: (1) unit = enclosing function, splice back by char offset highest-first; (2) contract = the deterministic tool's exact output shapes so LLM output is byte-consistent; (3) `validate()` before any splice (brace/paren balance, required-symbol present, no stub markers, **token-superset**); (4) single cached API pass, reruns/`--apply` are $0; (5) build + smoke is the real gate — "0 units held back" is *yellow*, not green. **The bug only the build caught:** an LLM stage that emits calls into a shim must own the same include/declaration plumbing the deterministic stage does (a missing `#include` injection failed files the mechanical pass never touched).

## tree-sitter for C structural transforms: differential oracle, never `has_error`

tree-sitter's C grammar is **lenient, not a preprocessor** — real files parse "clean" yet emit `ERROR`/missing nodes because unresolved macros (`NTAPI`, SAL, `#if` arms) become ERROR subtrees, so `root_node.has_error` is **effectively always true**. Use a **differential oracle:** parse original + transformed, require `errs(new) <= errs(orig)` (no *new* errors) **plus** a domain invariant (e.g. `switch` `case` labels unchanged). The real gate stays build + `nm` + smoke. tree-sitter IS right for plain-C structural transforms (a `switch` in a probe-lowered fn is plain C — `switch_statement`→`case_statement`); the "can't parse macros" lore is specific to `_SEH2_TRY/_EXCEPT`. tree-sitter offsets are **byte** offsets — splice in `bytes` or a non-ASCII char desyncs everything after. Conservative per-unit proof beats blanket extraction (extract a switch arm only if it ends in `return`, no `break`/`goto`/`_SEH2_`, idents ⊆ params ∪ file-scope ∪ arm-local).

## Resume cache and the `--no-resume` model-upgrade footgun

`translate` persists each function's Rust to `tools/rustconv/.responses/<subsystem>__<stem>/<func>.rs` the instant it returns, with `manifest.json` recording a sha256 of the C body. Next run, every function whose **current C-body hash matches** is reused for **$0** (`CACHED ($0)`); only new/changed functions hit the API. A killed run costs at most the single in-flight function. **Footgun — the cache keys on `(subsystem, stem, fn-body-hash)`, NOT the model.** Re-running `translate --fn X -m opus` to *upgrade* a function first translated by a cheaper model returns the **cached worse output for $0.00** — no Opus call happens, silently. You **must** pass `--no-resume` to force a fresh call with the stronger model (applies to re-translating review-flagged stubs, and the c2rust-anchor path re-translating an already-cached file).

## DeepSeek mechanical-mode failure modes (detector/auto-fix backlog)

DeepSeek (`--mode mechanical`) is the cheap gruntwork tier but markedly weaker than Claude. Recurring failure modes found grinding driver crates — each a candidate auto-fix/detector: (1) **leaves raw C `goto`/labels** in complex fns → wrap body in a `'cleanup: { ... }` block, forward `goto Exit`→`break 'cleanup`; detector: grep live `goto |^\s*\w+:\s*$`. (2) **`expr as usize < size_of::<T>()` without parens** → rustc mis-parses `<` as generics; `cargo fix --broken-code --allow-dirty` auto-applies the paren suggestion (run *before* grinding). (3) **adds `use ...::*` globs** → E0659 cascades (+700 phantom errors in one turn); rail: NEVER glob, use explicit `use ntkrnl_sys::ddk::{Name};`. (4) **stubs hard functions to no-ops** (`{ let _ = a; }`) — flag in review. (5) **wrong module path for driver-private structs** (`ntkrnl_sys::ddk::FDE` when it's `crate::types`) — bulk-sed. (6) **strips *correct* `as i32` casts** on C-shaped shims → E0308 flood; feed callee signatures into the prompt. (7) **drops bit-masking helper calls** (`SRB_STATUS(x)==S` → `x==S`) — compiler can't catch; detector: call-superset (every `NAME(` in input reappears in output). (8) **emits C ternary `?:` and stream-of-consciousness `//` comments** — reject output containing `?...:`. **General:** the grinder's self-reported error counts are routinely exaggerated — trust your own `cargo check`. Re-running repair on already-regressed code does NOT self-heal a prior semantic drop — run deterministic passes first, repair on a **clean** baseline.

## shell32 cheap-model evidence path

The shell32 C++/COM lane has stricter body-worker experiments that never delegate ABI,
constants, or the runtime call set to the model:

```bash
# Preview is guaranteed inspection-only; --prompt forces --dry-run.
python -m rustconv --arch amd64 translate path/to/Class.cpp \
  --target shell32 --cpp-com --class CClass --fn CClass_Method \
  --mode mechanical --seal-facts --seal-calls --prompt
# (--model deepseek is the DEFAULT since 2026-08-06; spell it only to escalate, e.g. -m sonnet)

# Batch N candidate/base replacements, one cargo invocation per architecture.
python -m rustconv candidate-check candidates.json
python -m rustconv candidate-check candidates.json --repair
python -m rustconv substrate-plan .responses/candidate-oracles/<run>/report.json

# Compile a whole newly generated module with no accepted function base.
# Manifest candidate entry: {"name":"pilot", "candidate":"pilot.rs",
#   "source":"path/to/Class.cpp", "full_module":true}
python -m rustconv candidate-check module-candidates.json

# Compare ordered cross-language branch/loop/return shape (veto, not acceptance).
python -m rustconv skeleton-check candidates.json

# Read-only opportunity census; alpha matches are never reused automatically.
python -m rustconv dup-census dll/win32/shell32 --top 20
```

`--seal-facts` owns HRESULT predicates and constant occurrences. `--seal-calls` owns each
known runtime callee occurrence with a unique exactly-once executable token. `--seal-wide`
owns every UTF-16 literal. Any of the three envelopes fails loud before the normal body
audit, and each flag contributes its own INDEPENDENT response-cache token (`factseal_v1`,
`callseal_v1`, `wideseal_v1`, joined by `__`) — which is exactly what makes a new protection
free to add: turning one on misses the cache and pays for the bodies it covers, while every
body already banked under the other tokens stays addressable and is never re-billed. Folding
a new protection into an existing token's meaning would re-charge for work already owned.

**`--seal-wide` (2026-08-07).** Rust has no UTF-16 literal, so a model asked to translate one
has nothing correct to write: it invents an encoding, and every invention is a hand-count no
compiler checks — a `b"..."` byte string behind a wide pointer (mswsock), an alignment-1
interleave, a `[u16; N]` one character short (wkssvc's 39 registry names), `L"0000%s"` banked
as five zeros (intl), or a terminator simply left off (18 shipped arrays). The seal replaces
each occurrence with an opaque token, reveals only its Rust TYPE (`*const u16` / `[u16; N]` /
`u16` — the model needs that to declare around the token, and rustconv computed it), and
substitutes the real code units afterward. Two C shapes, and **the second is the larger**:
across the tree's 5,987 `.c` files `L"..."`/`L'x'` is 27,545 tokens, while the Wine-style
brace list `{'C','o','m','b','o','L','B','o','x',0}` is 4,272 declarations and 55,928
elements — and every one of the 18 arrays #60 repaired was a brace list. They are not symmetric: `L"..."`
IMPLIES a terminator so the seal appends one; a brace list STATES its own, 195 of them
deliberately end without one, so the seal copies exactly what the C wrote. The restore is a
generated `RUSTCONV_WSTR_<hash>` static behind `.as_ptr()` in pointer position and an inline
array elsewhere — deliberately NOT the crate's `w!` macro, whose result type is not uniform
across this tree (secur32's yields `&[u16; N]`, aclui's `*const u16`). Measured coverage over
those 5,987 files: **31,127 sealed, 76 reported as `UNSEALED WIDE LITERAL` audit findings**
(0.24% — macro-pasted runs, sized initializers whose lowering is a copy rather than a value,
non-literal brace elements). Pure zero-fills (`= {0}`, `= L""`) are neither sealed nor
reported: they carry no bytes, and demanding a token there would reject the `[0u16; N]` /
`zeroed()` lowerings that are correct for them. The seal also covers the **file-scope
preamble**, which is a separate paid call the other two seals never reach and is precisely
where those 18 arrays lived.

**Sealing is NOT shell32-only any more (2026-07-24).** It works for any plain-C module —
kernel drivers, Win32 DLLs, services — and you should pass `--seal-facts --seal-calls` by
default. The dialect is selected automatically: `--cpp-com` keeps the **frozen** `cpp`
behaviour (changing it renumbers tokens and silently re-bills every cached *paid* shell32
worker), everything else gets the `c` dialect, which additionally protects **CamelCase DDK
enum constants** (`NonPagedPool`, `KernelMode`, `Executive`, the `Ndis*` vocabulary) and
**mixed-case numeric `#define`s** (`NERR_Success`) from generated ground truth
(`gen_enum_constants.py` → `rustconv/ddk_enum_constants.py`, staleness-gated in
`check-tooling`). The whole winsock + Tier-2 service program ran with sealing OFF and
shipped 13,503 unprotected constant occurrences; do not repeat that. Rationale, the
storage/type direction rules, and the numeric-replacement-text classifier that keeps SEH
control-flow macros (`RpcTryExcept`) out of the envelope: field manual **§9.117**.

`candidate-check` generates a durable synthetic `no_std` crate under
`.responses/candidate-oracles/`, replaces exactly one worker per isolated module backed by
an accepted shell32 base, and checks i386/amd64/arm64. `--repair` is deliberately bounded:
generated signatures and constant contexts; accepted-base argument casts, initializers,
and ABI struct fields; lexically proven mutable reassignment; and closed C-idiom lowerings
for known wide strings, shell allocation, byte-preserving memcpy, COM out pointers/unions,
and the two exact sealed strsafe append formats. It does not apply rustc suggestions blindly
or call an LLM. Every repair is named in `report.json`; a red candidate makes the command
nonzero.

For out-of-sample class pilots, a candidate entry with `"full_module": true` compiles the
assembled module directly and does not require or consult an accepted base function. This
mode is the honest oracle for generated scaffold/substrate completeness; do not manufacture
a base merely to make the replacement-mode harness run.

Every candidate oracle report now contains `substrate_analysis`: unresolved diagnostics are
collapsed into distinct roots across architectures, indexed against candidate definitions,
shell32 shared/module Rust, project Rust, SDK headers/IDL, and the optional manifest `source`,
then routed to deterministic ownership buckets. `rustconv substrate-plan REPORT.json` prints
that plan; `--refresh` recomputes it after analyzer changes. Follow
`rustconv-substrate-closure-protocol.md` rather than repairing a rustc cascade site by site.
This analysis is frontend-neutral and is intended for ordinary C modules too.

The shell32 C++ stitcher also emits a source-selected deterministic support closure after
the class/vtable scaffold. Selection includes the implementation, defining class header,
prepared workers, and generated interface/scaffold surface so types used only by WIDL or an
inline declaration are not missed. It currently owns numeric local/resource/unique-SDK
literal facts, simple source and SDK `typedef struct` POD closures (including embedded PODs,
SAL-stripped fields, proven fixed arrays, and pointer aliases), simple
`CMVERBMAP` aggregate tables, known Win32 FFI signatures, compiler/header macro lowerings
such as `IID_PPV_ARG`, implicit C scalar/pointer conversions, and the proven `CStringW`
operations used by the pilot. Post-response semantic lowerings deliberately run on cache
hits as well as fresh responses: expanding deterministic knowledge must not force a paid
regeneration. Shared COM interfaces remain in `shellcom.rs`; extend that canonical ABI from
WIDL rather than generating per-file duplicates.

The prepared helper closure is a C++ semantic boundary too. rustconv binds same-name,
same-arity local overloads using source-declared parameter, local, nested-call return, and
pointer-family types; expands trailing default arguments into explicit calls; then rewrites
the selected helpers to collision-free Rust worker names. Ambiguous calls fail in preflight.
Overload resolution and defaults are compiler facts, not translation judgment, so neither
belongs in an LLM prompt or post-hoc review.

Parallel DeepSeek translation drains every submitted future even when one worker fails API,
seal, or body audit. Accepted workers are committed immediately to the per-function resume
cache; rustconv raises a combined error only after the pool is drained. Declaration-only
extern blocks are deterministically discarded before executable fact/call closure, so a
cheap model cannot make a token fail merely by redundantly declaring a scaffold-owned call.
This ordering does not weaken the call proof: after normalization each required token must
still occur exactly once as executable code.

Every non-dry `translate` appends JSONL to `.responses/outcomes.jsonl`: model, target, arch,
mode, seal flags, source hashes, cheap routing features, outcome, token/cost/latency, and
artifact/cache identity. This ledger is the input to a future measured difficulty router;
do not invent thresholds before the representative corpus exists.

Sealed mechanical shell32 COM runs also consult an exact prepared-body translation memory
before making an API call. Its key includes the method/signature, deterministic interface
and state shape, and sealing namespace; only the owning class identifier may be substituted.
Every hit still passes the normal semantic normalization and worker audit before promotion
to the per-file response cache. Alpha-normalized census matches are never executable reuse.

Body-local `CComPtr<T>` ownership is deterministic. The source declaration must correspond
to exactly one null-initialized raw Rust pointer slot; rustconv injects a `CppComPtrGuard<T>`
watching that slot so every return path releases it. Any explicit translated release nulls
the slot and is therefore idempotent with the guard. A missing/renamed slot hard-fails
normalization rather than trusting the model to reproduce C++ scope-exit cleanup.

`skeleton-check` parses the C/C++ worker and Rust candidate with their native tree-sitter
grammars, then compares a deliberately small semantic skeleton: ordered conditionals,
loops, switches/matches, breaks/continues, and returns. It reports exact normalized shape
plus a fuzzy sequence score. Use it to reject obvious dropped or invented control flow.
Never accept on a similarity score: it cannot prove expressions, FFI types, pointer
semantics, ownership, or call arguments, and the compiler remains mandatory.

The first 20-worker shell32 corpus measured templates at 10/10 x3 compile for $0 and sealed
DeepSeek at 3/10 raw, 7/10 after bounded repair. All 20 had exact normalized control
skeletons. Retrying the three residuals with Sonnet produced 0/3 compile-green because the
dominant failures were missing Rust substrate/API adaptations, not model capacity. Route
unresolved-symbol and duplicate-ABI-type failures back into deterministic context/repair
work before paying for a stronger model.

After those substrate adaptations landed, a zero-call replay of the same cached DeepSeek
workers reached 10/10 model-tier and 20/20 overall on all three architectures. This is an
in-sample substrate result, not yet a production acceptance-rate estimate: broaden the
corpus across classes, and keep fact/call closure, body audit, skeleton compatibility, and
x3 compilation mandatory.

The first class with no accepted Rust base, `CCopyAsPathMenu`, initially produced the same
62 full-module diagnostics on every architecture. After the deterministic support closure
landed, the unchanged eight cached DeepSeek workers replayed for $0 and the module reached
zero diagnostics on i386/amd64/arm64. This validates substrate repair as the correct route
for unresolved ABI/header surface. Deterministic C-linkage bridges then exposed its two
internal C++ `SHELL_*` helpers, the generated ATL adapter took over the object-map entry, and
the C++ owner was disabled. The integrated image built and passed the complete three-arch
gate. The full pipeline result is therefore: two template workers plus eight cached
DeepSeek workers, no retry or stronger model, 62-to-zero x3 diagnostics, and class-atomic
link/smoke acceptance.

A later paired test retried three heterogeneous DeepSeek rejects once with Sonnet under the
same sealed mechanical contract. Final acceptance was 0/3 for $0.127848: one refactored
shared cleanup beyond occurrence-level call certification, one added a second model-owned
`scopeguard` cleanup layer, and one invented helpers for an unsupported `CDrop` owner after
reintroducing `SUCCEEDED`. Treat custom C++ RAII and shared-cleanup CFGs as deterministic
substrate/decomposition work, not automatic Sonnet routes. Static difficulty may nominate a
stronger model only after the substrate classifier is clear.

The subsequent hard-five stitched-worker comparison measured repair distance against accepted
full modules rather than demanding raw envelope acceptance. After identical bounded repairs,
Sonnet was x3-compile-green on 3/5 versus DeepSeek 2/5, left 2 versus 24 diagnostics per
architecture, and was closer to the accepted worker on four of five cases (496 versus 683
normalized edit tokens). Sonnet cost 2.55x for this set. Route substrate-clean high-band
compile failures to Sonnet when agent repair time matters; keep low/medium and deterministic
shapes on the cheaper lane.

The next production-shaped three-class batch confirmed that routing must retain a
decomposition tier above model selection. `CRecycleBinCleaner` used seven fresh sealed
DeepSeek workers (~$0.02), then replayed unchanged from 24 unique full-module diagnostics to
zero x3 after deterministic WIDL callback, ATL resource-string, array, SDK-struct/FFI, and
implicit-receiver support landed. `CFolderOptions` later closed through a hybrid substrate:
Rust owns activation, public vtables, refcount, and site state, while narrow C++ bridges own
the retained property-page callbacks and private `IBrowserService2` operation. The dialog
keeps only an opaque published-interface pointer and calls Rust ABI helpers, never the retired
C++ layout. The 231-line, 94-call `CQueryAssociations::GetString` worker broke its
72-token call envelope with DeepSeek, while a prior isolated Sonnet artifact had already
failed the same envelope. Route that shape to deterministic CFG decomposition; do not spend
on repeated whole-worker retries merely because a stronger model exists.

The first giant-class run (`CShellLink`, 91 workers / 219 vtable slots) added a second
zero-call recovery layer. A response that streamed completely but failed the then-current
deterministic audit remains a `.partial` paid artifact. Before submitting a missing sealed
shell32 worker, rustconv now tries matching historical partials through the current exact
signature, fact/call unsealing, semantic normalizers, and body audit. Only a result that
passes the same gates as a live response enters the cache; truncated, differently sealed,
or genuinely bad streams remain rejected. This recovered a prior `CreateShortcutIcon`
response after deterministic SDK-shadow stripping improved, avoiding another paid call.

Worker normalization now retains exactly the contracted function and discards all model-
owned module prefix/trailing surface. Body-local `#[repr(C)]` structs are accepted only when
the C worker declares the same named field sequence; otherwise type declarations remain a
hard ABI reject. Conversely, a model-local definition of an SDK type explicitly used (but
not declared) by the C snippet is removed so the generated/shared substrate stays the sole
layout authority. These are audit-boundary changes, not compiler repair heuristics.

By-value ATL `CStringW` class fields are also deterministic state, not opaque C++ owners.
The scaffold initializes them with `CStringW::new`, routes source assignment through an
owned helper, and calls `drop_in_place` before freeing the raw COM allocation. Static
worker-window callbacks are generated adapters: they recover the Rust object pointer from
window extra storage and invoke the translated body. Their `WNDPROC`, message records,
notification-icon SDK layout, and FFI surface remain outside the model response. The FFI
authority maps pointer-width header aliases to real symbols (notably the 32-bit
`GetWindowLongW`/`SetWindowLongW` exports).

The worker audit examines ABI declarations before declaration-only normalization can erase
them. It permits `unsafe extern "system" fn(...)` as a function-pointer type, but rejects
model-owned `extern` blocks and exported ABI functions. This keeps callback expressions
legal without allowing a body worker to compete with deterministic module scaffolding.

Cached-worker normalization is required to be exactly replay-idempotent. In particular,
`CComPtr` guards and the nulling paired with an explicit `Release` are inserted at most once;
an unused immutable raw-null placeholder emitted for a source-unused ATL local is removed
with its invented defensive release. Replaying an already-normalized worker must produce
byte-identical normalized output.

The C++ scanner also recovers a return type/calling-convention prefix when tree-sitter starts
an out-of-line qualified method node mid-line after `HRESULT WINAPI`. Without this recovery,
factory hooks such as `FinalConstruct` can silently acquire `()` in their generated contract.
The recovered prefix is source syntax, never a model inference.
