"""Stitch API output into Rust crate modules."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

from .batch_runner import TranslationResult
from .config import OUTPUT_CRATE, TARGETS, Target, hal_cmake_for_c
from .crate_integrate import integrate
from .function_coverage import require_rust_definitions
from .postprocess import postprocess
from .signature_fidelity import repair_signatures
from .table_fidelity import repair_tables
from .typedefs import pointer_typedefs_for_source

if TYPE_CHECKING:
    from .scanner import ScanResult
    from .spec_parser import SpecEntry

# First line of an assembled file, naming WHICH slice of the TU produced it. One C++ TU often
# needs several runs -- `--free-functions` for its free helpers, `--cpp-com --class X` per class
# -- and every one of them derives the same output path from the TU's stem. Without this marker
# the second run silently overwrites the first, which looks like a successful assembly and is
# only noticed later when the symbols it wrote are missing.
ASSEMBLY_MARKER = "// rustconv-assembly: "


class AssemblyClobberError(RuntimeError):
    """Raised when a run would overwrite an assembly produced by a DIFFERENT slice of the TU."""


def _is_committed_and_clean(out_path: Path) -> bool:
    """Is this file tracked by git with no uncommitted changes?

    The discriminator between INTEGRATED code and an in-flight grind. During initial
    translation a TU gets re-assembled repeatedly and the destination is untracked or already
    dirty; once it is committed and clean it is shipping code that the tree has since
    repaired. Anything git cannot answer (no repo, no git) returns False, which keeps the
    permissive path — this guard exists to stop a specific known loss, not to gate on tooling.
    """
    try:
        tracked = subprocess.run(
            ["git", "ls-files", "--error-unmatch", "--", str(out_path)],
            cwd=out_path.parent,
            capture_output=True,
            check=False,
        )
        if tracked.returncode != 0:
            return False
        dirty = subprocess.run(
            ["git", "status", "--porcelain", "--", str(out_path)],
            cwd=out_path.parent,
            capture_output=True,
            text=True,
            check=False,
        )
        return dirty.returncode == 0 and not dirty.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return False


def _guard_assembly_target(out_path: Path, selector: str, reassemble: bool = False) -> None:
    """Refuse to overwrite an assembly that a DIFFERENT slice of the TU produced.

    A partial slice (`free-functions`, `class:X`) covers only part of the TU, so writing it
    over someone else's output drops symbols. `whole-tu` covers everything, so it cannot drop
    a symbol — but that is not the same as being safe to overwrite, which is what this guard
    used to assume.

    A whole-TU re-run over COMMITTED code discards every repair made after integration, and
    those repairs are exactly the ones the C cannot express, so the model re-emits the
    original bug every time. Measured on browser-rust: one re-roll re-forked `extern "system"`
    back into `#[cfg(target_arch = "x86")] extern "stdcall"` ABI twins, put `&ServiceStatus`
    back on a `static mut` where the fix was `addr_of_mut!`, and renamed `MIDL_user_allocate`
    to the C's lowercase source spelling — `rpcndr.h` #defines it to the capitalised symbol
    the generated marshaller actually binds to, and the comment recording that was deleted
    along with it. Nothing about that output looks wrong in isolation.
    """
    if not selector or not out_path.exists():
        return
    if selector == "whole-tu" and not reassemble and _is_committed_and_clean(out_path):
        raise AssemblyClobberError(
            f"{out_path} is committed, unmodified, INTEGRATED code — refusing to re-assemble "
            f"over it.\nA re-translation sees only the C, so it silently reverts every repair "
            f"made after integration (ABI-twin collapse, symbol capitalisation, "
            f"`addr_of_mut!` on statics, diagnostics kept as macros).\n"
            f"The new output is durable in the run's .responses/ directory either way — diff "
            f"it in rather than overwriting.\n"
            f"fix: pass `-o <path>` to write elsewhere, or `--reassemble` if you really do "
            f"want the file replaced wholesale."
        )
    first = out_path.read_text(errors="replace").split("\n", 1)[0]

    if not first.startswith(ASSEMBLY_MARKER):
        # No marker: written before this guard existed, or hand-authored. We cannot tell WHAT
        # is in there, which makes it the dangerous case, not the safe one -- an unmarked
        # dialogs/drvdefext.rs held a whole translated CDrvDefExt class and a --free-functions
        # run replaced all 1,742 lines of it. A whole-TU run still may overwrite: it is not a
        # partial slice, so it cannot drop anything the TU should have.
        if selector == "whole-tu":
            return
        raise AssemblyClobberError(
            f"{out_path} already exists and carries no assembly marker, so this run cannot "
            f"tell whether `{selector}` covers everything in it.\n"
            f"Overwriting would DISCARD anything it covers that this slice does not.\n"
            f"fix: pass `-o <path>` to give this run its own file (and check what is in the "
            f"existing one first)."
        )

    prior = first[len(ASSEMBLY_MARKER) :].strip()
    if prior == selector:
        return
    raise AssemblyClobberError(
        f"{out_path} was assembled from `{prior}`, but this run is `{selector}`.\n"
        f"Both derive their path from the same TU, so writing would DISCARD the other slice.\n"
        f"fix: pass `-o <path>` to give this run its own file."
    )


def _c_source_for_slice(
    c_source: str,
    scan: ScanResult | None,
    assembled_fns: set[str] | None,
    selector: str,
) -> tuple[str, int]:
    """Embed only the C this run actually translated, when the run is a PARTIAL slice.

    The trailing C comment block is not decoration -- several lints use it as their oracle,
    pairing a commented-C function against the live Rust of the same name in the same file.
    Embedding the WHOLE TU under a `--free-functions` or `--cpp-com --class X` run therefore
    ships C whose Rust lives in a *sibling* module, and every one of those reads as a DROP:
    `folders/CNetFolder.cpp`'s `rgfInOut &= ~SFGAO_VALIDATE` was reported dropped by
    scan_mask_drift while sitting live and correct in `net_folder.rs:864`.

    A whole-TU run embeds everything, unchanged -- there its C and its Rust cover the same set.
    """
    if selector in ("", "whole-tu") or scan is None or not assembled_fns:
        return c_source, 0
    keep: set[int] = set()
    for f in scan.functions:
        # A free function's worker is its own name; a C++ method's is `<Class>_<method>`.
        names = {f.name}
        if f.cpp_class:
            names.add(f"{f.cpp_class}_{f.name}")
        if names & assembled_fns:
            keep.update(range(f.line_start, f.line_end + 1))
    if not keep:
        return c_source, 0
    # Everything above the first translated function is file scope (includes, macros, statics)
    # and belongs to every slice.
    first_fn = min(f.line_start for f in scan.functions) if scan.functions else 1
    lines = c_source.splitlines()
    out, elided = [], 0
    for i, line in enumerate(lines, 1):
        if i < first_fn or i in keep:
            out.append(line)
        else:
            elided += 1
    return "\n".join(out), elided


def assemble_file(
    result: TranslationResult,
    subsystem: str,
    c_source: str,
    output_dir: Path = OUTPUT_CRATE,
    exports: dict[str, SpecEntry] | None = None,
    scan: ScanResult | None = None,
    target: Target | None = None,
    c_source_path: Path | None = None,
    selector: str = "",
    assembled_fns: set[str] | None = None,
    reassemble: bool = False,
    expected_definitions: set[str] | None = None,
) -> Path:
    """Write a single translation result into the crate structure.

    For ntoskrnl: output_dir/src/{subsystem}/{filename}.rs
    For hal:      target.output_crate/src/x86/{filename}.rs
                  + appends `pub mod {filename};` to src/x86/mod.rs
                  + appends to CMakeLists.txt DEPENDS list
                  + comments out the C file in the matching subsystem .cmake

    Applies post-processing and appends original C source as comments.

    Args:
        result: Translation result with Rust code
        subsystem: Subsystem name (e.g., "ex", "ob", "mm") — used for ntoskrnl
        c_source: Original C source code to embed as comments
        output_dir: Root of the output crate (overridden by target if set)
        exports: Spec-exported functions for #[no_mangle] fixup
        scan: Scanner result for global #[no_mangle] fixup
        target: Target descriptor; if set, overrides output_dir + subdir
        c_source_path: Original C file path (for HAL CMake editing)

    Returns:
        Path to the written file
    """
    if target is None:
        target = TARGETS["ntoskrnl"]

    # Derive module name from source filename
    stem = Path(result.source_file).stem  # e.g., "uuid" from "uuid.c"

    if target.src_subdir is not None:
        # Fixed flat/subdir routing (hal → src/x86; kdcom/bootvid → src).
        sub_dir = target.output_crate / target.src_subdir
    else:
        sub_dir = (
            (target.output_crate if target.name != "ntoskrnl" else output_dir) / "src" / subsystem
        )

    sub_dir.mkdir(parents=True, exist_ok=True)

    out_path = sub_dir / f"{stem}.rs"
    _guard_assembly_target(out_path, selector, reassemble=reassemble)

    # Post-process the Rust code
    rust_code = postprocess(
        result.rust_code,
        exports=exports,
        globals=scan.globals if scan else None,
        target_name=target.name,
        sys_crate=target.sys_crate,
        sys_umbrella=target.sys_umbrella,
        c_source_path=c_source_path,
        # Match the `translate` path (__main__.py): the --assemble route was silently
        # skipping BOTH of these, so a COM target assembled here got no stdmethod! wrapper
        # and a crate-island target got its `types` symbols collapsed into the sys umbrella.
        com_method_macro=target.com_method_macro,
        concrete_stdcall_slots=target.concrete_stdcall_slots,
        crate_module=getattr(target, "crate_module", "") if target.crate_bindgen_files else "",
    )

    # Static data tables come from the C verbatim, not from the model's transcription of
    # it. This is pure substrate closure applied to data: mbcs.c's tables came back with a
    # wrong LENGTH on every run (the values were right each time), and a padded table that
    # still compiles is silently wrong forever. See table_fidelity's docstring.
    rust_code, table_findings = repair_tables(rust_code, c_source)
    for finding in table_findings:
        print(f"  [tables] {finding}", file=sys.stderr)

    # Same argument one level up, applied to signatures: a typedef that is already a pointer
    # must not be given a second star. The model added one to 73 of mbcs.c's 77 locale-taking
    # parameters, and that error is invisible at the ABI (both spellings are one machine
    # pointer) -- it only surfaces if some dereference in the body happens to be loud. See
    # signature_fidelity's docstring for why the rule cannot fire on a benign re-spelling.
    if scan is not None and c_source_path is not None:
        rust_code, sig_findings = repair_signatures(
            rust_code, scan, pointer_typedefs_for_source(c_source_path)
        )
        for finding in sig_findings:
            print(f"  [signatures] {finding}", file=sys.stderr)

    # Land the file on the target crate's shared prelude (import-surface swap + drop the
    # extern decls the prelude already provides — see crate_integrate). No-op for a crate
    # without a src/prelude.rs (ntoskrnl, hal, …), so it's safe on every target.
    rust_code, report = integrate(rust_code, target.output_crate, module=stem)
    if report.changed or report.kept_unprovided:
        print(f"  integrate: {report.summary()}")
        if report.kept_unprovided:
            # Each of these is either a real gap in the prelude (add it THERE, with the
            # right ABI) or a symbol the model invented. Keeping it means an
            # undefined-at-link surprise, so it must never pass silently.
            print(
                "  integrate: UNPROVIDED externs kept as-is (verify each is a real link "
                f"symbol with the right ABI): {', '.join(sorted(set(report.kept_unprovided)))}"
            )

    # Build the file content with C source as trailing comments
    c_source, elided = _c_source_for_slice(c_source, scan, assembled_fns, selector)
    if elided:
        print(
            f"  [c-source] embedded only this slice's C; elided {elided} line(s) belonging to "
            f"functions another module owns (keeps the C-vs-Rust lints' oracle honest)"
        )
    content_parts = []
    if selector:
        content_parts.append(f"{ASSEMBLY_MARKER}{selector}")
    content_parts.append(rust_code.rstrip())
    content_parts.append("")
    content_parts.append("")
    content_parts.append("// " + "=" * 76)
    content_parts.append("// ORIGINAL C SOURCE")
    content_parts.append("// " + "=" * 76)
    content_parts.append("//")
    for line in c_source.splitlines():
        if line:
            content_parts.append(f"// {line}")
        else:
            content_parts.append("//")
    content_parts.append("//")
    content_parts.append("// " + "=" * 76)
    content_parts.append("// END ORIGINAL C SOURCE")
    content_parts.append("// " + "=" * 76)
    content_parts.append("")

    # Strip per-line trailing whitespace. A model that wraps a call argument list leaves a
    # space after the comma, which `git diff --check` flags on every commit of generated
    # output — and stripping it by hand does not stick, because the next regeneration (now
    # a $0 cache replay, so it happens often) puts it straight back. The generator owns its
    # own whitespace. Only trailing blanks go; indentation and blank lines are untouched.
    text = "\n".join(line.rstrip() for line in "\n".join(content_parts).split("\n"))
    if expected_definitions is not None:
        require_rust_definitions(expected_definitions, text)
    out_path.write_text(text)

    # HAL-specific output routing: append to mod.rs, CMakeLists.txt DEPENDS,
    # and comment out the C file in the matching *.cmake.
    if target.name == "hal":
        _hal_register_mod(target, stem)
        _hal_register_cmake_deps(target, stem)
        if c_source_path is not None:
            _hal_comment_out_c_in_cmake(c_source_path, stem)

    return out_path


def _hal_register_mod(target: Target, stem: str) -> None:
    """Append `pub mod {stem};` to hal-rust/src/x86/mod.rs if absent."""
    assert target.src_subdir is not None  # always set for the hal target (caller invariant)
    mod_path = target.output_crate / target.src_subdir / "mod.rs"
    if not mod_path.exists():
        return  # don't create — manual review wanted
    text = mod_path.read_text()
    if re.search(rf"^pub mod {re.escape(stem)};", text, re.MULTILINE):
        return  # already registered
    if re.search(rf"^#\[cfg\(.*\)\]\s*\npub mod {re.escape(stem)};", text, re.MULTILINE):
        return  # cfg-gated variant present
    # Insert before the trailing blank line, in alphabetical position.
    lines = text.splitlines()
    insert_line = f"pub mod {stem};"
    # Find run of `pub mod X;` lines and insert in alphabetical order.
    mod_lines = [
        (i, line_text)
        for i, line_text in enumerate(lines)
        if re.match(r"pub mod \w+;", line_text.strip())
    ]
    inserted = False
    for i, line_text in mod_lines:
        existing_name = line_text.strip().split()[2].rstrip(";")
        if stem < existing_name:
            lines.insert(i, insert_line)
            inserted = True
            break
    if not inserted:
        # Append after last `pub mod` line
        if mod_lines:
            lines.insert(mod_lines[-1][0] + 1, insert_line)
        else:
            lines.append(insert_line)
    mod_path.write_text("\n".join(lines) + ("\n" if not text.endswith("\n") else ""))


def _hal_register_cmake_deps(target: Target, stem: str) -> None:
    """Append the new .rs file to the DEPENDS list in hal/halx86/CMakeLists.txt.

    Pattern in CMakeLists.txt:
        DEPENDS ${HAL_RUST_WORKSPACE_DIR}/hal-rust/Cargo.toml
                ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/lib.rs
                ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/dma.rs
                ...
    """
    if target.cmake_deps_file is None or not target.cmake_deps_file.exists():
        return
    text = target.cmake_deps_file.read_text()
    new_line = f"            ${{HAL_RUST_WORKSPACE_DIR}}/hal-rust/src/x86/{stem}.rs"
    if new_line in text:
        return  # already present
    # Find the existing block of `${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/*.rs` lines
    pattern = re.compile(
        r"(^            \$\{HAL_RUST_WORKSPACE_DIR\}/hal-rust/src/x86/\w+\.rs\n)+",
        re.MULTILINE,
    )
    m = pattern.search(text)
    if not m:
        return  # block not found — bail rather than corrupt CMakeLists
    # Collect existing entries inside the block, insert new one alphabetically.
    block_text = m.group(0)
    entry_pat = re.compile(
        r"^            \$\{HAL_RUST_WORKSPACE_DIR\}/hal-rust/src/x86/(\w+)\.rs$", re.MULTILINE
    )
    existing = [(em.group(1), em.group(0)) for em in entry_pat.finditer(block_text)]
    if any(name == stem for name, _ in existing):
        return
    existing.append((stem, new_line))
    existing.sort(key=lambda p: p[0])
    new_block = "\n".join(line for _, line in existing) + "\n"
    new_text = text[: m.start()] + new_block + text[m.end() :]
    target.cmake_deps_file.write_text(new_text)


def _hal_comment_out_c_in_cmake(c_source_path: Path, stem: str) -> None:
    """Comment out the .c entry in the matching subsystem .cmake.

    Pattern looks like:
        generic/dma.c
        → # generic/dma.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/dma.rs)
    """
    cmake_path = hal_cmake_for_c(c_source_path)
    if cmake_path is None or not cmake_path.exists():
        return
    text = cmake_path.read_text()
    # subsystem dir name relative to halx86/ (e.g. "generic" from hal/halx86/generic/dma.c)
    subsys = c_source_path.parent.name
    c_rel = f"{subsys}/{stem}.c"
    replacement_comment = f"# {c_rel} — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/{stem}.rs)"
    # If already commented out by us, leave alone.
    if replacement_comment in text:
        return
    # Replace exact `    generic/dma.c)` or `    generic/dma.c` (with trailing whitespace/EOL).
    # Be conservative: match `^(\s*)c_rel(\)?)\s*$`.
    pat = re.compile(
        rf"^(\s*){re.escape(c_rel)}(\)?)\s*$",
        re.MULTILINE,
    )
    if not pat.search(text):
        return

    def _sub(m):
        indent = m.group(1)
        paren = m.group(2)
        return f"{indent}{replacement_comment}{paren}"

    text = pat.sub(_sub, text, count=1)
    cmake_path.write_text(text)


def generate_mod_rs(
    subsystem: str, module_names: list[str], output_dir: Path = OUTPUT_CRATE
) -> Path:
    """Generate or update mod.rs for a subsystem, merging with existing modules."""
    sub_dir = output_dir / "src" / subsystem
    sub_dir.mkdir(parents=True, exist_ok=True)

    mod_path = sub_dir / "mod.rs"

    # Parse existing modules from mod.rs if it exists
    existing_modules = set()
    if mod_path.exists():
        for line in mod_path.read_text().splitlines():
            m = re.match(r"pub mod (\w+);", line)
            if m:
                existing_modules.add(m.group(1))

    # Merge new module names with existing
    all_modules = sorted(existing_modules | set(module_names))

    lines = [f"//! {subsystem} subsystem — translated from C.", ""]
    for name in all_modules:
        lines.append(f"pub mod {name};")
    lines.append("")

    mod_path.write_text("\n".join(lines))
    return mod_path


def generate_lib_rs(subsystems: list[str], output_dir: Path = OUTPUT_CRATE) -> Path:
    """Update the top-level lib.rs, merging with existing subsystem modules.

    If lib.rs already exists, parses existing `pub mod X;` lines and merges
    with the new subsystems list. Preserves any extra attributes/allows.
    """
    src_dir = output_dir / "src"
    src_dir.mkdir(parents=True, exist_ok=True)

    lib_path = src_dir / "lib.rs"

    # If lib.rs exists, parse existing subsystem modules and preserve content
    if lib_path.exists():
        existing_text = lib_path.read_text()
        existing_subs = set()
        for line in existing_text.splitlines():
            m = re.match(r"pub mod (\w+);", line)
            if m:
                existing_subs.add(m.group(1))

        all_subs = sorted(existing_subs | set(subsystems))

        # Replace just the pub mod lines, keep everything else
        # Remove old pub mod lines, then append merged list
        new_lines = []
        for line in existing_text.splitlines():
            if re.match(r"pub mod \w+;", line):
                continue
            new_lines.append(line)

        # Remove trailing blank lines
        while new_lines and new_lines[-1] == "":
            new_lines.pop()
        new_lines.append("")

        for sub in all_subs:
            new_lines.append(f"pub mod {sub};")
        new_lines.append("")

        lib_path.write_text("\n".join(new_lines))
    else:
        lines = [
            "//! ntoskrnl-rust: Rust translation of ReactOS ntoskrnl.",
            "//!",
            "//! Auto-generated by rustconv. Do not edit manually.",
            "",
            "#![no_std]",
            "#![allow(non_snake_case)]",
            "#![allow(non_camel_case_types)]",
            "#![allow(non_upper_case_globals)]",
            "#![allow(unused_unsafe)]",
            "#![allow(dead_code)]",
            "",
        ]
        for sub in sorted(subsystems):
            lines.append(f"pub mod {sub};")
        lines.append("")

        lib_path.write_text("\n".join(lines))

    return lib_path


def generate_cargo_toml(output_dir: Path = OUTPUT_CRATE) -> Path:
    """Generate Cargo.toml for the output crate."""
    cargo_path = output_dir / "Cargo.toml"
    content = """\
[package]
name = "ntoskrnl-rust"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib"]

[dependencies]
ntkrnl-sys = { path = "../ntkrnl-sys" }

[features]
default = []
"""
    cargo_path.write_text(content)
    return cargo_path
