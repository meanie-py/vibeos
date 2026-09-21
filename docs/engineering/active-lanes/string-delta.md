<!-- Research pass for the ucrtbase de-Microsoft-ification quest, string/ family.
     Produced 2026-09-20 by a read-only survey agent, reviewed before landing.
     This is a FINDINGS document, not a plan of record: it says what the UCRT
     contract is and where our existing msvcrt-graded Rust differs from it. The
     burn-down it feeds is `python3 tools/ucrt_burndown.py --arch i386`; the
     declaration each implemented family lands in is tools/ucrt-takeover.yaml.
     Claims here are cited to file:line -- re-derive before acting on one. -->

# ucrtbase `string/` family — MSVCRT vs UCRT delta research

Repo: `/home/curi/Projects/os-playground/vibeos-msvcrt-homebrew` (branch `msvcrt-homebrew`).
Scope: the 95 `string/` exports that already have a Rust body (written/graded against
MSVCRT, `_MSVCR_VER=0`), plus the 4 that don't. Oracle = the two winetest files:

- `modules/rostests/winetests/ucrtbase/string.c` (860 lines) — real UCRT behaviour
- `modules/rostests/winetests/msvcrt/string.c` (5223 lines) — real MSVCRT behaviour

Vendored MS source (`sdk/lib/ucrt/string/*.cpp`, `.s`) was read to understand contracts
only — none of it is transcribed below, per licence.

---

## 1. The 4 missing exports

| export | what it does | difficulty | winetest coverage |
|---|---|---|---|
| `_strset_s` | bounded `_strset`: fills a buffer up to `size_in_elements` with a byte value, NUL-terminating; secure-CRT return contract (0/EINVAL/ERANGE). Source (`sdk/lib/ucrt/string/strset_s.cpp`) just forwards to a shared `common_tcsset_s<char>` template. | **Low.** Same shape as `_strnset_s`/`_wcsnset_s`, which we already have; the template is generic over `char`/`wchar_t` in the MS source, and our two existing `_s`-family bodies are the pattern to copy. | **None found.** Neither winetest file has a `test__strset_s`/`test_strset_s`. `_wcsset_s` IS tested (`modules/rostests/winetests/msvcrt/string.c:3990`, `test__wcsset_s`) but that's the wide twin, already in `have`. Implement by symmetry with `_wcsnset_s`/`_wcsset_s` and difftest against the MS contract directly since there's no winetest net. |
| `__wcsncnt` | wide twin of `__strncnt`: count of non-NUL wchars up to `count`, source `sdk/lib/ucrt/string/wcsncnt.cpp`. | **Low**, mechanically — but see the §2 delta on `__strncnt`'s bound-check ORDER below; whichever arm we pick for `__strncnt` must be mirrored here, since `wcsncnt.cpp`'s `for (; *it && n != count; ++it, ++n)` is the `size--`-first (bound-checked-before-deref) shape, i.e. it does NOT show the `_MSVCR_VER>=140` re-ordering our `__strncnt` comment warns about. **This one MS source file, read directly, shows bound-check-before-deref** (safe on `(NULL,0)`), unlike the narrow `__strncnt`'s documented `_MSVCR_VER>=140` arm. Worth one extra look before assuming symmetry — see §2. | **None found.** No `test___wcsncnt`/`wcsncnt` test in either file. |
| `wmemcpy_s` | secure `wmemcpy`: copies `count` **wchar_t elements** (not bytes) from `source` to `destination`, bounded by `size_in_elements`; on overflow zeroes `destination` first, then returns EINVAL/ERANGE. Non-overlap contract (like `memcpy_s`). | **Low.** Structurally identical to `memcpy_s` (which we have) with the element type/size swapped and one extra "zero-then-fail" step that `memcpy_s` also has — copy that pattern. | **None found.** No `wmemcpy_s` test in either winetest file. |
| `wmemmove_s` | secure `wmemmove`: same as above but overlap-safe (uses `wmemmove`), and — note — does **not** zero the destination on failure (`sdk/lib/ucrt/string/wmemmove_s.cpp` has no zero-fill branch, unlike `wmemcpy_s`). | **Low.** Mirror `memmove_s` (have), element-sized. | **None found.** No `wmemmove_s` test in either winetest file. |

**None of the 4 missing exports have ANY winetest coverage in this tree.** All four should
be implemented by structural symmetry with an existing sibling we already have and graded
by `msvcrt-difftest`/manual reasoning against the MS contract, not by a winetest net.

---

## 2. The delta list — where UCRT and MSVCRT disagree (the 95 already-ported exports)

### 2.1 `strncmp` — **CRITICAL, confirmed, arch-dependent contract flip**

Our existing Rust body (`sdk/lib/rust/msvcrt-crt-rust/src/lib.rs:1012-1044`) is *already
self-documented* as arch-dependent for MSVCRT:

- 64-bit (`_WIN64`) → clamped **-1/0/1**
- i386 → **raw** unsigned-byte difference (unclamped)

This matches the vendored MSVCRT contract. But the UCRT winetest
(`modules/rostests/winetests/ucrtbase/string.c:459-472`, `test_strcmp`) has **no `#ifdef`
at all** — it unconditionally expects the clamped form on every arch:

```
ret = strncmp( "", "abc", 3 );      ok( ret == -1, ... );   /* would be 0-'a'=-97 raw */
ret = strncmp( "abc", "ab\xa0", 4 );ok( ret == -1, ... );   /* would be 'c'-0xa0 raw */
ret = strncmp( "ab\xb0", "ab\xa0", 3 ); ok( ret == 1, ... );/* would be 0xb0-0xa0=16 raw */
```
compare to the arch-split MSVCRT test at `modules/rostests/winetests/msvcrt/string.c:735-751`
(`#ifdef _WIN64` clamped / `#else` raw-diff).

Confirmed against the real UCRT asm (read for contract only, not transcribed):
- `sdk/lib/ucrt/string/i386/strncmp.s` — clamps (`sbb eax,eax; or eax,1`); its own header
  comment even says "the returned value is limited to {-1, 0, 1}".
- `sdk/lib/ucrt/string/amd64/strncmp.s` — same clamp idiom at line 125.
- `sdk/lib/ucrt/string/strncmp.c` (the generic C fallback, built for arm64 per
  `build-arm64/.../ucrt.dir/string/strncmp.c.obj`) computes the **raw** difference with no
  clamp at all — this conflicts with the winetest's unconditional clamped expectation, so
  either that fallback is a ReactOS reconstruction inaccuracy for real Windows ARM64 UCRT,
  or ARM64 wasn't what the winetest was captured against. **Flagging this as unresolved**
  rather than guessing: I could not independently confirm real Windows ARM64 UCRT `strncmp`
  clamps, only that i386/amd64 UCRT asm and the arch-independent winetest both say it does.

**Action for the Rust ucrtbase body: `strncmp` must clamp to -1/0/1 unconditionally on
ALL THREE arches**, not just 64-bit. Reusing the current `#[cfg(target_pointer_width)]`
body verbatim under `--allow-multiple-definition` would silently ship the wrong (raw-diff)
i386 contract into ucrtbase.

### 2.2 `wcstok` — **CRITICAL, confirmed, ABI/arity mismatch**

Our existing Rust `wcstok` (`lib.rs:3838-3852`) is **two-argument** (`str, delim`), backed
by per-thread state (`msvcrt_get_thread_data()->wcstok_next`) — this is explicitly
documented in its own doc comment: *"At `_MSVCR_VER == 0` this is the TWO-argument form
[...] The three-argument `wcstok` only exists at `_MSVCR_VER >= 140`; do not port that
one."* That warning is exactly the bug this quest exists to prevent, and it is currently
unheeded for ucrtbase, because there is no separate ucrtbase `wcstok`.

UCRT's `wcstok` **is** the three-argument form — confirmed directly by the ucrtbase
winetest, which never uses the two-arg call at all:

```
token = wcstok(buffer, L" ", &next);     /* modules/rostests/winetests/ucrtbase/string.c:325 */
...
token = wcstok(buffer, L" ", NULL);      /* :335 — context arg is optional-but-present, not absent */
```
(`test_wcstok`, `modules/rostests/winetests/ucrtbase/string.c:314-349`.)

**Action:** ucrtbase's `wcstok` is a different symbol contract from msvcrt's `wcstok` and
cannot share the existing body. It should essentially be `wcstok_s` (which we already
have, `lib.rs` — same 3-arg shape) re-exported/aliased as `wcstok` for ucrtbase, not the
2-arg per-thread-state version. Calling the existing 2-arg `wcstok` from ucrtbase-linked
code that passes 3 args is an actual stack-arg-count mismatch, not just a semantic
difference — this is the most severe class of delta in the whole family.

### 2.3 `__strncnt` — **CRITICAL, confirmed, dereference-vs-bound-check ordering**

This is *also already self-documented as a known risk* in the existing Rust body
(`lib.rs:1538-1556`), which currently implements the **MSVCRT (`_MSVCR_VER=0`)** arm:
bound checked before dereference (`while i < size && *p.add(i) != 0`) — safe on
`(NULL, 0)`.

Confirmed via both winetests:
- MSVCRT (`modules/rostests/winetests/msvcrt/string.c:4506-4537`, `test___strncnt`) lists
  `{ NULL, 0, 0 }` as an ordinary, **non-crashing** test case — matches bound-check-first.
- UCRT (`modules/rostests/winetests/ucrtbase/string.c:266-296`, `test___strncnt`)
  explicitly does **not** run that case — it's guarded behind `if (0) /* crashes */ { ret =
  __strncnt(NULL, 0); ret = __strncnt(NULL, 1); }` — i.e. UCRT's `__strncnt(NULL, 0)` is
  documented by the wine test authors as a crash, meaning the real UCRT `__strncnt`
  dereferences at least once before (or regardless of) the `size` bound check.

This matches the Rust comment's own prediction: the `_MSVCR_VER >= 140` arm is
`*str++ && size--` (dereference first, decrement bound second, short-circuited only after
the read), vs. the `_MSVCR_VER == 0` arm `size-- && *str++` (bound checked first). Our
current body is the safe MSVCRT arm; ucrtbase needs the UCRT arm, which will fault on
`(NULL, 0)` by design (matching real UCRT) rather than returning 0.

**Action:** ucrtbase's `__strncnt` needs the dereference-before-bound-check ordering. Same
applies to the still-unported `__wcsncnt` (§1) — worth explicitly re-checking against the
real MS `wcsncnt.cpp` source at implementation time, because that file (read directly, see
§1) shows the OTHER (bound-check-first) ordering, which would make `__wcsncnt` NOT share
this delta even though `__strncnt` does. Don't assume symmetry here — verify per function.

### 2.4 Export-surface note (not a behavioural delta, but a wiring trap)

`ucrtbase.def` (all three arches, checked in `build-amd64/dll/win32/ucrtbase/ucrtbase.def`)
exports **only the underscored name** for the Microsoft-nonstandard members of this family
— there is no unprefixed alias in ucrtbase for: `strupr`, `strlwr`, `strset`, `strnset`,
`strrev`, `stricmp`, `strnicmp`, `strdup`, `memicmp`, `strncoll`, `stricoll`, `strnicoll`,
`memccpy` (all present only as `_strupr`, `_strlwr`, ... `_memccpy`). ISO-standard members
keep the unprefixed name in both DLLs (`strtok`, `strcpy`, `strcat`, `strncpy`, `strncat`,
`strcmp`, `strncmp`, `strcoll`, `strxfrm`, `strnlen`, `strcspn`, `strpbrk`, `strspn`).
Checked our Rust symbols for the nonstandard set — they are already `#[no_mangle]`'d under
the underscored name (e.g. `_strupr` at `lib.rs:398`, `_strdup` at `string_l.rs:38`), so
this is **not currently a bug**, just a trap for whoever wires the ucrtbase `.def`/exports
list: do not add an unprefixed alias for this set, ucrtbase genuinely doesn't have one.

### 2.5 `_memicmp` / `_memicmp_l` — checked, appears IDENTICAL

UCRT test (`ucrtbase/string.c:187-256`, `test__memicmp`/`test__memicmp_l`) and MSVCRT test
(`msvcrt/string.c:4120-4193`) both: return `_NLSCMPERROR` + set `errno = EINVAL` + fire the
invalid-parameter handler on any NULL pointer with `len != 0`; return 0 quietly on
`(NULL, NULL, 0)`. The Rust body's own comment at `lib.rs:2930-2933` ("`_MSVCR_VER == 0` ->
this guard IS compiled in") reads at first as a version-gate warning like §2.1-2.3, but
cross-checked against both winetests it is NOT a behavioural fork — both DLLs show the
identical guard. No action needed here, but flagging the comment as worth a second look by
whoever owns this file, since it uses the same phrasing pattern as the three real deltas
above.

### 2.6 `_strnicmp` invalid-parameter behaviour — checked, appears identical in spirit, not independently confirmed on UCRT-vs-MSVCRT split

UCRT (`ucrtbase/string.c:495-522`, `test__strnicmp`) fires the invalid-parameter handler
and returns `_NLSCMPERROR`/`EINVAL` for: NULL second string, negative count, and count
`>= 0x80000000`. I did **not** find an equivalent MSVCRT `_strnicmp` test exercising the
same negative/huge-count edge cases in `msvcrt/string.c` (only `test__strnicmp_l` at
`:4933`, which is the `_l` twin, and doesn't cover this). **I cannot confirm from the
winetests alone whether this specific edge-case handling differs between the two DLLs** —
noting this as an open question rather than asserting identity or difference. Worth a
`msvcrt-difftest` probe (compile the same test body against both) before shipping a shared
implementation unreviewed.

### 2.7 `wcsnicmp` — checked, negative/huge count does NOT trigger the invalid handler

`ucrtbase/string.c:524-534` (`test_wcsnicmp`) explicitly calls `wcsnicmp(str1, str2, -1)`
and `wcsnicmp(str1, str2, 0x7fffffff)` and expects a normal `0` result, **no**
`SET_EXPECT(invalid_parameter_handler)` around either call — unlike `_strnicmp` (§2.6) one
test function above it in the same file, which DOES install the handler for the same shape
of oversized/negative count. This is an intra-UCRT asymmetry between `_strnicmp` (narrow,
validates count) and `wcsnicmp` (wide, does not) — worth confirming our existing `wcsnicmp`
body doesn't add a count validation that isn't there in the real contract. I did not find a
narrow-vs-wide inconsistency check already in the Rust source for this pair; flagging for
the implementer to verify `wcsnicmp`/`wcsnicoll` don't share the `_strnicmp`-family EINVAL
path.

---

## 3. Exports checked and found identical (MSVCRT == UCRT, both by winetest and/or MS source read)

Narrow compare/copy/scan core: `strcmp`, `strcpy`, `strcspn`, `strlen`, `strncpy`,
`strpbrk`, `strspn`, `memccpy`.

Secure (`_s`) family, checked structurally via MS source (no `_MSVCR_VER` gates found in
any of: `memcpy_s.cpp`, `strcpy_s.cpp`, `strncpy_s.cpp`, `wcscpy_s.cpp`, `wcsncpy_s.cpp`,
`strcat_s.cpp`, `strncat_s.cpp`, `wcscat_s.cpp`, `wcsncat_s.cpp`, `strnset_s.cpp`,
`wcsnset_s.cpp`, `wcsset_s.cpp`) and cross-checked against winetest for `strcpy_s`,
`memcpy_s`, `memmove_s`, `strcat_s`, `strncat_s`, `wcsncat_s`, `_strnset_s`(wide sibling
`_wcsnset_s`), `_wcsset_s` (all in `modules/rostests/winetests/msvcrt/string.c`): return
codes (0/EINVAL/ERANGE/STRUNCATE) and buffer-zeroing-on-failure behaviour match what the
MS source does unconditionally (no version fork visible).

Case-mapping: `strupr`/`_strupr_l`/`_strupr_s`/`_strupr_s_l`, `strlwr` family, `wcsupr`
family, `wcslwr` family — no `#if` in any of `strupr.cpp`, `strlwr.cpp`, `wcsupr.cpp`,
`wcslwr.cpp`.

Collation/transform: `strcoll`/`strcoll_l`, `strncoll`/`strncoll_l`, `stricoll`/
`stricoll_l`, `strnicoll`/`strnicoll_l`, `strxfrm`/`strxfrm_l` — no `#if` in
`strcoll.cpp`, `strncoll.cpp`, `stricoll.cpp`, `strxfrm.cpp`. (Their `_l`/locale-table
DEPENDENCY is real and important — see §4 — but it's not a MSVCRT-vs-UCRT delta, both
implementations go through the same locale machinery.)

`strdup`/`wcsdup` behaviour (allocate + copy; NULL-in → NULL-out): no logic fork, only
`#ifdef _DEBUG` (irrelevant to release), confirmed in `strdup.cpp`/`wcsdup.cpp`. (Export
*naming* does differ — see §2.4 — but the copied bytes/return value do not.)

`_memicmp`/`_memicmp_l` — see §2.5 (checked, identical).

`strtok`/`strtok_s`, `strnset`/`strset` (unbounded legacy forms), `strrev`, `wcscat`,
`wcscmp`, `wcscspn`, `wcsncat`, `wcsncmp`, `wcsnset`, `wcspbrk`, `wcsrev`, `wcsset`,
`wcsspn` — no version-gated behaviour found in the corresponding `.cpp`/`.s` sources or in
either winetest.

`__strncnt`'s narrow-vs-wide asymmetry aside (§2.3/§1), the count-clamping shape of
`strnlen`/`wcsnlen` themselves (as opposed to `__strncnt`) showed no `_MSVCR_VER` fork in
source or winetest.

---

## 4. Things that surprised me

- **The two most dangerous deltas in this family were already flagged by the previous
  translator**, in doc comments on the very functions at risk (`strncmp` at `lib.rs:996-1005`,
  `__strncnt` at `lib.rs:1538-1546`, `wcstok` at `lib.rs:3843-3846`) — they just describe the
  *MSVCRT* contract correctly and warn "don't port the other one" without ucrtbase existing
  yet to receive it. This is exactly the failure mode the quest is designed to catch: correct,
  well-documented code for the wrong `_MSVCR_VER`, sitting one `--allow-multiple-definition`
  link away from silently wrong behaviour in ucrtbase. I'd treat these three doc comments as
  the highest-value places to re-read before writing any ucrtbase string.c Rust.

- **`wcstok`'s delta is an arity change, not a value change.** Every other delta here is
  "same call shape, different numeric contract." `wcstok` genuinely takes a different number
  of arguments between the two DLLs. A shared-body reuse here isn't "subtly wrong," it's a
  miscompiled call at every UCRT call site that passes the third (context) argument.

- **`strncmp`'s delta is per-arch AND per-DLL simultaneously** — i386-MSVCRT is the one
  cell in the 2×3 (DLL × arch) grid that's unclamped; every other cell clamps. Easy to get
  half right (e.g. "ucrtbase clamps on i386 like the other two arches" is true, but if
  implemented by just deleting the `#[cfg]` split and always clamping, don't forget that
  *msvcrt's* i386 arm still needs to stay unclamped — these are two separate DLLs built from
  the same crate family, not one flip.)

- **I could not verify real Windows ARM64 UCRT `strncmp`** (§2.1) — the vendored C fallback
  built for arm64 in this tree contradicts the arch-independent winetest, and I don't have a
  way to run real Windows ARM64 to settle it. Flagging rather than guessing, per the task's
  own instruction.

- **`_s` secure variants**: did not find a single case where a `_s` function's
  invalid-parameter-handler/errno/return-code contract differs between the two DLLs — every
  one I checked (§3) forwards through the same shared validation macros in the MS source
  with no `_MSVCR_VER` gate. This family's secure variants look like a "free" 20-ish exports
  once the three critical items above are fixed.

- **`_l` locale-suffixed twins**: no MSVCRT-vs-UCRT fork found in the locale-table-touching
  code itself (collation, case folding) — the risk there (locale resolved once vs. per-byte,
  hot-path perf) is already handled by the existing implementation per its own comments
  (`lib.rs:3894-3910`+ wave-11 section), and is a *performance* contract, not a
  MSVCRT-vs-UCRT semantic delta. Did not find a case where UCRT's `_l` twin takes a
  different locale-table path than MSVCRT's.

- **§2.6 and §2.7 are real open questions, not findings** — I'm flagging them explicitly as
  "could not determine from the available oracles" rather than asserting an answer, since a
  wrong guess here is exactly the kind of thing that costs a paid debugging cycle later.
