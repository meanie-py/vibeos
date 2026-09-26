"""Bounded stable-Rust/ABI normalization of saved compiler output; never buys code."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shlex
import subprocess
from pathlib import Path

from . import c2rust_anchor
from .attempts import durable_json
from .lex_mask import mask_code

WIDTHS = {
    "short": "__SIZEOF_SHORT__",
    "int": "__SIZEOF_INT__",
    "long": "__SIZEOF_LONG__",
    "longlong": "__SIZEOF_LONG_LONG__",
}


def compiler_facts(source: Path, arch: str) -> dict:
    entry = c2rust_anchor._find_db_entry(source, arch)
    if entry is None:
        raise ValueError(f"Missing {arch} compile command for ABI derivation")
    command, cwd = entry
    tokens = command.get("arguments") or shlex.split(command["command"])
    compiler = c2rust_anchor._decanon(tokens[0])
    flags = c2rust_anchor._extract_clang_flags(command)
    argv = [compiler, *flags, "-dM", "-E", "-x", "c", "-"]
    proc = subprocess.run(argv, input="", cwd=cwd, text=True, capture_output=True, check=True)
    macros = dict(re.findall(r"^#define (\w+) (.+)$", proc.stdout, re.M))
    names = [*WIDTHS.values(), "__SIZEOF_POINTER__", "__SIZEOF_WCHAR_T__", "__CHAR_BIT__"]
    if any(name not in macros for name in names):
        raise ValueError("Compiler did not expose required ABI widths")
    facts = {name: int(macros[name]) for name in names}
    facts["char_unsigned"] = "__CHAR_UNSIGNED__" in macros
    if facts["__CHAR_BIT__"] != 8:
        raise ValueError("Only eight-bit-byte C ABIs are supported")
    return {
        "arch": arch,
        "facts": facts,
        "argv": argv,
        "compiler_sha256": hashlib.sha256(Path(compiler).read_bytes()).hexdigest(),
    }


def lower(raw: str, facts: dict) -> tuple[str, list[dict]]:
    """Normalize scalar spellings, not source algorithms or library semantics."""
    result = raw
    repairs = []
    features = list(re.finditer(r"^#!\[feature\(([^)]+)\)\][ \t]*\n", mask_code(result), re.M))
    for match in reversed(features):
        feature = match.group(1)
        if feature.strip() not in {"raw_ref_op", "asm"}:
            raise ValueError(f"Unsupported nightly feature in compiler output: {feature}")
        result = result[: match.start()] + result[match.end() :]
        repairs.append({"kind": "stable_feature", "feature": feature})
    mapping = {"c_char": "u8" if facts["char_unsigned"] else "i8", "c_schar": "i8", "c_uchar": "u8"}
    for name, macro in WIDTHS.items():
        bits = facts[macro] * 8
        if bits not in {16, 32, 64, 128}:
            raise ValueError(f"Unsupported C integer width: {name}={bits}")
        mapping["c_" + name] = f"i{bits}"
        mapping["c_u" + name] = f"u{bits}"
    pattern = re.compile(r"::core::ffi::(c_\w+)\b")
    for match in reversed(list(pattern.finditer(mask_code(result)))):
        name = match.group(1)
        if name in {"c_void", "c_float", "c_double"}:
            continue
        if name not in mapping:
            raise ValueError(f"Unsupported C ABI primitive: {name}")
        result = result[: match.start()] + mapping[name] + result[match.end() :]
    repairs.append({"kind": "compiler_integer_widths", "mapping": mapping})
    # C's address-sized typedefs follow the destination pointer width. Other
    # integer types keep compiler-derived fixed widths, including Windows long.
    for alias, rust_type in {
        "size_t": "usize",
        "ptrdiff_t": "isize",
        "intptr_t": "isize",
        "uintptr_t": "usize",
    }.items():
        pattern = re.compile(r"\bpub type " + alias + r" = [^;]+;")
        matches = list(pattern.finditer(mask_code(result)))
        for match in reversed(matches):
            result = (
                result[: match.start()] + f"pub type {alias} = {rust_type};" + result[match.end() :]
            )
        if matches:
            repairs.append({"kind": "pointer_typedef", "alias": alias, "rust_type": rust_type})
    return result, repairs


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("raw", type=Path)
    origin = parser.add_mutually_exclusive_group(required=True)
    origin.add_argument("--source", type=Path, help="derive ABI from the current C compile command")
    origin.add_argument(
        "--abi-json", type=Path, help="replay a prior normalization's pinned ABI record"
    )
    parser.add_argument("--arch", choices=("i386", "amd64", "arm64"))
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    raw = args.raw.read_text()
    if args.abi_json:
        if args.arch:
            parser.error("--arch cannot override a saved ABI record")
        abi = replay_abi(args.abi_json, raw)
    else:
        if not args.arch:
            parser.error("--source requires --arch")
        abi = compiler_facts(args.source.resolve(), args.arch)
    adapted, repairs = lower(raw, abi["facts"])
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x") as stream:
        stream.write(adapted)
    durable_json(
        args.output.with_suffix(".json"),
        {
            "raw_sha256": hashlib.sha256(raw.encode()).hexdigest(),
            "candidate_sha256": hashlib.sha256(adapted.encode()).hexdigest(),
            "compiler_abi": abi,
            "repairs": repairs,
            "scope": "scalar ABI/stable syntax only; no semantic acceptance implied",
        },
    )
    print(args.output)


def replay_abi(path: Path, raw: str) -> dict:
    """Reuse measured ABI facts only for the exact raw candidate they accompanied."""
    record = json.loads(path.read_text())
    if record.get("raw_sha256") != hashlib.sha256(raw.encode()).hexdigest():
        raise ValueError("Saved ABI record does not match this raw compiler candidate")
    abi = record["compiler_abi"]
    if abi.get("arch") not in {"i386", "amd64", "arm64"} or not abi.get("compiler_sha256"):
        raise ValueError("Saved ABI record lacks target/compiler provenance")
    return abi


if __name__ == "__main__":
    main()
