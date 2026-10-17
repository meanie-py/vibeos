"""Provision the host linter and demultiplex its single shared invocation."""

from __future__ import annotations

import fcntl
import hashlib
import os
import subprocess
from collections.abc import Callable
from pathlib import Path

from tools.source_walk import source_files

# Only parity-proven plugins belong here. The original registry owns stage ordering.
PORTED = frozenset(
    {
        "verify_match_consts",
        "verify_tables",
        "scan_export_shadows",
        "verify_ffi",
        "verify_lost_exports",
        "lint_kernel",
        "cohab_census",
        "scan_boot_media_drift",
        "null_offset_to_offset_of",
        "scan_magic_offsets",
        "scan_silent_failures",
        "strip_msvc_cmake",
        "shell32_tail_inventory",
        "scan_invented_constants",
        "verify_c_signatures",
        "verify_conv_macros",
        "verify_translation_literals",
        "verify_constants",
        "verify_constants_enums",
        "scan_call_literal_vs_c",
        "scan_cfg_vs_c",
        "scan_com_double_release",
        "scan_const_contradictions",
        "scan_const_vs_island",
        "scan_const_vs_own_island",
        "scan_const_within_crate",
        "scan_duplicate_symbols",
        "scan_extern_decl_drift",
        "scan_irql_balance",
        "scan_mask_drift",
        "scan_ob_refcount",
        "scan_override_self_dispatch",
        "scan_ptr_typedef_indirection",
        "scan_unaligned_wide",
        "scan_unpruned_walk",
        "scan_varargs_shim_parity",
        "scan_version_vs_c",
        "scan_arg_side_effects",
        "scan_assert_polarity",
        "scan_case_bitor_pattern",
        "scan_octal_literals",
        "scan_raw_user_access",
        "scan_safe_probe_deref",
        "scan_doc_sections",
        "scan_orphan_modules",
        "scan_pack_drift",
        "scan_bool_width",
        "scan_bugcheck_stubs",
        "scan_cc_mismatch",
        "scan_dropped_asserts",
        "scan_dropped_prints",
        "scan_dropped_probes",
        "scan_dropped_seh_pairs",
        "scan_extern_static_width",
        "scan_import_cc",
        "scan_macro_fn_shadow",
        "scan_nightly_surface",
        "scan_rawdylib_linkname",
        "scan_res_named_types",
        "scan_type_alias_contradictions",
        "scan_varargs_spec_arity",
        "scan_wide_literals",
        "scan_winerror_consts",
        "scan_inline_flag_literals",
        "scan_comment_swallowed_code",
        "scan_formatless_printf",
        "scan_ptr_typedef_cast_read",
        "scan_call_arg_swap",
        "scan_sse_helpers",
        "scan_extern_arity",
        "scan_dropped_null_guard",
        "scan_ptr_byte_diff",
        "scan_constant_placement",
        "scan_raising_probes",
        "scan_store_on_failure",
        "scan_assert_to_ub",
        "scan_bindgen_padding_index",
        "scan_bitwise_not_bool",
        "scan_bugcheck_codes",
        "scan_byte_step_one",
        "scan_byteptr_stride",
        "scan_calloc_elem_type",
        "scan_cdecl_alias",
        "scan_containing_record_null",
        "scan_dangling_transmute",
        "scan_fourcc_literals",
        "scan_gmem_flags",
        "scan_match_arm_break",
        "scan_narrow_vs_wide_literal",
        "scan_noop_goto",
        "scan_opaque_ptr_arith",
        "scan_ptr_highword_mask",
        "scan_raw_bitfield",
        "scan_raw_uart_writes",
        "scan_typed_byte_copy",
        "scan_wide_byte_copy",
        "scan_list_walk_from_head",
        "scan_dowhile_continue",
        "scan_variadic_array_arg",
        "scan_let_shadow",
        "scan_double_fetch",
        "scan_hresult_literals",
        "scan_init_then_move",
        "scan_read_after_clear",
        "scan_const_narrowing_cast",
        "scan_info_class_literals",
        "scan_misaligned_cast",
        "scan_fnptr_abi_transmute",
    }
)
SENTINEL = "VIBEOS_LINT"


def ensure(repo: Path) -> Path:
    source = repo / "tools/vibeos-lint"
    target = repo / ".cache/vibeos-lint/vibeos-lint"
    target.parent.mkdir(parents=True, exist_ok=True)

    def fingerprint() -> str:
        files = [
            source / "Cargo.toml",
            source / "Cargo.lock",
            repo / "rust-toolchain.toml",
            *sorted(source_files(source / "src")),
        ]
        digest = hashlib.sha256()
        for path in files:
            digest.update(str(path.relative_to(repo)).encode() + b"\0" + path.read_bytes() + b"\0")
        return digest.hexdigest()

    stamp = target.with_suffix(".sha256")
    with target.with_suffix(".lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        key = fingerprint()
        if target.is_file() and stamp.is_file() and stamp.read_text() == key:
            return target
        env = {
            k: v
            for k, v in os.environ.items()
            if k
            not in {
                "RUSTUP_TOOLCHAIN",
                "RUSTC_WRAPPER",
                "CARGO_BUILD_RUSTC_WRAPPER",
                "CARGO_BUILD_TARGET",
                "RUSTFLAGS",
                "CARGO_ENCODED_RUSTFLAGS",
                "RUSTC_BOOTSTRAP",
            }
        }
        build = target.parent / "target"
        cmd = [
            "cargo",
            "build",
            "--offline",
            "--locked",
            "--release",
            "--manifest-path",
            str(source / "Cargo.toml"),
            "--target-dir",
            str(build),
        ]
        cp = subprocess.run(cmd, cwd=repo, env=env, capture_output=True, text=True)
        artifact = build / "release/vibeos-lint"
        if cp.returncode or not artifact.is_file():
            raise RuntimeError(f"vibeos-lint bootstrap failed: {cp.stderr}")
        if fingerprint() != key:
            raise RuntimeError(
                "vibeos-lint sources changed during bootstrap; rerun after edits finish"
            )
        temporary = target.with_suffix(f".{os.getpid()}.tmp")
        try:
            temporary.write_bytes(artifact.read_bytes())
            temporary.chmod(0o755)
            temporary.replace(target)
            stamp.write_text(key)
        finally:
            temporary.unlink(missing_ok=True)
    return target


def split_output(
    cp: subprocess.CompletedProcess, names: list[str]
) -> dict[str, subprocess.CompletedProcess]:
    """A shared failure invalidates all requested plugins, even with partial output."""
    lines: dict[str, list[str]] = {name: [] for name in names}
    error = ""
    if cp.returncode or any(
        line.startswith(("FATAL:", "Traceback (most recent call last):"))
        for line in cp.stderr.splitlines()
    ):
        error = cp.stderr or f"shared scanner exited {cp.returncode}"
    else:
        for line in cp.stdout.splitlines():
            name, tab, finding = line.partition("\t")
            if not tab or name not in lines or not finding:
                error = f"invalid tagged output: {line!r}"
                break
            lines[name].append(finding)
    return {
        name: subprocess.CompletedProcess(
            [SENTINEL, name],
            2 if error else 0,
            "" if error else "\n".join(findings),
            f"FATAL: vibeos-lint: {error}" if error else "",
        )
        for name, findings in lines.items()
    }


def run_many(
    repo: Path, names: list[str], invoke: Callable[[list[str]], subprocess.CompletedProcess]
) -> dict[str, subprocess.CompletedProcess]:
    try:
        runner = ensure(repo)
        cp = invoke([str(runner), "--root", str(repo), "--rules", ",".join(names)])
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        cp = subprocess.CompletedProcess([], 2, "", f"FATAL: {error}")
    return split_output(cp, names)
