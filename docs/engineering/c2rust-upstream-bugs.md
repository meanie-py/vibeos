# c2rust upstream bugs — reproducers, root causes, and what they cost us

Two ICEs in **c2rust 0.22.1** block `rustconv translate --c2rust` (the compiler-anchored
translation path, `tools/rustconv/rustconv/c2rust_anchor.py`). Both are reproduced here from
scratch, root-caused in c2rust's own source, and minimized to standalone C that needs no
ReactOS headers. §1 additionally has a **verified one-line fix**.

Status: **not yet filed upstream** (github.com/immunant/c2rust/issues). The two sections below
are paste-ready. Neither is a duplicate as far as issue search shows; the closest existing
issue is [#231](https://github.com/immunant/c2rust/issues/231) (closed, `CTypeId`/`CStmtId` on
FANN) which shares §2's *downstream symptom* but not its cause.

---

## 1. `knr_function_type_with_parameters` does not unwrap `Attributed` → taking the address of a `__stdcall` function ICEs

```
thread 'main' panicked at c2rust-transpile-0.22.1/src/convert_type.rs:518:25:
ctype parameter must be a function instead of Attributed(CQualTypeId { qualifiers: Qualifiers {
is_const: false, is_restrict: false, is_volatile: false }, ctype: CTypeId(1207) }, None)
```

### Reproducer (3 lines, no headers)

```c
/* repro.c */
int __attribute__((stdcall)) f(int a) { return a; }
typedef int (__attribute__((stdcall)) *PFN)(int);
PFN get(void) { return f; }
```

```json
[{"directory":"<dir>",
  "command":"clang --target=x86_64-w64-mingw32 -c repro.c -o repro.o",
  "file":"<dir>/repro.c"}]
```

Dropping *either* the `stdcall` attribute **or** the `return f;` (so the address is never
taken) makes it transpile cleanly.

### Root cause

`convert_expr` casts a function to `fn()` when its address is needed
(`translator/mod.rs:3510` — *"If we are referring to a function and need its address, we need to
cast it to fn() to ensure that it has a real address"*), calling
`knr_function_type_with_parameters` to test for K&R-ness.

That function (`convert_type.rs:477-519`) unwraps the transparent wrappers `Elaborated`,
`Decayed`, `Paren`, `TypeOf`, `Typedef` by recursing, then `panic!`s on anything else.
**`Attributed` is missing**, and an AttributedType is exactly what clang builds for a
calling-convention attribute. The same file already treats it as transparent elsewhere:

```rust
// convert_type.rs:430
CTypeKind::Attributed(ty, _) => self.convert(ctxt, ty.ctype),
```

so this reads as an omission, not a deliberate restriction. Still present on `master`.

### Fix (verified)

```rust
CTypeKind::Attributed(ty, _) => {
    self.knr_function_type_with_parameters(ctxt, ty.ctype, params)
}
```

Applied to 0.22.1 and rebuilt: the reproducer transpiles, and the real file that surfaced this
(`dll/win32/oleaut32/varformat.c`, 2618 lines) transpiles to 295 KB of Rust with no other
change in behaviour. `enumprot.c` still hits §2 under the same patched binary, confirming the
two bugs are independent.

### Why it matters

`__stdcall` is spelled `WINAPI`/`CALLBACK`/`APIENTRY` throughout the Win32 headers, and taking
such a function's address is what every callback registration does:

```c
static LRESULT CALLBACK WndProc(HWND, UINT, WPARAM, LPARAM) { ... }
wc.lpfnWndProc = WndProc;   /* address taken -> ICE */
```

`noreturn`, `warn_unused_result` and `ms_hook_prologue` on the same function do **not** trigger
it — only attributes clang models as an `AttributedType` wrapping the function type.

---

## 2. `__label__` (GCC local label declaration) is unsupported, and c2rust panics instead of stopping on its own "AST was invalid" diagnostic

```
warning: Missing child 103700211514640 of node AstNode { tag: TagDeclStmt, children: [Some(..)],
         loc: SrcSpan { fileid: 1, begin_line: 3, .. }, type_id: None, .. }
Exported Clang AST was invalid. Check warnings above for unimplemented features.

thread 'main' panicked at c2rust-transpile-0.22.1/src/c_ast/mod.rs:314:30:
Could not find CDeclId(33364) in TypedAstContext
```

### Reproducer (9 lines, no headers)

```c
/* repro.c */
int f(int x) {
    {
        __label__ done;      /* GNU C local label declaration */
        if (x) goto done;
        x = 1;
        done:;
    }
    return x;
}
```

### Characterization

| case | result |
|---|---|
| `__label__` declared and used | ICE, 1 × `Missing child` |
| `__label__` declared, never used | ICE, 1 × `Missing child` |
| `__label__ a, b;` (two in one decl) | ICE, 2 × `Missing child` |
| `__label__` inside a statement-expression | ICE, 1 × `Missing child` |
| ordinary label + `goto`, no `__label__` | **clean** |

So the AST exporter emits no child for the `LabelDecl` inside a `DeclStmt`; one warning per
declared local label. Ordinary labels are unaffected.

### Two separate defects

1. **Feature gap:** `__label__` is not exported. (A GNU extension, but c2rust targets GNU C.)
2. **Robustness — the more annoying half:** c2rust *already knows* the AST is invalid and says
   so with a file:line. It then keeps going and dies with `Could not find CDeclId(33364) in
   TypedAstContext`, whose backtrace lands in `CommentLocator::pre` → `get_src_loc` — a place
   with no relationship to the actual problem. **The diagnostic it printed is actionable; the
   panic it exits with is not.** Bailing out after "Exported Clang AST was invalid" (or making
   `get_src_loc` return `None` for a missing decl, as `Index<CExprId>` already does with its
   `BADEXPR` fallback at `c_ast/mod.rs:1360`) would turn an ICE into a clean error.

---

## What this costs *us*

**Every `_SEH2_TRY` file is un-anchorable — 260 `.c` files.** Under `_USE_DUMMY_PSEH=1` (our
default on all three arches) `sdk/lib/pseh/include/pseh/pseh2.h:117` expands `_SEH2_TRY` to a
block opening with `__label__ __pseh_try_end;`, so §2 fires on every one of them. This is not a
translation-quality issue — it is a hard "no c2rust anchor available", and `--c2rust` correctly
HARD FAILS (`C2RustError`) rather than silently degrading to an unanchored LLM translation.

`_SEH2_TRY` is also why the ws2_32 lane's `enumprot.c` / `sockctrl.c` could not be anchored.

**Options, none taken yet:**
* Vendor the §1 patch (verified) and carry it; §2 needs real exporter work upstream.
* Preprocess `_SEH2_*` away before anchoring (changes the code c2rust sees — the anchor would
  no longer correspond to the source we are translating; probably not worth it).
* File both upstream and wait. §1 is a one-liner a maintainer can take immediately.

## Reproducing from this repo

The cohab'd `.c` files are commented out of CMake, so they have no `compile_commands.json`
entry and the pipeline path cannot reach them. Steal flags from a compiled sibling in the same
directory, repoint `file`/`-o` at the target, and run `c2rust transpile` under
`tools/canon-run.sh` (the `/tmp/vibeos` mount must exist for the `-I` paths to resolve).
Note c2rust only *parses* the compile DB for flags — it uses its own bundled libclang, so the
compiler path in `command` is never executed.

## Filing these (NOT done — needs a human to approve an outward-facing post)

`gh` is authenticated as `meanie-py` and `immunant/c2rust` has issues enabled, so these are one
command each. Bodies are split out verbatim under `docs/engineering/c2rust-issues/`:

```bash
gh issue create --repo immunant/c2rust \
  --title "ICE: knr_function_type_with_parameters does not unwrap CTypeKind::Attributed (taking the address of a __stdcall fn)" \
  --body-file docs/engineering/c2rust-issues/1-attributed-knr-ice.md

gh issue create --repo immunant/c2rust \
  --title "ICE: __label__ (GCC local label) unsupported — panics in the comment locator instead of stopping on its own \"AST was invalid\" diagnostic" \
  --body-file docs/engineering/c2rust-issues/2-label-decl-ice.md
```

§1 is worth filing with the patch attached — it is a one-line fix, verified locally (patched
0.22.1, rebuilt, both the 3-line reproducer and the real 2618-line `varformat.c` transpile).
§2 is two defects in one report; a maintainer may want it split (the missing `__label__` support
vs. panicking after already printing an actionable diagnostic).
