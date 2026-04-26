"""C source parser using tree-sitter for structural analysis."""

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import tree_sitter_c as tsc
from tree_sitter import Language, Parser

C_LANG = Language(tsc.language())

# Calling convention macros
CALLING_CONVENTIONS = {"NTAPI", "FASTCALL", "WINAPI", "CDECL", "__cdecl", "__stdcall", "__fastcall"}

# Section markers in ReactOS C files
RE_GLOBALS_SECTION = re.compile(r'/\*\s*GLOBALS\s*\*/')
RE_FUNCTIONS_SECTION = re.compile(r'/\*\s*FUNCTIONS\s*\*/')

# Arch preprocessor conditionals
RE_IFDEF_ARCH = re.compile(r'#\s*if(?:def|ndef)?\s+.*?(_M_ARM64|_M_AMD64|_M_IX86|_M_ARM|_WIN64)')

# SEH macros
RE_SEH_TRY = re.compile(r'_SEH2_TRY')
RE_SEH_EXCEPT = re.compile(r'_SEH2_EXCEPT')

# CODE_SEG
RE_CODE_SEG = re.compile(r'CODE_SEG\("(\w+)"\)')


@dataclass
class CFunction:
    name: str
    return_type: str
    calling_convention: Optional[str]  # "NTAPI", "FASTCALL", None
    params: list[tuple[str, str]]  # [(type, name), ...]
    is_static: bool
    code_seg: Optional[str]  # "INIT", "PAGE", etc.
    seh_count: int  # number of _SEH2_TRY blocks
    line_start: int
    line_end: int
    body_text: str  # raw C source of the function body


@dataclass
class CGlobal:
    name: str
    type_str: str
    initializer: Optional[str]
    line: int


@dataclass
class ScanResult:
    path: Path
    globals: list[CGlobal] = field(default_factory=list)
    functions: list[CFunction] = field(default_factory=list)
    identifiers: set[str] = field(default_factory=set)
    arch_ifdefs: list[str] = field(default_factory=list)  # arch macros used
    includes: list[str] = field(default_factory=list)
    defines: list[str] = field(default_factory=list)
    raw_source: str = ""
    globals_text: str = ""  # text between GLOBALS and FUNCTIONS markers


def _extract_text(node, source: bytes) -> str:
    return source[node.start_byte:node.end_byte].decode('utf-8', errors='replace')


def _collect_identifiers(node, source: bytes, identifiers: set[str]) -> None:
    """Recursively collect all identifier tokens from a tree-sitter node."""
    if node.type == 'identifier' or node.type == 'type_identifier':
        name = _extract_text(node, source)
        if len(name) > 1:  # skip single-char vars
            identifiers.add(name)
    for child in node.children:
        _collect_identifiers(child, source, identifiers)


def _parse_function(node, source: bytes, source_text: str) -> Optional[CFunction]:
    """Parse a function_definition node into CFunction."""
    name = None
    return_type = ""
    calling_convention = None
    params = []
    is_static = False
    code_seg = None

    # Check for storage class and CODE_SEG in the preceding lines
    start_line = node.start_point[0]
    # Look at up to 5 preceding lines for CODE_SEG / static
    line_start = max(0, start_line - 5)
    preceding = "\n".join(source_text.splitlines()[line_start:start_line + 1])
    m = RE_CODE_SEG.search(preceding)
    if m:
        code_seg = m.group(1)

    # Walk the declaration
    declarator = None
    for child in node.children:
        text = _extract_text(child, source)

        if child.type == 'storage_class_specifier' and text == 'static':
            is_static = True
        elif child.type == 'type_identifier' or child.type == 'primitive_type' or child.type == 'sized_type_specifier':
            if text in CALLING_CONVENTIONS:
                calling_convention = text
            else:
                return_type = text
        elif child.type == 'identifier':
            if text in CALLING_CONVENTIONS:
                calling_convention = text
            elif return_type == "":
                return_type = text
        elif child.type == 'function_declarator':
            declarator = child
        elif child.type == 'pointer_declarator':
            # Return type is a pointer
            for sub in child.children:
                if sub.type == 'function_declarator':
                    declarator = sub

    if declarator is None:
        return None

    # Extract function name from declarator
    for child in declarator.children:
        if child.type == 'identifier':
            name = _extract_text(child, source)
        elif child.type == 'parameter_list':
            for param_node in child.children:
                if param_node.type == 'parameter_declaration':
                    param_text = _extract_text(param_node, source)
                    # Crude but effective: split into type and name
                    parts = param_text.rsplit(None, 1)
                    if len(parts) == 2:
                        ptype, pname = parts
                        # Remove pointer markers from name
                        while pname.startswith('*'):
                            ptype += ' *'
                            pname = pname[1:]
                        params.append((ptype.strip(), pname.strip()))
                    elif parts and parts[0] != 'VOID':
                        params.append((parts[0], ""))

    if name is None:
        return None

    # Count SEH blocks in body
    body_node = None
    for child in node.children:
        if child.type == 'compound_statement':
            body_node = child
            break

    body_text = _extract_text(node, source)
    seh_count = body_text.count('_SEH2_TRY')

    return CFunction(
        name=name,
        return_type=return_type,
        calling_convention=calling_convention,
        params=params,
        is_static=is_static,
        code_seg=code_seg,
        seh_count=seh_count,
        line_start=node.start_point[0] + 1,
        line_end=node.end_point[0] + 1,
        body_text=body_text,
    )


def scan_file(path: Path) -> ScanResult:
    """Parse a C source file and extract structural information."""
    source_text = path.read_text(errors='replace')
    source = source_text.encode('utf-8')

    parser = Parser(C_LANG)
    tree = parser.parse(source)

    result = ScanResult(path=path, raw_source=source_text)

    # Extract globals text (between GLOBALS and FUNCTIONS markers)
    lines = source_text.splitlines()
    globals_start = None
    functions_start = None
    for i, line in enumerate(lines):
        if RE_GLOBALS_SECTION.search(line):
            globals_start = i + 1
        if RE_FUNCTIONS_SECTION.search(line):
            functions_start = i
            break

    if globals_start is not None and functions_start is not None:
        result.globals_text = "\n".join(lines[globals_start:functions_start])

    # Extract arch ifdefs
    for line in lines:
        m = RE_IFDEF_ARCH.search(line)
        if m:
            macro = m.group(1)
            if macro not in result.arch_ifdefs:
                result.arch_ifdefs.append(macro)

    # Walk the AST
    cursor = tree.walk()

    def walk(node):
        if node.type == 'function_definition':
            func = _parse_function(node, source, source_text)
            if func:
                result.functions.append(func)
            # Collect identifiers from the function
            _collect_identifiers(node, source, result.identifiers)
            return  # don't recurse into function body again

        if node.type == 'declaration':
            # Top-level declarations could be globals
            _collect_identifiers(node, source, result.identifiers)

        if node.type == 'preproc_include':
            for child in node.children:
                if child.type in ('string_literal', 'system_lib_string'):
                    result.includes.append(_extract_text(child, source))

        if node.type == 'preproc_def':
            for child in node.children:
                if child.type == 'identifier':
                    result.defines.append(_extract_text(child, source))
                    break

        for child in node.children:
            walk(child)

    walk(cursor.node)

    # Parse globals from the globals section text using simple regex
    if result.globals_text:
        for line in result.globals_text.splitlines():
            line = line.strip()
            if not line or line.startswith('//') or line.startswith('/*'):
                continue
            # Match patterns like: TYPE NAME = INIT;  or  TYPE NAME;
            # This is simplified — globals can be complex
            m = re.match(r'^(\w[\w\s\*]+?)\s+(\w+)\s*(?:=\s*(.+?))?\s*;', line)
            if m:
                result.globals.append(CGlobal(
                    name=m.group(2),
                    type_str=m.group(1).strip(),
                    initializer=m.group(3).strip() if m.group(3) else None,
                    line=0,  # approximate
                ))

    return result


def print_scan_result(result: ScanResult) -> None:
    """Print scan results in human-readable form."""
    print(f"File: {result.path}")
    print(f"Includes: {len(result.includes)}")
    print(f"Defines: {len(result.defines)}")
    print(f"Arch ifdefs: {result.arch_ifdefs or 'none'}")
    print()

    if result.globals:
        print(f"Globals ({len(result.globals)}):")
        for g in result.globals:
            init = f" = {g.initializer}" if g.initializer else ""
            print(f"  {g.type_str} {g.name}{init}")
        print()

    print(f"Functions ({len(result.functions)}):")
    for f in result.functions:
        cc = f.calling_convention or ""
        static = "static " if f.is_static else ""
        seg = f"[{f.code_seg}] " if f.code_seg else ""
        seh = f" (SEH×{f.seh_count})" if f.seh_count else ""
        params_str = ", ".join(f"{t} {n}" for t, n in f.params)
        print(f"  {seg}{static}{f.return_type} {cc} {f.name}({params_str}){seh}  L{f.line_start}-{f.line_end}")

    print(f"\nTotal identifiers: {len(result.identifiers)}")
