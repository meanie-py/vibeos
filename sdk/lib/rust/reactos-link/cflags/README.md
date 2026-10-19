# Committed compile rule templates

**`rc.<arch>.flags`**, **`c.<arch>.flags`** and **`asm.<arch>.flags`** are the shared part of
cmake's windres, C and ASM rules. `Linker::rc` / `Linker::c_compiled` and the shared C
libraries' recipes (`../clibs/<arch>/<target>.recipe`) complete them with each compile's own
includes, flags and defines, for every such compile, cmake build or not
(docs/engineering/cargo-drive-plan.md, rungs 4 and 6). Regenerate with
`python3 tools/cargo_cell_gen.py --rc-template` / `--c-template`.

A `c_from_compile_db(<target>, sources)` library replays cmake's compile_commands.json in a
cmake build; with none, it compiles its sources' lines of `<target>`'s recipe
(`tools/cargo_cell_gen.py --clib-recipes`). The per-library flag files that used to stand
in for it (`blrtl`, `libcntpr`, `win32k_residue`) are retired.

**`genincdata.<arch>.flags`** is the one per-target template left: the flags
`generate_ks_inc` compiles `genincdata.c` with, to self-host geninc's `ks*.inc`. It is
cmake's recorded command for that target with the PCH and the `build-<arch>` `-I`s stripped
and the repo root as `{REPO}`. Regenerate it from a configured build with
`python3 tools/gen_cflags.py`; `vibeos-cli check`'s drift lint compares it.
