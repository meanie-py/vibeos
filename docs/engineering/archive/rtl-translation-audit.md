> ⚠️ **ARCHIVED 2026-07-03 — superseded by [`rust-port-status.md`](../rust-port-status.md).**
> This per-surface audit is a point-in-time historical record. Its per-file counts were
> measured ~2026-06-21 (before the last swarm wind-down) and are **stale** — several
> "0% / in-progress" claims here are now done. For current Rust-vs-C coverage use the
> status doc; this file is kept only for the detailed per-file mapping and history.

# RTL (sdk/lib/rtl) C→Rust Translation Audit

> **Translation: ~99% (methodology: by source `.c` files in `sdk/lib/rtl`).** Of 82 `.c`
> files in the tree, **80 are flipped to Rust** (cohabitation-commented in
> `sdk/lib/rtl/CMakeLists.txt` — **83 `# … — replaced by Rust` markers**, counting the
> per-arch `except.c`/`thread.c`/`unwind.c`/`dynfntbl.c`/`stubs.c` variants). The only
> active (uncommented) `.c` left are the irreducible floor: `dbgprint_variadic.c` (180 LOC,
> C-variadic FFI trampolines that back `rtl-diag`'s `DbgPrint*`/`wine_dbg_*`) and
> `rtl_vista_stub.c` (17 LOC placeholder) — plus ~2.35K LOC of per-arch asm
> (`*/debug_asm.S`, `i386/{except_asm.s,rtlmem.s,rtlswap.S}`, `amd64/{except_asm.S,unwind-asm.s}`).
> No `Rtl*` API *body* remains in C.

> **Ground-truth re-verified 2026-06-21** against `sdk/lib/rtl/CMakeLists.txt` + the `sdk/lib/rust/` crates — **the whole `sdk/lib/rtl/*.c` surface is now flipped to Rust**, including the previously-"deferred" `registry.c` (→ `rtl-misc`), the entire `heap.c` family (`heap.c`/`heapdbg.c` → `rtl-heap`, `heappage.c` → `rtl-heappage`, `heapuser.c` → `rtl-heapuser`), `image.c` (→ `rtl-diag`), and the per-arch exception/unwind tail (→ `rtl-except`). The **only active (uncommented) C/asm left in `librtl.a`** is the irreducible floor: `dbgprint_variadic.c` (C-variadic FFI glue), `rtl_vista_stub.c` (placeholder), and the per-arch asm `*/debug_asm.S`, `i386/{except_asm.s,rtlmem.s,rtlswap.S}`, `amd64/{except_asm.S,unwind-asm.s}`. The "Still C" / "deferred" claims below are SUPERSEDED — corrected inline.
>
> **Rust footprint (2026-06-21):** 20 `rtl-*` crates + the `rtl-rust` umbrella —
> **94 `.rs` files, ~56.7K LOC** (largest: `rtl-strings` 22 files/~15.9K LOC,
> `rtl-actctx` ~7.7K, `rtl-difftest` ~5.5K test harness, `rtl-heap` ~4.8K,
> `rtl-threadpool` ~4.1K, `rtl-except` ~3.2K, `rtl-sync` ~3.0K). Per-crate table in §4.

**Surface:** `sdk/lib/rtl/*.c` — the NT `Rtl*` run-time library that compiles
into the single `librtl.a`, which links into **both** `ntoskrnl.exe` (kernel,
`-D_NTOSKRNL_`) **and** `ntdll.dll` (user mode). One archive, two consumers.

**Created:** 2026-06-01 (consolidates the prior `project_rtl_strings_track`
memory + the `handoff-rtl-libs-next-steps.md` codex handoff into an in-repo
ledger). Sister docs: `ntoskrnl-translation-audit.md`,
`ntdll-translation-audit.md`, `static-libs-translation-audit.md`.

**Status (2026-06-21):** the entire rtl flip is **DONE** and landed on mainline
(`continue-rust-port` == `master`). The `rtl-priv-port` lane that drove the original
string/path waves is closed; all 20 `rtl-*` crates are integrated and live in
`sdk/lib/rtl/CMakeLists.txt`. The historical lane notes below are kept for provenance.

---

## TL;DR

- The byte-diffable + translate-only string/security/process/env/path surface
  of `sdk/lib/rtl` is **flipped to Rust** (crate `sdk/lib/rust/rtl-strings/`,
  cohabitation-commented in `sdk/lib/rtl/CMakeLists.txt`). ~20 `.c` files.
- The pure-algorithm surface (bitmap, avltable, splaytree, crc32, …) was
  already carved into sibling rlibs under `rtl-rust` — tracked separately in
  `static-libs-translation-audit.md`, **not here**.
- **path.c (3040 LOC, the largest single file) is flipped as of this
  session** (27/27 fns). Flipping it advanced amd64 boot far enough to expose
  — and this session **fix** — three latent downstream bugs in the
  (already-committed) process-creation / env path. amd64 now reaches
  `[WIN32K] shell ready`; the current blocker is a **services.exe init hang**
  (post-`shell ready`), still under investigation.
- **UPDATE 2026-06-14:** `registry.c`, the `heap.c` family, `image.c`, and the
  misc remainder are **all flipped now** — see the Ground-truth marker at the
  top. The TL;DR below is preserved as a point-in-time (2026-06-01) record.

---

## Three rtl-ish surfaces — don't confuse them

The single biggest source of "wait, which rtl?" confusion. There are three
distinct axes, and **this doc is only axis (3)**:

1. **`ntoskrnl-rust/src/rtl/`** — tiny kernel-private glue (libsupp / misc /
   prefetch). Part of the ntoskrnl port → `ntoskrnl-translation-audit.md`.
2. **`rtl-rust` umbrella + ~18 algorithm rlibs** — the pure-algorithm core of
   sdk/lib/rtl carved out (bitmap, avltable, splaytree, generictable, crc32,
   compress, encode, largeint, time, random, luid, handle, interlck, prefix,
   access, mem, byteswap, slist, utf8, uniprefix). →
   `static-libs-translation-audit.md`.
3. **The rest of `sdk/lib/rtl/*.c`** — the `Rtl*` API bodies (strings,
   security, env, process, thread, path, …) translated into the
   **`rtl-strings`** crate (`core_rs` pure + `cabi` `#[no_mangle]` shims).
   **← THIS DOC.**

`cnt-rust`/`libcntpr` (the C `_LIBCNT_` stdlib) is a *fourth*, orthogonal axis
— rtl-strings reuses its `core_rs::mbcp` CP1252 model as the one-truth NLS
core, but they are separate tracks.

---

## Cohabitation mechanism

- `librtl` has **NO `--allow-multiple-definition`** (unlike HAL). A `.c` is
  whole-file-commentable in `sdk/lib/rtl/CMakeLists.txt` **only when Rust
  covers 100% of that file's exports.** Partial flips don't link.
- `rtl-strings` is **one crate** spanning many `.c` files, so a swap is
  all-or-nothing across whatever set of files share unresolved symbols.
- Kernel vs user: `librtl` is built `-D_NTOSKRNL_`. rtl-strings mirrors this
  with a `kernel` cargo feature (e.g. `nls_tables.rs`, SxS branches gated
  `#[cfg(not(feature="kernel"))]`). `rtl-rust` enables `features=["cabi",
  "kernel"]` for the kernel `librtl`; the diff harness uses `["cabi"]` only.
- PEB/TEB/heap access from kernel-linked rtl uses **libsupp accessor shims**
  (`{ntoskrnl,dll/ntdll}/rtl/libsupp.{c,rs}` + the `RtlpGetPebEnvironment`,
  `RtlpPrcPebProcessParametersAddr`, … family) so the Rust never mirrors the
  per-arch PEB layout. **These shims are a recurring bug source — see §4.**

---

## 4. Current status (2026-06-21)

### Rust crate footprint (measured from tree, 2026-06-21)

| Crate | `.rs` | LOC | Covers |
|---|---|---|---|
| rtl-strings | 22 | 15,907 | unicode/nls/dos8dot3/sid/acl/sd/priv/tz/env/sec/res/thr/prc/atom/path (the `Rtl*` API bodies) |
| rtl-actctx | 5 | 7,692 | actctx.c (SxS activation contexts) |
| rtl-difftest | 3 | 5,504 | byte-diff fuzz harness (host-only, not linked) |
| rtl-heap | 4 | 4,772 | heap.c + heapdbg.c |
| rtl-threadpool | 6 | 4,146 | threadpool.c / timerqueue.c / workitem.c |
| rtl-except | 12 | 3,192 | per-arch exception/unwind tail (i386/amd64/arm64) |
| rtl-sync | 9 | 2,957 | critical/wait/srw/condvar/runonce/locale |
| rtl-diag | 12 | 2,372 | image/debug/assert/trace/message/exception/wine_debug/… |
| rtl-misc | 5 | 2,359 | registry/res/memstream |
| rtl-heappage | 2 | 2,352 | heappage.c |
| rtl-error | 3 | 2,188 | error.c |
| rtl-rxact | 1 | 658 | rxact.c |
| rtl-sysvol | 1 | 632 | sysvol.c |
| rtl-rangelist | 1 | 561 | rangelist.c |
| rtl-ppb | 1 | 352 | ppb.c |
| rtl-version | 1 | 332 | version.c |
| rtl-bootdata | 1 | 255 | bootdata.c |
| rtl-propvar | 1 | 184 | propvar.c |
| rtl-heapuser | 2 | 143 | heapuser.c |
| rtl-rust (umbrella) | 1 | 96 | re-exports the algorithm rlibs + cabi feature wiring |
| rtl-vista-rust | 1 | 26 | vista stub crate |
| **TOTAL** | **94** | **56,680** | (rtl-difftest is host-test-only; algorithm rlibs counted in static-libs audit) |

### Flipped to Rust (rtl-strings cabi) — cohabitation-commented in CMakeLists

| File | Wave | Net | Notes |
|---|---|---|---|
| unicode.c / nls.c / dos8dot3.c | RTL-1…6b | byte-diff | strings + NLS (shared CP1252 model). `5593d886` |
| sid.c | SID-1+2 | byte-diff | pointer-free, arch-neutral |
| acl.c | ACL-1+2 | byte-diff | ACL/ACE pointer-free |
| sd.c | SD-1+2+swap | byte-diff | has pointers (abs form); per-arch SD size |
| priv.c | PRIV | translate-only | pure Zw* plumbing |
| timezone.c | TZ | translate-only | Rtl{Query,Set}TimeZoneInformation |
| env.c | ENV | translate-only | env block; **RtlExpandEnvironmentStrings_U** lives here |
| security.c | SEC | byte-diff | 9 RtlHeap edges |
| resource.c | RES | translate-only | SRW-style locks |
| thread.c | THR | translate-only | **RtlCreateUserThread** lives here |
| process.c | PRC | translate-only | **RtlCreateUserProcess / RtlpInitEnvironment** |
| atom.c | ATM | layout-mirror | `d54a073a4bf` (was the deferred "rabbit hole", now done) |
| **path.c** | **PATH** | **translate + smoke** | 27/27 fns — **committed; flip is live in CMakeLists** |

> **2026-06-14:** the table above is the 2026-06-01 wave snapshot. Since then the
> flip completed across the *entire* `sdk/lib/rtl/*.c` surface — additional Rust
> homes now live in CMakeLists: `rtl-misc` (registry/res/memstream), `rtl-heap`
> + `rtl-heappage` + `rtl-heapuser` (the heap family), `rtl-diag` (image/debug/
> assert/trace/wine_debug/… — variadic shims kept in `dbgprint_variadic.c`),
> `rtl-except` (per-arch exception/unwind tail), `rtl-sync` (critical/wait/srw/
> condvar/runonce/locale), `rtl-threadpool`, `rtl-actctx`, `rtl-error`,
> `rtl-rangelist`, `rtl-propvar`, `rtl-ppb`, `rtl-rxact`, `rtl-sysvol`,
> `rtl-version`, `rtl-bootdata`, `network`. Authoritative list = grep the
> `# <file>.c — replaced by Rust (...)` comments in `sdk/lib/rtl/CMakeLists.txt`.

(Algorithm crates — bitmap/bitmap64, avltable, splaytree, unicodeprefix,
crc32, compress, encode, largeint, time, random, luid, handle, interlck,
prefix, access, mem, byteswap, slist, utf8 — also commented out here but are
tracked in `static-libs-translation-audit.md`.)

### Still C in sdk/lib/rtl

> **SUPERSEDED AGAIN 2026-06-14 — NOTHING in the table below is C anymore.**
> registry.c → `rtl-misc`, heap.c+heapdbg.c → `rtl-heap`, heappage.c →
> `rtl-heappage`, heapuser.c → `rtl-heapuser`, image.c → `rtl-diag`, and the
> misc remainder are all flipped + active in `sdk/lib/rtl/CMakeLists.txt`. The
> only active C/asm left is the irreducible floor (see the Ground-truth marker
> at the top of this file). The table at the end of this subsection is kept for
> history only — every row is now Rust.
>
> **SUPERSEDED 2026-06-09 — rtl is ~fully Rust now: registry.c (fc4563c30ba), image.c (rtl-diag), heap.c+heapdbg (rtl-heap), heappage/heapuser translated; only heappage/heapuser/arch-asm remain active C in CMakeLists.**
>
> **SUPERSEDED AGAIN 2026-06-11 — the per-arch exception/unwind tail is now Rust
> too** (`sdk/lib/rust/rtl-except/`): `amd64/{unwind,except,stubs,dynfntbl}.c`,
> `i386/{except,thread}.c`, `arm64/except.c` (7 files, ~3.1K LOC — including the
> x64 RtlVirtualUnwind/RtlUnwindEx unwinder state machine). Smoke-verified ×3.
> The only active C left in librtl is `dbgprint_variadic.c` (deliberate
> C-variadic FFI glue) + the `.s/.S` asm (trampolines/capture-context) +
> `rtl_vista_stub.c` (placeholder). **NOTE the stale-fat-archive trap found
> during this flip:** the cmake `ar` rule appends without truncating, so
> `librtl.a` had been accumulating ZOMBIE objects of every previously-flipped
> `.c` (heappage.c.obj, thread.c.obj, …) which silently kept satisfying symbols
> (`RtlpPageHeapEnabled`, `NtCurrentTeb`, `RtlFillMemoryUlong`, `_swprintf`).
> If a flip "links clean" suspiciously easily, `rm librtl.a && ninja` and check
> `ar t` for members that should be gone. Fixes landed with this wave:
> rtl-heappage `#[no_mangle]` on RtlpPageHeapEnabled/RtlpDphGlobalFlags +
> inline NtCurrentTeb, mem crate RtlFillMemoryUlong enabled on amd64,
> ntdll-cargo compiles `_swprintf.c`.

> *(Historical table — every row below is now FLIPPED to Rust as of 2026-06-14. "Why still C" reflects the 2026-06-01 state, with the current Rust home appended.)*

| File | LOC | 2026-06-01 rationale → **NOW** |
|---|---|---|
| **registry.c** | 1351 | RtlQueryRegistryValues etc. **NOW Rust → `rtl-misc/src/registry.rs`** (Wave MISC). |
| heap.c (+ heapdbg/heappage/heapuser) | large | stateful allocator (locks/VM/free-lists), was deferred to behavioral/Kani track. **NOW Rust → `rtl-heap` (heap.c+heapdbg.c), `rtl-heappage`, `rtl-heapuser`.** |
| image.c | — | shared PE format primitives (`RtlImageNtHeader`, …). **NOW Rust → `rtl-diag/src/image.rs`** (C-variadic shims in `dbgprint_variadic.c`). |
| misc remainder | — | bootdata, generic small files. **NOW Rust** (`rtl-bootdata`, `rtl-misc`, etc.). |

### Boot state

> **RESOLVED — all three arches boot green on mainline (2026-06-21).** The
> rtl flip (path.c + the full surface) is fully landed; i386/amd64/arm64 each
> reach the desktop (`shell ready`) in smoke. The amd64 `services.exe` init
> hang recorded below was a **2026-06-01 point-in-time state of the
> `rtl-priv-port` session** — it has since been cleared (the deeper amd64 boot
> stalls were the timer-base skew + scsiport LUNEX/find-loop bugs, root-caused
> in unrelated lanes, not an rtl defect). The snapshot is kept for history.

> **Point-in-time snapshot (2026-06-01); the services.exe-hang described below is NO LONGER current — see above.**

`./vibeos-cli smoke --arch amd64` reached **`[WIN32K] shell ready`** — the
full kernel → smss → csrss → win32k chain booted with path.c live. Winlogon
then started, switched desktop, and launched `services.exe`, which **hung
during init** (PID 128 loading apphelp/setupapi/newdev; no msgqueue spam;
clean hang) → smoke verdict `HANG_TIMEOUT`. **This was the 2026-06-01 amd64
blocker (now cleared).**

---

## The PATH wave (this session) — flip exposed 3 latent bugs

Flipping path.c was *correct*: instrumented boot confirmed
`RtlGetFullPathName_Ustr` / `RtlDosPathNameToNtPathName_U` convert paths
perfectly (`X:\reactos\winsxs\manifests` → `\??\X:\reactos\winsxs\manifests`).
The flip simply let amd64 boot reach the (committed) user-process-creation and
env code paths — which had **never been exercised on amd64** before — and
those carried three latent bugs. All three are fixed this session; all three
are all-arch:

1. **`RtlpPrcPebProcessParametersAddr` kernel stub returned NULL**
   (`ntoskrnl-rust/src/rtl/libsupp.rs`). It only computes
   `&((PPEB)peb_base)->ProcessParameters` (pointer arithmetic on the *child*
   PEB user-VA — no deref, valid in kernel). The stub returned NULL →
   `RtlpInitEnvironment`'s 3rd `ZwWriteVirtualMemory` wrote 8 bytes to address
   0 → uncatchable under DUMMY_PSEH → **BUGCHECK 0x1E**. Diagnosed via CR2=0 +
   page-table walk (`monitor xp/1gx`; VA 0 P=0). Fix = real pointer arithmetic
   on the bindgen `_PEB`.

2. **`RtlCreateUserThread` passed `&initial_teb` instead of
   `initial_teb.StackBase`** to `RtlInitializeContext` (`rtl-strings/cabi`).
   The C `RtlInitializeContext` 5th param is mistyped `PINITIAL_TEB` but used
   as `(ULONG_PTR)StackBase`; the original C caller (`sdk/lib/rtl/thread.c`)
   passes `InitialTeb.StackBase`. The Rust passed the struct address → new
   thread's user `Rsp` = a kernel-stack address → `KiInitializeUserApc` probe
   raised AV → **BUGCHECK 0x1E**. Confirmed via trap frame (Rip = smss entry
   ✓, Rsp = kernel stack ✗). Fix = pass `initial_teb.StackBase`.

3. **`RtlExpandEnvironmentStrings_U` ignored the "NULL = current process
   environment" contract** (`rtl-strings/cabi`, env wave). C passes
   `Environment` to `RtlQueryEnvironmentVariable_U`, which resolves NULL to
   `NtCurrentPeb()->ProcessParameters->Environment`. The Rust reimplemented
   the lookup via the pure `expand_env_strings(env_slice)` and mapped NULL →
   empty `&[]`. `RtlQueryRegistryValues` (still-C) expands `REG_EXPAND_SZ` by
   calling it with NULL → empty env → smss's KnownDll `DllDirectory` value
   `%SystemRoot%\system32` was left **unexpanded** → `NtOpenFile` returned
   `c000003a OBJECT_PATH_NOT_FOUND` → `SmpInit` failed → smss (critical
   process) died → **BUGCHECK 0x4C**. Fix = NULL → `RtlpGetPebEnvironment()`.
   (smss's env *did* contain `SystemRoot=X:\reactos`; expansion was simply
   skipped.)

> **Historical correction:** the 2026-05-28 handoff attributed a
> `c000003a`-family boot failure to an "upstream PnP / uniata IDE-binding
> bug." At least the smss-KnownDll variant was, in fact, bug #3 above (a cabi
> translation defect), **not** upstream. The old 0x6B ArcName/CdRom symptom no
> longer reproduces on current HEAD; boot now progresses to services.exe.

---

## Methodology

- **Byte-diff waves** (strings, sid, acl, sd, sec, atom): the `rtl-difftest`
  crate compiles the *genuine* `.c` (own ref TU in `build.rs`, with a SAL
  neutralization block + `scrape_protos()` for call-before-definition) and
  fuzzes Rust output byte-for-byte against it. `core_rs` is `forbid(unsafe)`,
  Kani-friendly. ~86 cases today.
- **Translate-only waves** (priv, tz, env, res, thr, prc, path): pure syscall
  plumbing (chains of `Zw*`) — wrong shape for byte-diff; smoke is the only
  net. Single block in `cabi/mod.rs`, comment in `CMakeLists.txt`, ship.
- **Heap is its own track** (behavioral / property / Kani) — never byte-diff.

---

## Open issues / debt

The rtl flip is complete and green on mainline; remaining items are quality/cleanup,
not blockers.

- ~~**amd64 services.exe init hang**~~ — **RESOLVED** (2026-06-01 point-in-time state;
  cleared on mainline — see Boot state above).
- ~~**i386 + arm64 not re-validated** with the 3 PATH-wave fixes~~ — **RESOLVED:**
  all three arches boot green to `shell ready`/desktop on mainline.
- **amd64 SD-wave msgqueue.split2.c:87 spam** (pre-PATH) — diff-coverage gap,
  needs `--timeout 600`; extend `mk_relative_sd`/`mk_absolute_sd`. (Cosmetic/coverage,
  not a boot blocker.)
- ~~`registry.c`, `image.c`, `heap.c` family remain C~~ — **RESOLVED:**
  all flipped to Rust (registry → `rtl-misc`, image → `rtl-diag`, heap family
  → `rtl-heap`/`rtl-heappage`/`rtl-heapuser`). Only the irreducible
  variadic-glue + per-arch asm floor is C now (see top marker).
- **Irreducible C floor (by design, not debt):** `dbgprint_variadic.c` (180 LOC,
  C-variadic trampolines) + `rtl_vista_stub.c` (17 LOC) + ~2.35K LOC per-arch asm.
  These back the Rust crates over an FFI boundary and are not slated for translation.
- **Quality / Stage-3 idiomation** of the landed crates (Result+Drop-RAII) is the
  remaining frontier, tracked with the broader translation effort — not rtl-specific.

---

## Conventions (must-follow)

- **Worktree path discipline** — when reopening rtl work in a dedicated worktree,
  prefix every edit with that worktree's absolute toplevel; bare repo-root paths
  leak into the shared main checkout. (The original `rtl-priv-port` lane is closed.)
- **Never revert a `.rs` to C** to pass a build/smoke; **never delete a
  translated `.rs`** without explicit approval; a broken-but-integrated
  translation visible in smoke is the correct state (see AGENTS.md).
- **Never weaken smoke markers.** Boot bar is `shell ready` (x86).
- **Stage commits explicitly by path; never `git add -A`.** Always-excluded
  pre-existing mods: `AGENTS.md`, `boot/bootdata/livecd.ini`,
  `drivers/input/i8042prt/*.c`, `ntoskrnl-rust/.../pnpmgr/devaction.rs`,
  `arm64input-rust/`, `win32k-(kernel-)?rust`, `win32k-sys`, the
  `sdk/lib/rust/Cargo.toml` workspace edit.
- **`core_rs` is `forbid(unsafe_code)`** — inline asm + raw FFI live in
  `cabi/mod.rs` only.

---

## Cross-references

- Tracker memory: `project_rtl_strings_track`, `project_rtl_path_wave_wip`.
- Methodology: `cnt_rust_prep` (diff-harness), `smoke-tests`.
- Sister audits: `static-libs-translation-audit.md` (rtl algo crates),
  `ntdll-translation-audit.md` (ntdll DLL + libsupp boundary),
  `ntoskrnl-translation-audit.md`.
- Superseded handoff: `handoff-rtl-libs-next-steps.md` (codex session handoff;
  its "upstream PnP blocker" framing is corrected above).
