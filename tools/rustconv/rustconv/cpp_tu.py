"""Deterministic whole-translation-unit preflight for shell32 C++ tails."""

from __future__ import annotations

import json
import re
from dataclasses import asdict, dataclass, field
from pathlib import Path

from .cpp_com import CppComError, analyze_class
from .scanner import scan_file


class CppTuError(ValueError):
    pass


@dataclass(frozen=True)
class TuClass:
    name: str
    methods: tuple[str, ...]
    ownership: str
    rust_owner: str | None = None
    activation: str | None = None
    interfaces: tuple[str, ...] = ()
    state_fields: tuple[str, ...] = ()


@dataclass
class CppTuReport:
    source: Path
    functions: tuple[str, ...]
    free_functions: tuple[str, ...]
    classes: list[TuClass] = field(default_factory=list)
    unextracted: tuple[str, ...] = ()
    unsupported: list[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return not self.unextracted and not self.unsupported

    def to_dict(self) -> dict[str, object]:
        data = asdict(self)
        data["source"] = str(self.source)
        data["ok"] = self.ok
        return data

    def to_json(self) -> str:
        return json.dumps(self.to_dict(), indent=2, sort_keys=True)


def _find_rust_factory(repo: Path, class_name: str) -> Path | None:
    pattern = re.compile(
        rf"#\s*\[\s*no_mangle\s*\]\s*"
        rf"pub\s+(?:unsafe\s+)?extern\s+\"(?:system|C)\"\s+fn\s+"
        rf"{re.escape(class_name)}_CreateInstance\s*\(",
        re.MULTILINE,
    )
    rust_root = repo / "sdk/lib/rust"
    for crate in ("shell32-rust", "menuband-rust"):
        src = rust_root / crate / "src"
        if not src.is_dir():
            continue
        for path in sorted(src.rglob("*.rs")):
            if pattern.search(path.read_text(encoding="utf-8", errors="replace")):
                return path
    return None


def _rust_activation_adapter_owner(function: object, repo: Path) -> Path | None:
    """Prove that a nested ATL creator is only an adapter to a Rust factory."""
    if (
        getattr(function, "cpp_class", None) != "_CreatorClass"
        or getattr(function, "name", None) != "CreateInstance"
    ):
        return None
    body = getattr(function, "body_text", "")
    owners = set(re.findall(r"\b([A-Za-z_]\w*)_CreateInstance\s*\(", body))
    if len(owners) != 1:
        return None
    return _find_rust_factory(repo, owners.pop())


def _find_local_raii_owner(owners: tuple[Path, ...], class_name: str) -> Path | None:
    marker = re.compile(rf"^// --local-raii {re.escape(class_name)}:", re.MULTILINE)
    definition = re.compile(rf"^struct\s+{re.escape(class_name)}\b", re.MULTILINE)
    matches: list[Path] = []
    for owner in owners:
        text = owner.read_text(encoding="utf-8", errors="replace")
        if marker.search(text) and definition.search(text):
            matches.append(owner)
    if len(matches) > 1:
        raise CppTuError(
            f"{class_name}: ambiguous accepted --local-raii owners: "
            + ", ".join(str(path) for path in matches)
        )
    return matches[0] if matches else None


def analyze_translation_unit(source: Path, repo: Path) -> CppTuReport:
    source = source.resolve()
    repo = repo.resolve()
    if not source.is_file() or source.suffix.lower() not in {".cpp", ".cc", ".cxx"}:
        raise CppTuError(f"C++ source not found: {source}")
    scan = scan_file(source)
    classes: dict[str, list[str]] = {}
    free_functions: list[str] = []
    for function in scan.functions:
        if _rust_activation_adapter_owner(function, repo) is not None:
            continue
        if function.cpp_class:
            classes.setdefault(function.cpp_class, []).append(function.name)
        else:
            free_functions.append(function.name)

    report = CppTuReport(
        source=source,
        functions=tuple(function.display_name for function in scan.functions),
        free_functions=tuple(free_functions),
        unextracted=tuple(scan.unextracted),
    )
    object_map = repo / "dll/win32/shell32/shell32.cpp"
    factory_owners = {
        class_name: owner
        for class_name in classes
        if (owner := _find_rust_factory(repo, class_name)) is not None
    }
    owner_paths = tuple(dict.fromkeys(factory_owners.values()))
    for class_name, methods in classes.items():
        rust_owner = factory_owners.get(class_name)
        if rust_owner is not None:
            report.classes.append(
                TuClass(
                    name=class_name,
                    methods=tuple(methods),
                    ownership="rust-owned",
                    rust_owner=rust_owner.relative_to(repo).as_posix(),
                )
            )
            continue
        local_raii_owner = _find_local_raii_owner(owner_paths, class_name)
        if local_raii_owner is not None:
            report.classes.append(
                TuClass(
                    name=class_name,
                    methods=tuple(methods),
                    ownership="rust-owned-local-raii",
                    rust_owner=local_raii_owner.relative_to(repo).as_posix(),
                )
            )
            continue
        try:
            cls = analyze_class(
                source,
                class_name,
                object_map_source=object_map,
                method_names=methods,
            )
            cls.require_supported(allow_internal=True)
        except CppComError as exc:
            report.unsupported.append(str(exc))
            continue
        activation = "object-map" if cls.object_map_clsid else "internal-shell-object-creator"
        report.classes.append(
            TuClass(
                name=class_name,
                methods=tuple(methods),
                ownership="translate",
                activation=activation,
                interfaces=tuple(cls.interface_names()),
                state_fields=tuple(cls.state_fields),
            )
        )
    return report


def print_report(report: CppTuReport, *, as_json: bool = False) -> None:
    if as_json:
        print(report.to_json())
        return
    print(
        f"cpp-tu: {'PASS' if report.ok else 'FAIL'} — {report.source.name}: "
        f"{len(report.functions)} functions, {len(report.classes)} classes, "
        f"{len(report.free_functions)} free"
    )
    for cls in report.classes:
        detail = cls.rust_owner or cls.activation or ""
        print(f"  {cls.name}: {cls.ownership} ({len(cls.methods)} methods) {detail}".rstrip())
    for name in report.free_functions:
        print(f"  free: {name}")
    for item in report.unextracted:
        print(f"  UNEXTRACTED: {item}")
    for item in report.unsupported:
        print("  UNSUPPORTED: " + item.replace("\n", "\n    "))
