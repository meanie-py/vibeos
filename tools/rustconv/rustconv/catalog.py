"""Build ntkrnl-sys type/function/constant lookup from .rs files."""

import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from .config import NTKRNL_SYS_DIR, BINDGEN_FILES, HAND_WRITTEN_MODULES


@dataclass
class Symbol:
    name: str
    kind: str  # "struct", "type", "fn", "const", "static"
    module: str  # source module: "bindgen_arm64", "base", "ex", etc.
    calling_convention: Optional[str] = None  # for fn: "C", "system", "stdcall", "fastcall"


@dataclass
class Catalog:
    """Index of all symbols available in ntkrnl-sys."""
    types: dict[str, Symbol] = field(default_factory=dict)     # struct + type alias
    functions: dict[str, Symbol] = field(default_factory=dict)  # fn declarations
    constants: dict[str, Symbol] = field(default_factory=dict)  # const + static

    @property
    def all_names(self) -> set[str]:
        return set(self.types) | set(self.functions) | set(self.constants)

    def lookup(self, name: str) -> Optional[Symbol]:
        return self.types.get(name) or self.functions.get(name) or self.constants.get(name)


# Regex patterns for hand-written modules
RE_PUB_STRUCT = re.compile(r'^pub struct (\w+)')
RE_PUB_TYPE = re.compile(r'^pub type (\w+)')
RE_PUB_CONST = re.compile(r'^pub const (\w+)')
RE_PUB_STATIC = re.compile(r'^pub static (?:mut )?(\w+)')
RE_PUB_FN_DECL = re.compile(r'^\s+pub fn (\w+)\(')  # inside extern block
RE_PUB_FN_IMPL = re.compile(r'^pub (?:unsafe )?fn (\w+)')
RE_EXTERN_BLOCK = re.compile(r'^extern "(\w+)"')
RE_UNSAFE_EXTERN_BLOCK = re.compile(r'^unsafe extern "(\w+)"')

# For bindgen files (each fn is in its own `unsafe extern "C" { pub fn ... }`)
RE_BINDGEN_FN = re.compile(r'^\s+pub fn (\w+)\(')
RE_BINDGEN_EXTERN = re.compile(r'^unsafe extern "(\w+)"')


def _scan_hand_written(path: Path, module_name: str) -> tuple[list[Symbol], list[Symbol], list[Symbol]]:
    """Scan a hand-written .rs file for symbols."""
    types, functions, constants = [], [], []
    current_extern_cc = None
    in_extern_block = False
    brace_depth = 0

    for line in path.read_text().splitlines():
        stripped = line.strip()

        # Track extern blocks
        m = RE_EXTERN_BLOCK.match(stripped) or RE_UNSAFE_EXTERN_BLOCK.match(stripped)
        if m:
            current_extern_cc = m.group(1)
            in_extern_block = True
            if '{' in stripped:
                brace_depth += stripped.count('{') - stripped.count('}')
            continue

        if in_extern_block:
            brace_depth += stripped.count('{') - stripped.count('}')
            if brace_depth <= 0:
                in_extern_block = False
                current_extern_cc = None
                brace_depth = 0

            m = RE_PUB_FN_DECL.match(line)
            if m:
                name = m.group(1)
                # Skip bitfield accessors (lowercase start)
                if name[0].isupper() or name.startswith('_'):
                    functions.append(Symbol(name, "fn", module_name, current_extern_cc))
                continue

        # Top-level items
        m = RE_PUB_STRUCT.match(stripped)
        if m:
            types.append(Symbol(m.group(1), "struct", module_name))
            continue

        m = RE_PUB_TYPE.match(stripped)
        if m:
            types.append(Symbol(m.group(1), "type", module_name))
            continue

        m = RE_PUB_CONST.match(stripped)
        if m:
            constants.append(Symbol(m.group(1), "const", module_name))
            continue

        m = RE_PUB_STATIC.match(stripped)
        if m:
            constants.append(Symbol(m.group(1), "static", module_name))
            continue

        m = RE_PUB_FN_IMPL.match(stripped)
        if m:
            functions.append(Symbol(m.group(1), "fn", module_name, "C"))
            continue

    return types, functions, constants


def _scan_bindgen(path: Path, module_name: str) -> tuple[list[Symbol], list[Symbol], list[Symbol]]:
    """Scan a bindgen-generated .rs file for symbols."""
    types, functions, constants = [], [], []
    current_extern_cc = None

    for line in path.read_text().splitlines():
        stripped = line.strip()

        # Track extern blocks
        m = RE_BINDGEN_EXTERN.match(stripped)
        if m:
            current_extern_cc = m.group(1)
            continue

        # Struct definitions
        m = RE_PUB_STRUCT.match(stripped)
        if m:
            name = m.group(1)
            if not name.startswith('__'):  # skip __BindgenUnionField etc.
                types.append(Symbol(name, "struct", module_name))
            continue

        # Type aliases
        m = RE_PUB_TYPE.match(stripped)
        if m:
            types.append(Symbol(m.group(1), "type", module_name))
            continue

        # Constants
        m = RE_PUB_CONST.match(stripped)
        if m:
            constants.append(Symbol(m.group(1), "const", module_name))
            continue

        # Function declarations (inside extern blocks)
        m = RE_BINDGEN_FN.match(line)
        if m and current_extern_cc:
            name = m.group(1)
            # Skip bitfield accessor methods (start lowercase, inside impl blocks)
            # Real extern fns start with uppercase or _
            if name[0].isupper() or name.startswith('_'):
                functions.append(Symbol(name, "fn", module_name, current_extern_cc))

    return types, functions, constants


def build_catalog(arch: str = "arm64") -> Catalog:
    """Build a complete symbol catalog from ntkrnl-sys sources.

    Args:
        arch: Which bindgen file to use ("arm64", "amd64", "i386")
    """
    catalog = Catalog()

    # Scan hand-written modules first (higher priority)
    for filename in HAND_WRITTEN_MODULES:
        path = NTKRNL_SYS_DIR / filename
        if not path.exists():
            continue
        module_name = filename.replace('.rs', '')
        types, functions, constants = _scan_hand_written(path, module_name)
        for s in types:
            catalog.types[s.name] = s
        for s in functions:
            catalog.functions[s.name] = s
        for s in constants:
            catalog.constants[s.name] = s

    # Scan bindgen file
    bindgen_path = BINDGEN_FILES.get(arch)
    if bindgen_path and bindgen_path.exists():
        module_name = f"ddk"
        types, functions, constants = _scan_bindgen(bindgen_path, module_name)
        for s in types:
            if s.name not in catalog.types:  # hand-written takes priority
                catalog.types[s.name] = s
        for s in functions:
            if s.name not in catalog.functions:
                catalog.functions[s.name] = s
        for s in constants:
            if s.name not in catalog.constants:
                catalog.constants[s.name] = s

    # Also scan bindgen_common.rs
    common_path = NTKRNL_SYS_DIR / "bindgen_common.rs"
    if common_path.exists():
        types, functions, constants = _scan_hand_written(common_path, "bindgen_common")
        for s in types:
            if s.name not in catalog.types:
                catalog.types[s.name] = s

    return catalog


def print_catalog_stats(catalog: Catalog) -> None:
    """Print summary statistics."""
    print(f"Catalog: {len(catalog.types)} types, {len(catalog.functions)} functions, {len(catalog.constants)} constants")

    # Group by module
    modules: dict[str, int] = {}
    for sym in list(catalog.types.values()) + list(catalog.functions.values()) + list(catalog.constants.values()):
        modules[sym.module] = modules.get(sym.module, 0) + 1

    for mod_name, count in sorted(modules.items()):
        print(f"  {mod_name}: {count} symbols")
