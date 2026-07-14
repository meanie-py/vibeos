"""Build ntkrnl-sys type/function/constant lookup from .rs files."""

import re
from dataclasses import dataclass, field
from pathlib import Path

from .config import (
    HAND_WRITTEN_MODULES,
    NTKRNL_SYS_DIR,
    TARGETS,
    Target,
)


@dataclass
class Symbol:
    name: str
    kind: str  # "struct", "type", "fn", "const", "static"
    module: str  # source module: "bindgen_arm64", "base", "ex", etc.
    calling_convention: str | None = None  # for fn: "C", "system", "stdcall", "fastcall"


@dataclass
class Catalog:
    """Index of all symbols available in ntkrnl-sys."""

    types: dict[str, Symbol] = field(default_factory=dict)  # struct + type alias
    functions: dict[str, Symbol] = field(default_factory=dict)  # fn declarations
    constants: dict[str, Symbol] = field(default_factory=dict)  # const + static

    @property
    def all_names(self) -> set[str]:
        return set(self.types) | set(self.functions) | set(self.constants)

    def lookup(self, name: str) -> Symbol | None:
        return self.types.get(name) or self.functions.get(name) or self.constants.get(name)


# Regex patterns for hand-written modules
RE_PUB_STRUCT = re.compile(r"^pub struct (\w+)")
RE_PUB_TYPE = re.compile(r"^pub type (\w+)")
RE_PUB_CONST = re.compile(r"^pub const (\w+)")
# bindgen renders tagged-enum members as `pub const <EnumType>_<C_NAME>:
# <EnumType> = N;` — the C source uses the bare <C_NAME>. Capture both.
RE_PUB_CONST_TYPED = re.compile(r"^pub const (\w+)\s*:\s*(\w+)")
RE_PUB_STATIC = re.compile(r"^pub static (?:mut )?(\w+)")
RE_PUB_FN_DECL = re.compile(r"^\s+pub fn (\w+)\(")  # inside extern block
RE_PUB_FN_IMPL = re.compile(r"^pub (?:unsafe )?fn (\w+)")
RE_EXTERN_BLOCK = re.compile(r'^extern "(\w+)"')
RE_UNSAFE_EXTERN_BLOCK = re.compile(r'^unsafe extern "(\w+)"')

# For bindgen files (each fn is in its own `unsafe extern "C" { pub fn ... }`)
RE_BINDGEN_FN = re.compile(r"^\s+pub fn (\w+)\(")
RE_BINDGEN_EXTERN = re.compile(r'^unsafe extern "(\w+)"')


def _scan_hand_written(
    path: Path, module_name: str
) -> tuple[list[Symbol], list[Symbol], list[Symbol]]:
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
            if "{" in stripped:
                brace_depth += stripped.count("{") - stripped.count("}")
            continue

        if in_extern_block:
            brace_depth += stripped.count("{") - stripped.count("}")
            if brace_depth <= 0:
                in_extern_block = False
                current_extern_cc = None
                brace_depth = 0

            m = RE_PUB_FN_DECL.match(line)
            if m:
                name = m.group(1)
                # Skip bitfield accessors (lowercase start)
                if name[0].isupper() or name.startswith("_"):
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
            if not name.startswith("__"):  # skip __BindgenUnionField etc.
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
            cname = m.group(1)
            constants.append(Symbol(cname, "const", module_name))
            # bindgen prefixes tagged-enum constants as `<EnumType>_<C_NAME>`;
            # the C uses the bare <C_NAME>. Register that alias too (general
            # — also helps ntkrnl-sys/hal enum constants).
            mt = RE_PUB_CONST_TYPED.match(stripped)
            if mt:
                etype = mt.group(2)
                pfx = etype + "_"
                if cname.startswith(pfx) and len(cname) > len(pfx):
                    constants.append(Symbol(cname[len(pfx) :], "const", module_name))
            continue

        # Extern static data — bindgen emits `pub static mut NAME: TYPE;`
        # inside `unsafe extern "C" {}` blocks for data exports
        # (CsrObjectDirectory/CsrHeap/CsrRootProcess; kernel data like
        # KdDebuggerNotPresent). Previously dropped: only
        # _scan_hand_written matched RE_PUB_STATIC, _scan_bindgen never
        # did, so every bindgen data export was a phantom "unresolved".
        # Generic fix — helps every target, not just smss-csrss.
        m = RE_PUB_STATIC.match(stripped)
        if m and current_extern_cc:
            constants.append(Symbol(m.group(1), "static", module_name))
            continue

        # Function declarations (inside extern blocks)
        m = RE_BINDGEN_FN.match(line)
        if m and current_extern_cc:
            name = m.group(1)
            # Skip bitfield accessor methods (start lowercase, inside impl blocks)
            # Real extern fns start with uppercase or _
            if name[0].isupper() or name.startswith("_"):
                functions.append(Symbol(name, "fn", module_name, current_extern_cc))

    return types, functions, constants


def build_catalog(arch: str = "arm64", target: Target | None = None) -> Catalog:
    """Build a complete symbol catalog from ntkrnl-sys sources.

    Args:
        arch: Which bindgen file to use ("arm64", "amd64", "i386")
        target: Target descriptor (selects ntoskrnl vs hal scope).  Falls back
                to TARGETS["ntoskrnl"] for backward compatibility.
    """
    if target is None:
        target = TARGETS["ntoskrnl"]

    catalog = Catalog()

    # Scan hand-written modules first (higher priority).  These are always
    # available regardless of target.
    for filename in HAND_WRITTEN_MODULES:
        path = NTKRNL_SYS_DIR / filename
        if not path.exists():
            continue
        module_name = filename.replace(".rs", "")
        types, functions, constants = _scan_hand_written(path, module_name)
        for s in types:
            catalog.types[s.name] = s
        for s in functions:
            catalog.functions[s.name] = s
        for s in constants:
            catalog.constants[s.name] = s

    # Scan the kernel bindgen file → module "ddk"
    bindgen_path = target.bindgen_files.get(arch)
    if bindgen_path and bindgen_path.exists():
        types, functions, constants = _scan_bindgen(bindgen_path, "ddk")
        for s in types:
            if s.name not in catalog.types:  # hand-written takes priority
                catalog.types[s.name] = s
        for s in functions:
            if s.name not in catalog.functions:
                catalog.functions[s.name] = s
        for s in constants:
            if s.name not in catalog.constants:
                catalog.constants[s.name] = s

    # For HAL target, also scan hal_bindgen file → module "hal_ddk"
    # Symbols in hal_bindgen override ddk for HAL-internal types (ADAPTER_OBJECT,
    # DEVICE_DESCRIPTION, DMA_PAGE, etc.) — those are HAL-private and must come
    # from hal_ddk to keep the type-identity correct in struct fields.
    if target.name == "hal":
        hal_bindgen_path = target.hal_bindgen_files.get(arch)
        if hal_bindgen_path and hal_bindgen_path.exists():
            types, functions, constants = _scan_bindgen(hal_bindgen_path, "hal_ddk")
            # hal_ddk wins for types/constants — HAL-internal definitions are
            # what the C code actually uses for these symbols.
            for s in types:
                catalog.types[s.name] = s
            for s in constants:
                catalog.constants[s.name] = s
            # Functions: ddk wins (kernel API is the linkable surface);
            # hal_ddk fn decls are usually duplicates of ddk for re-exports.
            for s in functions:
                if s.name not in catalog.functions:
                    catalog.functions[s.name] = s

    # For the win32k target, also scan win32k-sys bindgen → module "win32k".
    # win32k calls both the ntoskrnl API (ddk, scanned above) and its own
    # internal surface. win32k-internal types/constants are authoritative
    # for win32k code (the C uses _WND/_DESKTOP/NOPARAM_ROUTINE_* layouts
    # from win32k headers); functions add-if-absent so the shared kernel
    # API (ddk) stays the linkable surface and win32k adds Int*/Gre*/co_*.
    win32k_files = getattr(target, "win32k_bindgen_files", {}) or {}
    win32k_bindgen_path = win32k_files.get(arch)
    if win32k_bindgen_path and win32k_bindgen_path.exists():
        types, functions, constants = _scan_bindgen(win32k_bindgen_path, "win32k")
        for s in types:
            catalog.types[s.name] = s
        for s in constants:
            catalog.constants[s.name] = s
        for s in functions:
            if s.name not in catalog.functions:
                catalog.functions[s.name] = s

    # For the winsrv target, also scan winsrv-sys bindgen → module "winsrv".
    # winsrv calls both ntdll-sys (NT/CSR/SM/BASE — scanned above as "ddk")
    # AND its own internal surface (CONSOLE_PROCESS_DATA, TERMINAL_VTBL,
    # FRONTEND_VTBL, SHUTDOWN_SETTINGS, CONSOLE_*, COORD, INPUT_RECORD,
    # HWND/HDC/LOGFONTW/MSG/WNDPROC, …). winsrv-internal types/constants
    # are authoritative for winsrv code; functions add-if-absent so the
    # NT surface stays the linkable surface and winsrv adds Con*/Term*/
    # Concfg* helpers. Mirrors the win32k branch.
    winsrv_files = getattr(target, "winsrv_bindgen_files", {}) or {}
    winsrv_bindgen_path = winsrv_files.get(arch)
    if winsrv_bindgen_path and winsrv_bindgen_path.exists():
        types, functions, constants = _scan_bindgen(winsrv_bindgen_path, "winsrv")
        for s in types:
            catalog.types[s.name] = s
        for s in constants:
            catalog.constants[s.name] = s
        for s in functions:
            if s.name not in catalog.functions:
                catalog.functions[s.name] = s

    # Generic CRATE-LOCAL bindgen island (gen_driver_types.sh → <crate>/src/types_<arch>.rs).
    # The generalization of the win32k_/winsrv_ branches above, for any DLL/driver whose
    # private object model is generated rather than shared — ws2_32 (TCATALOG/TPROVIDER/
    # NSCATALOG/NSQUERY/WSPROCESS/WSTHREAD/WSSOCKET/WSASYNCBLOCK from inc/ws2_32p.h, plus
    # the winsock/ws2spi surface: INVALID_SOCKET, LPWSABUF, LPQOS, AF_*, …).
    #
    # Island types/constants are AUTHORITATIVE (same as win32k/winsrv): the C in this module
    # is compiled against exactly these headers, so its field accesses and constant values
    # are the island's, not some other module's same-named symbol. Functions add-if-absent so
    # the shared NT/kernel surface stays the linkable one.
    crate_files = getattr(target, "crate_bindgen_files", {}) or {}
    crate_bindgen_path = crate_files.get(arch)
    if crate_bindgen_path and crate_bindgen_path.exists():
        module = getattr(target, "crate_module", "types") or "types"
        types, functions, constants = _scan_bindgen(crate_bindgen_path, module)
        for s in types:
            catalog.types[s.name] = s
        for s in constants:
            catalog.constants[s.name] = s
        for s in functions:
            if s.name not in catalog.functions:
                catalog.functions[s.name] = s

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
    print(
        f"Catalog: {len(catalog.types)} types, {len(catalog.functions)} functions, "
        f"{len(catalog.constants)} constants"
    )

    # Group by module
    modules: dict[str, int] = {}
    for sym in (
        list(catalog.types.values())
        + list(catalog.functions.values())
        + list(catalog.constants.values())
    ):
        modules[sym.module] = modules.get(sym.module, 0) + 1

    for mod_name, count in sorted(modules.items()):
        print(f"  {mod_name}: {count} symbols")
