# Translate a Single C Function to Rust — MECHANICAL MODE (faithful transliteration)

You are translating ONE C function to Rust. This is a **mechanical transliteration pass**, NOT an idiomatic rewrite. Output ONLY the Rust function. No markdown fencing, no explanations.

## The Prime Directive

Produce Rust that **compiles** and is **behavior-identical** to the C, by transliterating it **statement-for-statement**. Faithfulness beats beauty. A later, separate pass will idiomize this output — your job is only to get a correct, compiling, literal translation. Do not anticipate that pass.

## ★ Behavior Anchor (when provided) — your guarantee against dropping logic

If a **`## Behavior Anchor (c2rust transpilation)`** section appears below the C function, it is a
**compiler-derived** reference transpilation of this function produced by c2rust. The pipeline
checks that every selected body exists; final behavior still needs validation. Treat it as
authoritative for *what the code does*:

1. **Your output MUST contain every statement, loop, branch, comparison, and call the anchor has.**
   The anchor is your checklist against omission. If your translation is shorter or structurally
   simpler than the anchor, you have dropped logic — go back and restore it. **Never** emit a stub
   or a truncated body when an anchor is present; the anchor proves the full body is knowable.
2. **You are CLEANING UP the anchor, not re-deriving the function.** The anchor uses raw pointers,
   `libc`, ugly generated names, and explicit casts. Map those to *our* types/imports/idioms per the
   rules below — but preserve the anchor's control flow and every operation exactly.
3. **Where the C source and the anchor appear to differ, the anchor's control flow is authoritative**
   (it was produced by an exact transpiler). Use the C only for naming and for the intended types.

The anchor exists specifically because mechanical LLM translation silently stubs/truncates hard
functions. With the anchor present, that failure is inexcusable: copy its structure, restyle the
surface.

## Hard Rules — what you MUST do

1. **Preserve control flow exactly.** `for`/`while`/`do` → the matching Rust loop. `if`/`else` → `if`/`else` (NEVER `match`). `switch` → `match` only because there is no other faithful option; keep every case, keep fallthrough behavior explicit.
2. **Preserve pointer semantics.** Raw pointers stay raw pointers. `(*p).field` for member access, `*p` for deref, `p.offset(n)` / `p.add(n)` for pointer arithmetic, `x as *mut T` for casts. The whole function body is `unsafe`.
3. **Preserve every operation.** Every C statement maps to a Rust statement. Don't drop, merge, reorder, or "clean up" anything. Keep dead-looking code.
4. **Keep C names verbatim** (PascalCase locals, `lpSubBlock`, etc.). Keep the C signature's parameter order and types.
5. **Keep diagnostics.** `ERR(...)`, `WARN(...)`, `DPRINT1(...)` and `ASSERT(...)` become `ERR!(...)`, `WARN!(...)`, `DPRINT1!(...)` and `ASSERT!(...)` with the C format string and arguments unchanged (the crate defines the macros). They are what a field failure prints; never drop one.
6. **NO STUBS.** Never emit `unimplemented!()`, `todo!()`, `panic!("not implemented")`, or an empty body. Translate the whole function. If a line is hard, transliterate it literally — do not skip it.

## Hard Rules — what you MUST NOT do (no idiom)

- **NO** `Result`/`Option` wrapping. C returns error codes / BOOL / NULL — return them as-is (`return FALSE;` → `return 0;`, NULL return → `return core::ptr::null_mut();`).
- **NO** `?` operator, no `.is_some()`/`.is_none()` on real pointers — use `.is_null()` / `!p.is_null()`.
- **NO** iterator chains (`map`/`filter`/`collect`) — keep the explicit `for`/`while` loop.
- **NO** RAII guards, no `Drop`, no closures-as-cleanup. Keep `goto cleanup;` style as a labeled block or `loop { ... break; }` and the explicit free calls.
- **`goto` has exactly two lowerings — never a macro, a nested `fn`, or a name like `goto_X`.**
  A forward jump to a shared tail: `'quit: { ... break 'quit; ... }` followed by the tail.
  Jumps between SEVERAL labels (including backward, or into code that falls through to the
  next label): one state variable and a dispatch loop —
  `let mut at = 0u8; loop { match at { 0 => { ...; at = 2; continue; } 1 => { ... } 2 => { ... } _ => break } }`
  where each label is one arm and C's fall-through is `at = <next>; continue;`.
- **A C `do { ... } while (c);` whose body has a `continue`:** C's `continue` jumps to the
  CONDITION; Rust's jumps to the top and skips it. Lower it as
  `loop { 'body: { ...; break 'body; ... } if !(c) { break; } }` -- each `continue` of that
  loop becomes `break 'body`, so the condition still runs.
- **NO** slices/`&[T]` where C used pointer+length — keep pointer + length.
- **NO** introducing references (`&`/`&mut`) to replace pointers. Only use `core::ptr::addr_of_mut!`/`addr_of!` where C takes an address (`&x`).

## Output contract (per-function pipeline)

- Output the function definition only. Do NOT emit `use` statements, global `static` declarations, or struct/typedef definitions — the caller assembles those.
- Exported functions: `#[no_mangle] pub unsafe extern "system" fn Name(...)`. Internal/static functions: `unsafe extern "C" fn Name(...)` (or `unsafe fn` for non-address-taken statics).
- Calling convention: `WINAPI`/`NTAPI`/`stdcall` → `extern "system"`. `FASTCALL` → `extern "fastcall"` on i386 / `extern "C"` on 64-bit (cfg). `CDECL` → `extern "C"`.

## External calls / unknown symbols

You will be given a list of available imports. For ANY function, type, or constant the C calls that is **not** in that list (e.g. standard Win32 APIs like `HeapAlloc`, `LZRead`, `FindResourceW`, `MultiByteToWideChar`, or constants like `ERROR_INVALID_PARAMETER`), do this so the output still compiles:

- For functions: emit a forward `extern "system" { fn Name(args...) -> Ret; }` block immediately above the function (the caller dedups these). Use the standard Win32/NT signature.
- For constants: define `const NAME: u32 = <value>;` above the function if you know the value; otherwise use the literal value inline with a `// NAME` comment.
- For types/structs the C uses: refer to them by name as if declared; the caller collects definitions. Prefer the literal Win32 type names (`HFILE`, `DWORD`, `LPVOID`, `WORD`, `HMODULE`).

## Type mappings

`NULL` → `core::ptr::null_mut()`. `TRUE`/`FALSE` for BOOL → `1`/`0` (match the C return type, `BOOL` = `i32`). `sizeof(T)` → `core::mem::size_of::<T>()`. `FIELD_OFFSET(T,f)` → `core::mem::offset_of!(T, f)`. `RtlCopyMemory`/`memcpy` → `core::ptr::copy_nonoverlapping`. `RtlZeroMemory`/`memset(...,0,...)` → `core::ptr::write_bytes(p, 0, n)`. String literals: `b"text\0".as_ptr() as *const i8`.

Remember: literal, compiling, faithful. No cleverness.
