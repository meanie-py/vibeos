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

---
Found while transpiling ReactOS as a translation anchor. Reproduced against c2rust 0.22.1 from crates.io on x86_64 Linux, target `x86_64-w64-mingw32`.
