"""Compiler-derived provenance for a host C++ reference translation unit."""

from __future__ import annotations

import hashlib
import shlex
import subprocess
from pathlib import Path
from typing import TypedDict


class CppProvenance(TypedDict):
    source_sha256: str
    compiler: str
    compile_flags: list[str]
    dependency_argv: list[str]
    preprocess_argv: list[str]
    preprocessed_sha256: str
    headers: list[dict[str, str]]


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def cpp_reference_provenance(
    source: Path,
    compiler: str = "g++",
    compile_flags: tuple[str, ...] = ("-std=c++17", "-O2"),
) -> CppProvenance:
    source = source.resolve()
    dependency_argv = [compiler, *compile_flags, "-M", str(source)]
    dependencies = subprocess.run(
        dependency_argv, check=True, text=True, capture_output=True
    ).stdout
    paths = shlex.split(dependencies.replace("\\\n", " ").split(":", 1)[1])
    headers = sorted(
        {str(Path(path).resolve()) for path in paths if Path(path).resolve() != source}
    )
    preprocess_argv = [compiler, *compile_flags, "-E", "-P", str(source)]
    preprocessed = subprocess.run(preprocess_argv, check=True, capture_output=True).stdout
    version = subprocess.run(
        [compiler, "--version"], check=True, text=True, capture_output=True
    ).stdout.splitlines()[0]
    return {
        "source_sha256": digest(source.read_bytes()),
        "compiler": version,
        "compile_flags": list(compile_flags),
        "dependency_argv": dependency_argv,
        "preprocess_argv": preprocess_argv,
        "preprocessed_sha256": digest(preprocessed),
        "headers": [{"path": path, "sha256": digest(Path(path).read_bytes())} for path in headers],
    }
