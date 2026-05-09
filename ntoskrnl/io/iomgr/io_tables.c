/*
 * io_tables.c — IO operation length/access tables consumed by the Rust
 * iofunc.rs translation.
 *
 * `iofunc.c` (which used to host these tables via `#include
 * "internal/io_i.h"`) was replaced by Rust. The Rust side declares
 * `extern "C" { static IopQueryOperationLength: u8; ... }` for all 8
 * tables and accesses them via raw pointer arithmetic, so the symbols
 * still need a C-side definition.
 *
 * Previously they lived inside `io_rust_shims.c` as a side effect of
 * including `internal/io_i.h`. Extracted to this dedicated file so the
 * shim can stay header-free; this file deliberately keeps the PCH
 * `<ntoskrnl.h>` inject + the io_i.h include because it's nothing but
 * the table definitions.
 */

#include <ntoskrnl.h>
#include "internal/io_i.h"
