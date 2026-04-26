---
name: volatile-static-linker-issue
description: Volatile<T> on #[no_mangle] statics causes boot hang — likely read-only section placement
type: feedback
---

`Volatile<T>` wrapper on `#[no_mangle] pub static` globals causes boot hang on i386. Tested 2026-03-22.

**Why:** Changing `static mut X: KIRQL = 0` to `static X: Volatile<KIRQL>` moves the symbol from `.data` (writable) to potentially `.rdata` (read-only). Even though `UnsafeCell` provides interior mutability, the Rust compiler may place `#[no_mangle] static` (non-mut) symbols in read-only sections. Writing to a read-only section causes a page fault → boot hang.

**How to apply:** Don't use `Volatile<T>` for `#[no_mangle] pub static` globals that are written to. Instead use `read_volatile`/`write_volatile` on the raw `static mut` pointer. `Volatile<T>` is safe for internal (non-exported) statics, or for statics that are only read.

**Alternative:** Could use `static mut X: Volatile<KIRQL>` but that combines `mut` (compiler knows it's writable) with volatile (forces memory access). Ugly but correct. Or keep `static mut KIRQL` and just wrap accesses in `read_volatile`/`write_volatile`.
