# cargo_cell_gen: behaviour the Rust port must carry

The Rust port now carries items 1–12 below, including wave 4 from `f1d2c6ff0a6`.
The compatibility tests run against the native implementation. Conversion also refuses
missing recipes for a newly built cell’s own C libraries (`86ea9f4e40e`); native manifest
refresh preserves the media staged-directory guard (`1f2e9344078`). Deliberate stage removal
requires `tools/build-infra media-manifest --write --allow-drop`.

lint-rs `93d5def2dfe` ports `tools/cargo_cell_gen.py` to `vibeos-codegen` from master
`613f7f991d1`. The body-less (C-only) cell waves changed the Python after that base, so a port
that starts from `613f7f991d1` silently drops the items below. Each is the scar of a real
failure during the waves (2026-11-01), and each has a test in `tools/test_cargo_cell_gen.py`.
The test names are the porting checklist: port the behaviour, then port or keep the test.

## In ctail-wave2 (`d6412702fb0`) and ctail-wave3 (`e3b46afc3d1`), queued for landing

1. **i386 body-less `.eh_frame`** (`I386_BODYLESS_EH_FRAME_MAX = 0x400`, in `check()`). A C-only
   cell whose C cmake compiled without unwind tables gains a small `.eh_frame`: rsbegin/rsend's
   frame markers and the shell's panic handler (0x88 for kernelbase and dxapi). Accepted only on
   i386, only for a body-less cell, only when cmake's PE had no `.eh_frame`, only up to the bound,
   and reported as `(+ the shell's i386 .eh_fram)`. `_cell_is_bodyless()` decides "body-less"
   from the build.rs marker. Test: `test_a_bodyless_cell_is_recognised_by_its_build_script`.
2. **i386 body-less `.reloc`** (`I386_BODYLESS_RELOC_MAX = 0x80`, the same rule). When cmake's PE
   had no relocations, that machinery is the first thing needing any: 0x48 bytes for kdstub and
   dxgthk, every entry in .ctors/.dtors/.eh_frame, rsbegin's .data and its registration code.
   Same conditions, same note.
3. **Empty link orders.** A cell whose every arch has an empty link order (kdstub links no
   library) emits neither the `link_order` table nor the `.link_order(link_order)` call. Both
   emitters test `any(cell.link_order.values())`, not the dict's truthiness.
4. **`--convert` keeps an existing baseline.** When `--baseline DIR` already holds every arch and a
   `targets.json` naming each target, it is reused, not re-copied: after a failed build,
   `build-<arch>` may hold cell-linked PEs, and a fresh copy compares the cells with themselves.
   A target missing from the index is an error. Test: `test_a_rerun_keeps_the_first_runs_baseline`.
5. **A failed build names its failure.** `_ninja` reports the `FAILED:` edges and `error` lines,
   then the tail, not the last 15 lines alone, which in a parallel cargo build are
   "Blocking waiting for file lock" noise. Test:
   `test_a_failed_build_names_the_failed_edge_not_the_lock_noise`.

## In ctail-wave4, being verified now

6. **Dry-run every rewrite before writing.** `convert()` runs `rewrite_cmake` for every cell
   before `apply()` writes any: a rewrite refusal (dbghelp's link in an else branch) at the 60th
   target had left 59 half-converted modules. Test: `test_a_rewrite_refusal_writes_no_cell`.
7. **Refuse a target another CMakeLists reads by generator expression.** If any other CMakeLists
   names the target in `$<TARGET_FILE…:target>` (rshell and browseui copy themselves into
   `$<TARGET_FILE_DIR:filebrowser>`), refuse it in `model()`: a cell is a custom target, which
   those cannot resolve, and the whole tree's configure fails. Test:
   `test_a_target_another_cmakelists_reads_by_genex_is_found`.
8. **Moved statements keep their `if()`s.** `rewrite_cmake` used to move every TRANSFERRED
   statement on the target (compile options, definitions, include dirs) onto the OBJECT library
   unconditionally: libxslt's MSVC-only `/wd4267` reached clang. Each moved statement is now
   re-emitted inside the same `if(...)` chain it sat in. One under an `elseif`/`else` branch is
   refused, not reconstructed. An audit of all 345 earlier conversions found 7 such hoists, all
   harmless (always-true Clang conditions, one warning flag, imm32's default-ON option). Test:
   `test_a_conditional_compile_setting_keeps_its_condition`.
9. **Objects map to sources with their subdirectory.** `_obj_source()` reads the path under
   `CMakeFiles/<target>.dir/`, and `_shim_name()` writes shims relative to the module directory.
   By basename alone, crypt32's `reactos/unixlib.c.obj` resolved to Wine's `unixlib.c` one level
   up, and `reactos/store.c` would have compiled Wine's `store.c` without a word. Test:
   `test_an_object_in_a_subdirectory_maps_to_that_subdirectorys_source`.
10. **`.rc` dependencies behind list variables.** `_rc_object_depends` results are expanded through
    the file's own `set()`/`list(APPEND)` lists (`_list_vars`, `_expand`) before searching them for
    generated inputs, in both `rewrite_cmake` and `_drop_res_in`. msxml3's `.rc` waited on
    `${msxml3_rc_deps}`, which holds `msxml3_v1.tlb`, so its cell could compile the .rc before widl
    made the typelib. Test: `test_a_generated_rc_input_behind_a_list_variable_orders_the_cell`.

## From ctail-bodyless (wave 1, on master since batch 65): confirm the port has it

11. **Body-less cells.** `model()` accepts a target with no Rust body and links only its C, unless
    a custom `cargo build` command or a `lib<target>*_rust.a` on its link shows Rust that a
    body-less cell would silently drop (desk, jscript). Test:
    `test_a_c_module_with_no_rust_body_is_a_cell_with_no_rust_archive`.
12. **The CRT anchor.** In a body-less cell, the `"body"` link-order item sits where cmake linked
    `libmsvcrtex_*.a`, so the engine emits the Rust CRT there. Without it a C DLL has no
    `_DllMainCRTStartup@12`.
