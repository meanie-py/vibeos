"""Migrate single-package CMake Cargo storage declarations; preview unless --apply.

Only direct static-library cargo builds with a literal package qualify. Outer
cargo bins, multi-command target dirs and unusual declarations stay private.
"""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path

DECL = re.compile(r'set\((\w+)\s+"(\$\{CMAKE_CURRENT_BINARY_DIR\}/[^"\n]*target)"\)')
COMMAND = re.compile(r"add_custom_command\((.*?)(?=\n[a-zA-Z_]+\(|\Z)", re.S)


def migrate(source: str) -> tuple[str, list[str]]:
    changes = []

    def replace(match: re.Match[str]) -> str:
        variable, path = match.groups()
        commands = [c for c in COMMAND.findall(source) if f"CARGO_TARGET_DIR=${{{variable}}}" in c]
        if len(commands) != 1:
            return match[0]
        packages = re.findall(r"\s-p\s+([\w-]+)(?=\s)", commands[0])
        if len(packages) != 1 or packages[0].endswith("-cargo"):
            return match[0]
        package = packages[0]
        changes.append(package)
        return f'rust_storage_target({variable} "{path}" {package})'

    return DECL.sub(replace, source), changes


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    files = subprocess.check_output(
        ["git", "ls-files", "-z", "*CMakeLists.txt", "*.cmake"], cwd=root
    )
    count = 0
    for name in files.decode().split("\0"):
        if not name:
            continue
        path = root / name
        old = path.read_text()
        new, packages = migrate(old)
        if packages:
            print(f"{name}: {', '.join(packages)}")
            count += len(packages)
            if args.apply:
                path.write_text(new)
    print(f"{count} declarations {'migrated' if args.apply else 'eligible (dry-run)'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
