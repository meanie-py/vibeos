# JScript decoder

`dll/win32/jscript/decode.c` is translated by c2rust from its real amd64 compile
command, with no model call or algorithm rewrite. `rustconv.anchor_lower` removes
the obsolete `raw_ref_op` feature and maps C integer spellings using compiler
widths; address-sized typedefs remain `usize`/`isize`. The original C copyright
and LGPL notice are retained. This crate supplies only `decode_source`; the DLL
still builds its other translation units from C.

All three target compilers report 32-bit `long`, 16-bit `wchar_t`, and signed
8-bit `char`; pointer width is 32 bits on i386 and 64 bits on amd64/ARM64.
The exported function uses C calling convention, including `_decode_source`
on i386. The actual DLL links Wine diagnostics and the CRT's `wcsncmp`.

The offline cohort compiles the genuine C and this shipping Rust module with
host-only UTF-16/diagnostic shims. A root-run corpus (seed 230923, 1,000 random
cases per family) compared 3,267 inputs, return statuses, and complete in-place
buffers with zero mismatches or buffer-guard failures. Table-phase and checksum
mutants were both detected. This is bounded behavior coverage, not proof of
memory safety or a guest execution test of the decoder.

Original compiler output is preserved in the response bank at
`tools/rustconv/.responses/compiler-jscript-decode/`. Regenerate neither the C
translation nor a model answer to test tooling changes: replay the saved Rust.
The benchmark records source, candidate, harness and tool fingerprints.
