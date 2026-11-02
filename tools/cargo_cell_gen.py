#!/usr/bin/env python3
"""Compatibility facade for the native cargo-cell generator.

DTOs and original Python call signatures are retained for importers. Parsing,
modelling, rendering, and filesystem queries are performed by the native tool.
"""

from __future__ import annotations

import dataclasses
import inspect
import json
import os
import re
import subprocess
import sys
import time
import types
import typing
from dataclasses import dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RUST = REPO / "sdk/lib/rust"
ARCHES = ("i386", "amd64", "arm64")


class GenError(Exception):
    pass


class NoLinkEdge(GenError):
    def __init__(self, target, arch, why, message=None):
        self.target, self.arch, self.why = target, arch, why
        super().__init__(
            message
            if message is not None
            else f"{target}: {_metadata['_NO_EDGE_TEXT'][why].format(arch=arch)}"
        )


@dataclass
class Link:
    arch: str
    output: str  # build-dir relative
    kind: str  # C_EXECUTABLE / C_MODULE_LIBRARY / C_SHARED_LIBRARY
    entry: str
    subsystem: str | None
    image_base: int | None
    stack: int | None
    inputs: list[str]
    libs: list[str]
    # `--require-defined=SYM` operands, in link order (ole32's target_link_options).
    require_defined: tuple[str, ...] = ()


@dataclass
class Cell:
    target: str
    cmake_dir: Path
    output_name: str
    module_kind: str  # Win32Exe | Win32Dll
    entries: dict[str, str]
    subsystem: str | None
    image_base: dict[str, int | None]
    stack: int | None
    body_crate: str
    body_features: list[str]
    body_defaults: bool
    import_libs: list[tuple[str, str, str]]  # (spec_rel, dll_name, basename)
    msvcrtex: str | None
    static_libs: dict[str, list[str]]  # per arch, build-dir relative .a, cmake-built for now
    spec: tuple[str, str] | None  # (spec_rel, dll_name) for exports
    spec_stubs: bool
    c_shims: list[str]
    rc: str | None  # rc source, repo relative
    rc_defines: list[str]
    destinations: list[str]  # CD directories, as the CMakeLists spells them
    name_on_cd: str | None
    # Per arch: did cmake's link carry the body's panic handler? Only then does the cell
    # take panic_common (and with it all of core); otherwise panic-lean.
    panics: dict[str, bool] = field(default_factory=lambda: dict.fromkeys(ARCHES, True))
    # The rust_staticlib_variant that builds the body; "" means `<target>_rust`.
    body_var: str = ""
    # Per arch, the static libs cmake linked AFTER the body (they lose a duplicate
    # definition to it); the rest of `static_libs` precede it.
    late_libs: dict[str, list[str]] = field(default_factory=dict)
    # add_delay_importlibs: same shape as import_libs, linked through gen-delaylib.py.
    delay_import_libs: list[tuple[str, str, str]] = field(default_factory=list)
    # The c_shims cmake generates into the build dir (widl's RPC stubs, flex output).
    generated: list[str] = field(default_factory=list)
    # The c_shims from OUTSIDE the module's directory (cscript compiles ../wscript's).
    foreign: list[str] = field(default_factory=list)
    # (basename, delayed) in cmake's link order, when there are delay imports: an
    # `__imp_` symbol two of the libs define binds to the first (Linker::delay_import_lib).
    import_order: list[tuple[str, bool]] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)
    # Per arch, the RUSTFLAGS cmake built the body with (`ENV "RUSTFLAGS=..."` on its
    # rust_staticlib_variant): calc and friends pass floats by value, so i386 needs SSE.
    body_rustflags: dict[str, str] = field(default_factory=dict)
    # That `set(<VAR>) if(ARCH ...) set(<VAR> ENV ...) endif()` block, which the rewrite
    # drops with the variant.
    body_env_block: str = ""
    # Per arch, when the arches ship to different CD dirs (winprint's prtprocs/<arch>).
    arch_destinations: dict[str, list[str]] = field(default_factory=dict)
    # Per arch, cmake's order of "body", "import:<base>" and static libs; empty for
    # cells generated before Linker::link_order (they keep static_libs/late_libs).
    link_order: dict[str, list[str]] = field(default_factory=dict)
    # add_cd_file(... NO_CAB FOR all): not in reactos.cab, so setup does not install it.
    no_cab: bool = False
    # KernelDll: set_module_type's type (kernelmodedriver, kerneldll), which is
    # add_cargo_module's KIND and so its post-link.
    kernel_kind: str = ""
    # KernelDll: the driver's own body custom command (_custom_body), which the rewrite drops.
    custom_body: dict = field(default_factory=dict)

    @property
    def crate(self) -> str:
        """The cell's cargo package. A package name cannot start with a digit, so a
        target that does (3dtext) takes its body crate's stem (text3d-cargo)."""
        if self.target[:1].isdigit():
            return self.body_crate.removesuffix("-rust") + "-cargo"
        return f"{self.target}-cargo"

    # Per arch, the C/asm compiled into the module on that arch only (on top of c_shims).
    arch_shims: dict[str, list[str]] = field(default_factory=dict)
    # Per arch, when the arches link different subsystem versions (welcome: 4.00 on i386).
    subsystems: dict[str, str] = field(default_factory=dict)
    # add_cd_file argument lists (TARGET dropped) for the boot-CD-only placements.
    extra_cd: list[str] = field(default_factory=list)
    # Per arch, the module's part of cmake's windres command (Linker::rc).
    rc_flags: dict = field(default_factory=dict)
    # Per arch, `-Wl,--require-defined=SYM` operands (target_link_options): symbols the
    # link must define up front, so the body's definition wins over an import thunk.
    require_defined: dict[str, list[str]] = field(default_factory=dict)


@dataclass
class RcParts:
    windres: str
    includes: list[str]  # absolute, canonical
    flags: list[str]  # <FLAGS>: CMAKE_RC_FLAGS and whatever the target added
    defs: list[str]  # RC_INVOKED .. the I18N_DEFS
    defines: list[str]  # <DEFINES>, sorted
    src: str


@dataclass
class RcTemplate:
    windres: str
    std: list[str]  # encoded
    flags: list[str]
    defs: list[str]
    defines: list[str]

    def text(self, arch: str) -> str:
        return _rpc("rc_template_text", {"template": self, "arch": arch}, REPO)


@dataclass(frozen=True)
class RcFlags:
    """A module's part of its windres command, as Linker::rc takes it."""

    includes: tuple[str, ...]
    flags: tuple[str, ...]
    defines: tuple[str, ...]


@dataclass
class CParts:
    cc: str
    target: str
    defines: list[str]
    includes: list[str]
    flags: list[str]
    obj: str
    src: str


@dataclass
class CTemplate:
    cc: str
    target: str
    std: list[str]  # encoded include dirs
    core: list[str]  # encoded flags
    defines: list[str]

    def text(self, arch: str) -> str:
        return _rpc("c_template_text", {"template": self, "arch": arch}, REPO)


@dataclass(frozen=True)
class CFlags:
    """A C library's part of cmake's C rule, as Linker::c_compiled takes it."""

    includes: tuple[str, ...]
    flags: tuple[str, ...]
    defines: tuple[str, ...]


def _jsonable(value) -> typing.Any:
    if isinstance(value, NoLinkEdge):
        return {"target": value.target, "arch": value.arch, "why": value.why, "message": str(value)}
    if dataclasses.is_dataclass(value):
        return {f.name: _jsonable(getattr(value, f.name)) for f in dataclasses.fields(value)}
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, bytes):
        return list(value)
    if isinstance(value, dict):
        return {str(k): _jsonable(v) for k, v in value.items()}
    if isinstance(value, (tuple, list, set, frozenset)):
        return [_jsonable(v) for v in value]
    return value


def _rpc(op, kwargs, root) -> typing.Any:
    tool = REPO / "tools/codegen"
    command = [str(tool), "--root", str(root), "cargo-cell", "--api-json"]
    request = json.dumps({"op": op, "args": [], "kwargs": _jsonable(kwargs)}, ensure_ascii=False)
    try:
        result = subprocess.run(command, input=request, text=True, capture_output=True, check=False)
    except OSError as exc:
        raise GenError(f"cannot run native cargo-cell tool {tool}: {exc}") from exc
    try:
        response = json.loads(result.stdout)
    except (ValueError, TypeError) as exc:
        raise GenError(
            f"native cargo-cell {op} returned invalid JSON (exit {result.returncode}): "
            f"{result.stderr.strip()}"
        ) from exc
    if not isinstance(response, dict):
        raise GenError(f"native cargo-cell {op} returned a non-object response")
    if "error" in response:
        error = response["error"]
        if not isinstance(error, dict):
            raise GenError(f"native cargo-cell {op} returned a malformed error")
        msg = error.get("message", "native cargo-cell failed")
        if error.get("kind") == "NoLinkEdge":
            raise NoLinkEdge(error.get("target"), error.get("arch"), error.get("why"), msg)
        raise GenError(msg)
    if result.returncode or "ok" not in response:
        raise GenError(
            f"native cargo-cell {op} failed (exit {result.returncode}): {result.stderr.strip()}"
        )
    return response["ok"]


def _coerce(value, annotation) -> typing.Any:
    if value is None or annotation is inspect.Signature.empty or annotation is typing.Any:
        return value
    origin = typing.get_origin(annotation)
    params = typing.get_args(annotation)
    if origin in (typing.Union, types.UnionType):
        if type(None) in params and value is None:
            return None
        alternatives = [t for t in params if t is not type(None)]
        if len(alternatives) == 1:
            return _coerce(value, alternatives[0])
        for t in alternatives:
            try:
                return _coerce(value, t)
            except (TypeError, ValueError, KeyError):
                continue
        return value
    if annotation is Path:
        return Path(value)
    if isinstance(annotation, type) and dataclasses.is_dataclass(annotation):
        hints = typing.get_type_hints(annotation)
        result: typing.Any = annotation(
            **{k: _coerce(v, hints.get(k, typing.Any)) for k, v in value.items()}
        )
        if annotation is Cell:
            result.rc_flags = {k: _coerce(v, RcFlags) for k, v in result.rc_flags.items()}
            if isinstance(result.custom_body.get("vars"), list):
                result.custom_body["vars"] = set(result.custom_body["vars"])
        return result
    if origin in (list, set, frozenset):
        entries = [_coerce(v, params[0]) for v in value] if params else value
        return origin(entries)
    if origin is tuple:
        if not params:
            return tuple(value)
        if len(params) == 2 and params[1] is Ellipsis:
            return tuple(_coerce(v, params[0]) for v in value)
        return tuple(_coerce(v, t) for v, t in zip(value, params, strict=True))
    if origin is dict:
        return (
            {_coerce(k, params[0]): _coerce(v, params[1]) for k, v in value.items()}
            if params
            else value
        )
    return value


def _invoke(op, kwargs) -> typing.Any:
    repo = kwargs.get("repo", REPO)
    root = Path(repo) if repo is not None else REPO
    value = _rpc(op, kwargs, root)
    annotation = typing.get_type_hints(globals()[op]).get("return", typing.Any)
    if op == "check":
        if value.get("report"):
            print(value["report"], end="" if value["report"].endswith("\n") else "\n")
        return value["code"]
    result = _coerce(value, annotation)
    if op == "reshell":
        for line in value:
            print(line)
        return None
    if op == "cmake_facts" and result:
        if result.get("body") is not None:
            result["body"] = tuple(result["body"])
        if result.get("set_entrypoint") is not None:
            result["set_entrypoint"] = tuple(result["set_entrypoint"])
        if isinstance(result.get("custom_body", {}).get("vars"), list):
            result["custom_body"]["vars"] = set(result["custom_body"]["vars"])
    if op == "_custom_body" and result and isinstance(result.get("vars"), list):
        result["vars"] = set(result["vars"])
    if op == "_rc_flags_by_analogy":
        result = {k: _coerce(v, RcFlags) for k, v in result.items()}
    if op == "c_lib_for":
        result = (result[0], {k: _coerce(v, CFlags) for k, v in result[1].items()})
    return result


def _no_cache():
    pass


ENGINE_LIBS: dict[str, str]
REBOUND_EXPORTS: dict[str, set[str]]

_metadata = _rpc("constants", {}, REPO)
if not isinstance(_metadata, dict):
    raise GenError("native cargo-cell constants response must be an object")
for _name, _value in _metadata.items():
    if _name not in ("REPO", "RUST", "ARCHES"):
        globals()[_name] = _value
for _name in (
    "PANIC_COMMON_IMPORTS",
    "STACK_PROBE_IMPORTS",
    "NAME_PROPERTIES",
    "SHARED_C_LIBS",
    "_NOT_STATIC",
):
    if _name in globals():
        globals()[_name] = set(globals()[_name])
for _name in ("REPLACED", "TRANSFERRED", "KERNEL_KINDS", "KEPT"):
    if _name in globals():
        globals()[_name] = tuple(globals()[_name])
for _name in ("DEAD_IMPORTS", "REBOUND_EXPORTS"):
    if _name in globals():
        globals()[_name] = {k: set(v) for k, v in globals()[_name].items()}
if "I386_RSBEGIN_SECTIONS" in globals():
    I386_RSBEGIN_SECTIONS = set(_metadata["I386_RSBEGIN_SECTIONS"])
for _name in ("_SPEC_IMPORT_ENTRY", "_SPEC_SELF_TARGET", "RES_CALL", "SHIMS_CALL"):
    if _name in globals():
        globals()[_name] = re.compile(
            globals()[_name], re.M if _name in ("_SPEC_IMPORT_ENTRY", "_SPEC_SELF_TARGET") else 0
        )


def _spec_imported_exports(spec: Path) -> set[str]:
    return _invoke("_spec_imported_exports", locals())


def _read(path: Path) -> str:
    return _invoke("_read", locals())


def _ccdb(path: Path) -> list:
    return _invoke("_ccdb", locals())


def _no_edge_why(ninja: str, target: str) -> str:
    return _invoke("_no_edge_why", locals())


def _link_edges(target: str, repo: Path) -> dict[str, Link]:
    return _invoke("_link_edges", locals())


def _why_by_arch(miss: dict) -> str:
    return _invoke("_why_by_arch", locals())


def link_edge(arch: str, target: str, repo: Path = REPO) -> Link:
    return _invoke("link_edge", locals())


def spec_index(repo: Path = REPO) -> dict[str, tuple[str, str] | None]:
    return _invoke("spec_index", locals())


def cd_dirs(arch: str, output: str, repo: Path = REPO) -> set[str]:
    return _invoke("cd_dirs", locals())


def _one_list(arch: str, stem: str, repo: Path) -> Path | None:
    return _invoke("_one_list", locals())


def txtsetup_files(repo: Path = REPO) -> frozenset[str]:
    return _invoke("txtsetup_files", locals())


def cd_placement(arch: str, output: str, repo: Path = REPO) -> list[str]:
    return _invoke("cd_placement", locals())


def _rc_command(arch: str, rc_obj: str, repo: Path = REPO) -> tuple[str, list[str]]:
    return _invoke("_rc_command", locals())


def rc_template_path(arch: str, repo: Path = REPO) -> Path:
    return _invoke("rc_template_path", locals())


def _rc_parts(cmd: str) -> RcParts:
    return _invoke("_rc_parts", locals())


def _rc_enc(path: str, arch: str) -> str:
    return _invoke("_rc_enc", locals())


def _rc_dec(tok: str, arch: str) -> str:
    return _invoke("_rc_dec", locals())


def _rc_commands(arch: str, repo: Path = REPO) -> list[dict]:
    return _invoke("_rc_commands", locals())


def derive_rc_template(arch: str, repo: Path = REPO) -> RcTemplate:
    return _invoke("derive_rc_template", locals())


def read_rc_template(arch: str, repo: Path = REPO) -> RcTemplate:
    return _invoke("read_rc_template", locals())


def rc_decode(f: RcFlags, tmpl: RcTemplate, arch: str, src: str) -> list[str]:
    return _invoke("rc_decode", locals())


def rc_encode(cmd: str, tmpl: RcTemplate, arch: str) -> RcFlags:
    return _invoke("rc_encode", locals())


def rc_flags_for(rc_rel: str, marker: str, repo: Path = REPO) -> dict[str, RcFlags]:
    return _invoke("rc_flags_for", locals())


def _statements(text: str) -> list[tuple[int, int, str, str]]:
    return _invoke("_statements", locals())


def _args(s: str) -> list[str]:
    return _invoke("_args", locals())


def _branches(stmts: list[tuple[int, int, str, str]]) -> list[tuple[tuple[int, int], ...]]:
    return _invoke("_branches", locals())


def _exclusive(p: tuple[tuple[int, int], ...], q: tuple[tuple[int, int], ...]) -> bool:
    return _invoke("_exclusive", locals())


def _included_functions(cmake: Path, text: str) -> set[str]:
    return _invoke("_included_functions", locals())


def cmake_facts(cmake: Path, target: str, libs: list[str] | None = None) -> dict:
    return _invoke("cmake_facts", locals())


def _arch_values(text: str, var: str) -> dict[str, str]:
    return _invoke("_arch_values", locals())


def _custom_body(text: str) -> dict | None:
    return _invoke("_custom_body", locals())


def _source_list(text: str) -> list[str]:
    return _invoke("_source_list", locals())


def _inlinable_source(text: str, cmake_dir: Path) -> list[str] | None:
    return _invoke("_inlinable_source", locals())


def _cd_for(a: list[str]) -> list[str]:
    return _invoke("_cd_for", locals())


def _drop_target(a: list[str], target: str) -> list[str]:
    return _invoke("_drop_target", locals())


def _body_env(text: str, extras: list[str]) -> tuple[dict[str, str], str] | None:
    return _invoke("_body_env", locals())


def _ssp_objects(a: list[str]) -> bool:
    return _invoke("_ssp_objects", locals())


def _subsystem_only(a: list[str]) -> bool:
    return _invoke("_subsystem_only", locals())


def _require_defined_only(a: list[str]) -> bool:
    return _invoke("_require_defined_only", locals())


def _subsystem_flag(sub: str) -> str:
    return _invoke("_subsystem_flag", locals())


def _entry_symbol(arch: str, sym: str, stack: str | None) -> str:
    return _invoke("_entry_symbol", locals())


def _names_only(a: list[str]) -> bool:
    return _invoke("_names_only", locals())


def _retarget(cmd: str, raw: str, new: str) -> str:
    return _invoke("_retarget", locals())


def rewrite_cmake(cmake: Path, cell: Cell) -> str:
    return _invoke("rewrite_cmake", locals())


def _enclosing_ifs(text: str, target: str) -> list[str]:
    return _invoke("_enclosing_ifs", locals())


def _library_index(repo: Path = REPO) -> dict[str, tuple[str, list[str], list[str]]]:
    return _invoke("_library_index", locals())


def engine_lib(lib: str, repo: Path = REPO) -> tuple[str, list[str]] | None:
    return _invoke("engine_lib", locals())


def _engine_deps(name: str) -> list[str]:
    return _invoke("_engine_deps", locals())


def _placeholder(path: Path) -> bool:
    return _invoke("_placeholder", locals())


def _links_delayimp(static: list[str]) -> bool:
    return _invoke("_links_delayimp", locals())


def _placeholder_archive(arch: str, lib: str, repo: Path) -> bool:
    return _invoke("_placeholder_archive", locals())


def source_dir(arch: str, output: str, target: str, repo: Path = REPO) -> Path:
    return _invoke("source_dir", locals())


def _check_statement_effects(target: str, facts: dict, links: dict[str, Link]) -> None:
    return _invoke("_check_statement_effects", locals())


def model(target: str, repo: Path = REPO) -> Cell:
    return _invoke("model", locals())


def _rs_list(items: tuple[str, ...] | list[str], indent: int) -> str:
    return _invoke("_rs_list", locals())


def rc_binding(flags: dict) -> list[str]:
    return _invoke("rc_binding", locals())


def rc_call(rc: str) -> str:
    return _invoke("rc_call", locals())


def build_rs(cell: Cell) -> str:
    return _invoke("build_rs", locals())


def _export_is_import_thunk(code: bytes, machine: int) -> bool:
    return _invoke("_export_is_import_thunk", locals())


def _exports_bound_to_imports(pe_path: Path) -> set[str]:
    return _invoke("_exports_bound_to_imports", locals())


def carries_panic_handler(pe_path: Path) -> bool:
    return _invoke("carries_panic_handler", locals())


def _cfg(arches: list[str]) -> str:
    return _invoke("_cfg", locals())


def cargo_toml(cell: Cell) -> str:
    return _invoke("cargo_toml", locals())


def _panic_deps(cell: Cell) -> str:
    return _invoke("_panic_deps", locals())


def _user_panic_deps(cell: Cell) -> str:
    return _invoke("_user_panic_deps", locals())


def main_rs(cell: Cell) -> str:
    return _invoke("main_rs", locals())


def apply(cell: Cell, repo: Path = REPO) -> list[Path]:
    return _invoke("apply", locals())


def survey(repo: Path = REPO) -> list[tuple[str, str]]:
    return _invoke("survey", locals())


def cell_crate(target: str, repo: Path = REPO) -> str:
    return _invoke("cell_crate", locals())


def _cell_is_bodyless(target: str, repo: Path) -> bool:
    return _invoke("_cell_is_bodyless", locals())


def _cell_uses_panic_common(target: str, arch: str, repo: Path) -> bool:
    return _invoke("_cell_uses_panic_common", locals())


def _baseline_pe(baseline: Path, arch: str, target: str) -> list[Path]:
    return _invoke("_baseline_pe", locals())


def reshell(targets: list[str], baseline: Path, repo: Path = REPO) -> None:
    return _invoke("reshell", locals())


def check(targets: list[str], baseline: Path, repo: Path = REPO) -> int:
    return _invoke("check", locals())


def remap(crates: list[str] | None = None, repo: Path = REPO) -> list[str]:
    return _invoke("remap", locals())


def c_template_path(arch: str, repo: Path = REPO) -> Path:
    return _invoke("c_template_path", locals())


def _c_parts(cmd: str) -> CParts:
    return _invoke("_c_parts", locals())


def _flag_enc(tok: str, arch: str) -> str:
    return _invoke("_flag_enc", locals())


def _flag_dec(tok: str, arch: str) -> str:
    return _invoke("_flag_dec", locals())


def _c_commands(arch: str, repo: Path = REPO) -> list[dict]:
    return _invoke("_c_commands", locals())


def _majority_run(lists: list[list[str]], share: float = 0.95) -> list[str]:
    return _invoke("_majority_run", locals())


def derive_c_template(arch: str, repo: Path = REPO) -> CTemplate:
    return _invoke("derive_c_template", locals())


def read_c_template(arch: str, repo: Path = REPO) -> CTemplate:
    return _invoke("read_c_template", locals())


def c_decode(f: CFlags, tmpl: CTemplate, arch: str) -> list[str]:
    return _invoke("c_decode", locals())


def _with_marker(seq: list[str], run: list[str], marker: str) -> list[str] | None:
    return _invoke("_with_marker", locals())


def c_encode(cmd: str, tmpl: CTemplate, arch: str) -> CFlags:
    return _invoke("c_encode", locals())


def c_lib_for(target: str, repo: Path = REPO) -> tuple[dict, dict]:
    return _invoke("c_lib_for", locals())


def _cmakelists(repo: Path) -> list[Path]:
    return _invoke("_cmakelists", locals())


def own_rc(crates: list[str] | None = None, repo: Path = REPO) -> list[str]:
    return _invoke("own_rc", locals())


def asm_template_path(arch: str, repo: Path = REPO) -> Path:
    return _invoke("asm_template_path", locals())


def _asm_parts(cmd: str) -> CParts:
    return _invoke("_asm_parts", locals())


def _asm_commands(arch: str, repo: Path = REPO) -> list[dict]:
    return _invoke("_asm_commands", locals())


def derive_asm_template(arch: str, repo: Path = REPO) -> CTemplate:
    return _invoke("derive_asm_template", locals())


def read_asm_template(arch: str, repo: Path = REPO) -> CTemplate:
    return _invoke("read_asm_template", locals())


def asm_decode(f: CFlags, tmpl: CTemplate, arch: str, obj: str) -> list[str]:
    return _invoke("asm_decode", locals())


def asm_encode(cmd: str, tmpl: CTemplate, arch: str) -> CFlags:
    return _invoke("asm_encode", locals())


def clib_recipe_path(arch: str, target: str, repo: Path = REPO) -> Path:
    return _invoke("clib_recipe_path", locals())


def clib_recipe(arch: str, target: str, repo: Path = REPO) -> str:
    return _invoke("clib_recipe", locals())


def engine_c_targets(repo: Path = REPO) -> list[str]:
    return _invoke("engine_c_targets", locals())


def _call_span(text: str, start: int) -> int:
    return _invoke("_call_span", locals())


def c_binding(srcs: dict, flags: dict) -> list[str]:
    return _invoke("c_binding", locals())


def own_c(crates: list[str] | None = None, repo: Path = REPO) -> list[str]:
    return _invoke("own_c", locals())


def _drop_shims_in(cmake: Path, target: str, generated: list[str]) -> None:
    return _invoke("_drop_shims_in", locals())


def _rc_flags_by_analogy(rc_rel: str, repo: Path) -> dict:
    return _invoke("_rc_flags_by_analogy", locals())


def _rc_object_depends(cmd: str, raw: str) -> list[str] | None:
    return _invoke("_rc_object_depends", locals())


def _drop_res_library(target: str, repo: Path) -> None:
    return _invoke("_drop_res_library", locals())


def _drop_res_in(cmake: Path, target: str) -> None:
    return _invoke("_drop_res_in", locals())


def save_baseline(targets: list[str], baseline: Path, repo: Path = REPO) -> None:
    return _invoke("save_baseline", locals())


def _ninja(arch: str, targets: list[str], repo: Path) -> None:
    return _invoke("_ninja", locals())


def refresh_manifests(repo: Path = REPO) -> None:
    return _invoke("refresh_manifests", locals())


# The old functions were cached; each native request is stateless.
for _name in ("_read", "_ccdb", "_library_index", "spec_index", "txtsetup_files"):
    globals()[_name].cache_clear = _no_cache


# Preserve convert's mockable transaction ordering. CLI conversion is native.
def convert(targets: list[str], baseline: Path | None = None, repo: Path = REPO) -> int:
    if not targets:
        raise GenError("--convert needs at least one target")
    cells = [model(target, repo) for target in targets]
    baseline = baseline or (
        Path.home() / "vibeos-cargo-baselines" / time.strftime("convert-%Y%m%d-%H%M%S")
    )
    if _rpc("baseline_reusable", {"targets": targets, "baseline": baseline}, repo):
        print(f"baseline: {baseline} (kept from the first run)")
    else:
        save_baseline(targets, baseline, repo)
        print(f"baseline: {baseline}")
    for cell in cells:
        for path in apply(cell, repo):
            print(f"wrote {path.relative_to(repo)}")
    for arch in ARCHES:
        _ninja(arch, targets, repo)
        print(f"built {len(targets)} cell(s) on {arch}")
    result = check(targets, baseline, repo)
    if result == 0:
        refresh_manifests(repo)
    return result


def main(argv: list[str] | None = None) -> int:
    # Invoke the native CLI in this process so exit status and signals propagate.
    tool = REPO / "tools/codegen"
    os.execv(str(tool), [str(tool), "cargo-cell", *(sys.argv[1:] if argv is None else argv)])
    raise AssertionError("execv returned")


if __name__ == "__main__":
    sys.exit(main())
