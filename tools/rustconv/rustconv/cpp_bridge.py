"""Compiler-owned C++ objects with C-compatible method workers.

This bounded lowering passes pointers to fields, never copies or guesses a C++
class layout. Original construction, destruction, QI and virtual dispatch remain
compiled C++. Selected method bodies become C reference workers for the ordinary
anchored translation pipeline. Unsupported member expressions fail before payment.
"""

import re
from dataclasses import dataclass

import tree_sitter_cpp
from tree_sitter import Language, Parser

from .cpp_com import _CLASS_HEAD, _balanced_block, _state_fields
from .lex_mask import mask_code
from .scanner import ScanResult


class BridgeError(ValueError):
    pass


@dataclass
class BridgeOutput:
    header: str
    reference: str
    bridge: str
    workers: tuple[str, ...]
    retained: tuple[str, ...]


def generate_bridge(
    source: str,
    scan: ScanResult,
    class_name: str,
    methods: list[str],
    *,
    header_name: str,
    includes: str,
    macros: str = "",
) -> BridgeOutput:
    """Generate a reviewable, compiler-checkable bridge for an explicit method slice.

    `includes` supplies the real C-compatible SDK closure. `macros` contains exact
    source macro definitions needed by the slice. Neither is model-generated.
    """
    if not methods or len(methods) != len(set(methods)):
        raise BridgeError("select unique method names")
    candidates = [f for f in scan.functions if f.cpp_class == class_name]
    by_name = {f.name: f for f in candidates if f.return_type}
    missing = set(methods) - by_name.keys()
    if missing:
        raise BridgeError(f"missing ordinary methods: {sorted(missing)}")
    selected = [by_name[name] for name in methods]
    masked = mask_code(source)
    heads = [m for m in _CLASS_HEAD.finditer(masked) if m.group("name") == class_name]
    if len(heads) != 1:
        raise BridgeError("class definition must be unique")
    _, class_end = _balanced_block(masked, heads[0].end() - 1)
    fields = {}
    for declaration in _state_fields(source[heads[0].end() : class_end]):
        match = re.fullmatch(r"(.+?)\s+(m_\w+)", declaration.strip())
        if match:
            fields[match[2]] = match[1]
    prefix = f"RustBridge_{class_name}"
    context = prefix + "_Context"
    required_fields = set()
    adapters = {}
    references = []
    replacements = []
    parser = Parser(Language(tree_sitter_cpp.language()))

    def params(function):
        if any(not name or re.search(r"[&<>]", typ) for typ, name in function.params):
            raise BridgeError(f"{function.name}: non-C parameter type")
        return [f"{typ} {name}" for typ, name in function.params]

    for function in selected:
        matches = list(
            re.finditer(r"\b" + re.escape(class_name + "::" + function.name) + r"\s*\(", source)
        )
        if len(matches) != 1:
            raise BridgeError(f"{function.name}: source definition is not unique")
        opening = masked.find("{", matches[0].end())
        if opening < 0:
            raise BridgeError(f"{function.name}: no body")
        _, closing = _balanced_block(masked, opening)
        body = source[opening : closing + 1]
        # Parse only the intact body: signature annotations cannot hide its syntax.
        wrapper = ("void bridge_probe() " + body).encode()
        tree = parser.parse(wrapper)
        if tree.root_node.has_error:
            raise BridgeError(f"{function.name}: body has unsupported C++ syntax")
        edits = []
        used = set()
        parameter_types = {name: typ for typ, name in function.params}

        def walk(
            node,
            wrapper=wrapper,
            function=function,
            parameter_types=parameter_types,
            edits=edits,
            used=used,
        ):
            text = wrapper[node.start_byte : node.end_byte].decode()
            if node.type in (
                "new_expression",
                "delete_expression",
                "lambda_expression",
                "throw_statement",
                "try_statement",
                "this",
            ):
                raise BridgeError(
                    f"{function.name}: {node.type} needs an explicit compiler adapter"
                )
            if node.type == "call_expression":
                callee = node.child_by_field_name("function")
                arguments = node.child_by_field_name("arguments")
                if callee is not None and arguments is not None:
                    spelling = wrapper[callee.start_byte : callee.end_byte].decode()
                    owner = None
                    receiver = None
                    method = None
                    if callee.type == "identifier" and spelling in by_name:
                        owner, receiver, method = class_name, "context->object", spelling
                    elif callee.type == "field_expression":
                        obj = callee.child_by_field_name("argument")
                        member = callee.child_by_field_name("field")
                        if obj is None or member is None or obj.type != "identifier":
                            raise BridgeError(f"{function.name}: complex member call {spelling}")
                        obj_name = wrapper[obj.start_byte : obj.end_byte].decode()
                        typ = parameter_types.get(obj_name, "")
                        if not re.fullmatch(r"\w+\s*\*", typ) or "->" not in spelling:
                            raise BridgeError(f"{function.name}: unproven receiver {spelling}")
                        owner = typ.rstrip(" *")
                        receiver = obj_name
                        method = wrapper[member.start_byte : member.end_byte].decode()
                    if method is not None:
                        if receiver is None:
                            raise BridgeError(f"{function.name}: missing adapter receiver")
                        if method not in by_name:
                            raise BridgeError(f"{function.name}: no signature for {method}")
                        adapter = f"{prefix}_call_{owner}_{method}"
                        adapters[adapter] = (owner, by_name[method])
                        edits.append((callee.start_byte, callee.end_byte, adapter))
                        has_args = bool(arguments.named_children)
                        edits.append(
                            (
                                arguments.start_byte + 1,
                                arguments.start_byte + 1,
                                receiver + (", " if has_args else ""),
                            )
                        )
            if node.type == "identifier" and text.startswith("m_") and text not in fields:
                raise BridgeError(f"{function.name}: unresolved state field {text}")
            if node.type == "identifier" and text in fields:
                used.add(text)
                edits.append((node.start_byte, node.end_byte, f"(*context->{text})"))
            for child in node.named_children:
                walk(child)

        walk(tree.root_node)
        lowered = wrapper
        for start, end, replacement in sorted(edits, reverse=True):
            lowered = lowered[:start] + replacement.encode() + lowered[end:]
        lowered_body = lowered[len("void bridge_probe() ") :].decode()
        required_fields.update(used)
        worker = f"{prefix}_{function.name}"
        arguments_text = ", ".join([context + " *context", *params(function)])
        signature = f"{function.return_type} {worker}({arguments_text})"
        references.append(signature + "\n" + lowered_body)
        args = ", ".join(["&context", *(name for _, name in function.params)])
        # Context initialization is finalized once all selected fields are known.
        statement = (
            "" if function.return_type == "void" or function.return_type == "VOID" else "return "
        ) + f"{worker}({args});"
        replacements.append((opening, closing + 1, statement))

    field_order = sorted(required_fields)
    declarations = [f"typedef struct {context} {{", "    void *object;"]
    declarations.extend(f"    {fields[name]} *{name};" for name in field_order)
    declarations.append(f"}} {context};")
    declarations.extend(
        f"{f.return_type} {prefix}_{f.name}({', '.join([context + ' *context', *params(f)])});"
        for f in selected
    )
    adapter_bodies = []
    for name, (owner, function) in sorted(adapters.items()):
        receiver_type = "void *" if owner == class_name else f"{owner} *"
        arguments_text = ", ".join([receiver_type + "object", *params(function)])
        declaration = f"{function.return_type} {name}({arguments_text})"
        declarations.append(declaration + ";")
        args = ", ".join(name for _, name in function.params)
        statement = (
            "" if function.return_type in ("void", "VOID") else "return "
        ) + f"static_cast<{owner} *>(object)->{function.name}({args});"
        adapter_bodies.append('extern "C" ' + declaration + " { " + statement + " }")
    header = (
        "// Generated by rustconv.cpp_bridge; C++ compiler owns class layout.\n#pragma once\n"
        + includes
        + '\n#ifdef __cplusplus\nextern "C" {\n#endif\n'
        + "\n".join(declarations)
        + "\n#ifdef __cplusplus\n}\n#endif\n"
    )
    bridge = source
    initializer = ", ".join(["this", *("&this->" + name for name in field_order)])
    for start, end, statement in sorted(replacements, reverse=True):
        bridge = (
            bridge[:start]
            + "{\n    "
            + context
            + " context = {"
            + initializer
            + "};\n    "
            + statement
            + "\n}"
            + bridge[end:]
        )
    # private.hpp must precede the generated header's SDK includes and declarations.
    include = re.search(r"^\s*#include[^\n]+", bridge, flags=re.M)
    if include is None:
        raise BridgeError("no source include boundary")
    bridge = bridge[: include.end()] + f'\n#include "{header_name}"' + bridge[include.end() :]
    bridge += "\n" + "\n".join(adapter_bodies) + "\n"
    reference = (
        f"// Generated C reference workers from {class_name}; never linked into the driver.\n"
        f'#include "{header_name}"\n#define NDEBUG\n#include <debug.h>\n'
        + macros
        + "\n\n"
        + "\n\n".join(references)
        + "\n"
    )
    return BridgeOutput(
        header,
        reference,
        bridge,
        tuple(prefix + "_" + name for name in methods),
        tuple(f.name for f in candidates if f.name not in methods),
    )


def source_macros(text: str, names: list[str]) -> str:
    """Copy explicit, uniquely defined macros verbatim; ambiguous definitions stay red."""
    lines = text.splitlines(keepends=True)
    guard = re.match(r"\s*#\s*ifndef\s+(\w+)\s*\n\s*#\s*define\s+\1\b", mask_code(text))
    guard_name = guard[1] if guard else None
    stack: list[bool] = []
    definitions = {}
    conditional = set()
    index = 0
    while index < len(lines):
        line = lines[index]
        directive = re.match(r"\s*#\s*(if|ifdef|ifndef|else|elif|endif)\b(.*)", line)
        if directive:
            kind, expression = directive.groups()
            if kind in ("if", "ifdef", "ifndef"):
                stack.append(not stack and kind == "ifndef" and expression.strip() == guard_name)
            elif kind in ("else", "elif") and stack:
                stack[-1] = False
            elif kind == "endif" and stack:
                stack.pop()
        match = re.match(r"\s*#\s*define\s+(\w+)", line)
        index += 1
        if match is None:
            continue
        while line.rstrip().endswith("\\") and index < len(lines):
            line += lines[index]
            index += 1
        definitions.setdefault(match[1], []).append(line)
        if stack and not all(stack):
            conditional.add(match[1])
    result = []
    for name in names:
        candidates = definitions.get(name, [])
        if len(candidates) != 1:
            raise BridgeError(f"macro {name}: expected one definition, found {len(candidates)}")
        if name in conditional:
            raise BridgeError(
                f"macro {name}: conditional definition requires compiler preprocessing"
            )
        result.append(candidates[0])
    return "\n".join(result)


def write_bridge(source_path, scan, args):
    """CLI frontend. Derived C/C++ artifacts are separate from paid Rust output."""
    import hashlib
    import json
    from pathlib import Path

    from .config import REACTOS_ROOT

    if not args.output or not args.bridge_methods or not args.bridge_include:
        raise BridgeError(
            "--compiler-owned requires --output PREFIX, --bridge-methods and --bridge-include"
        )
    if any(not re.fullmatch(r"[\w./-]+", header) for header in args.bridge_include):
        raise BridgeError("bridge include must be a header path")
    macros = ""
    if args.bridge_macro:
        if not args.bridge_macro_header:
            raise BridgeError("--bridge-macro requires --bridge-macro-header")
        macros = source_macros(Path(args.bridge_macro_header).read_text(), args.bridge_macro)
    prefix = Path(args.output)
    header = prefix.with_name(prefix.name + "_bridge.h")
    reference = prefix.with_name(prefix.name + "_reference.c")
    bridge = prefix.with_name(prefix.name + "_bridge.cpp")
    result = generate_bridge(
        source_path.read_text(),
        scan,
        args.class_name,
        args.bridge_methods.split(","),
        header_name=header.name,
        includes="\n".join(f"#include <{h}>" for h in args.bridge_include),
        macros=macros,
    )
    for path, content in (
        (header, result.header),
        (reference, result.reference),
        (bridge, result.bridge),
    ):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def relative(path):
        return str(path.resolve().relative_to(REACTOS_ROOT.resolve()))

    inputs = [source_path, header, reference, bridge]
    if args.bridge_macro_header:
        inputs.append(Path(args.bridge_macro_header))
    manifest = {
        "source": relative(source_path),
        "class": args.class_name,
        "workers": result.workers,
        "retained_cpp": result.retained,
        "retained_free_functions": [f.name for f in scan.functions if f.cpp_class is None],
        "header": relative(header),
        "reference": relative(reference),
        "bridge": relative(bridge),
        "sha256": {
            relative(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs
        },
        "status": (
            "generated; translate recompiles both C and C++ on all three arches before payment"
        ),
    }
    prefix.with_name(prefix.name + "_bridge.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


def validate_reference(reference, *, required=False):
    """Verify provenance and compile both sides of a bridge before a paid C run."""
    import hashlib
    import json
    import shlex
    import subprocess

    from .c2rust_anchor import _decanon, _find_db_entry
    from .config import REACTOS_ROOT

    if not reference.read_bytes().startswith(b"// Generated C reference workers"):
        if required:
            raise BridgeError("expected a generated C reference with compiler provenance")
        return None
    manifest_path = reference.with_name(
        reference.name.removesuffix("_reference.c") + "_bridge.json"
    )
    if not manifest_path.is_file():
        raise BridgeError(f"{reference}: missing compiler-bridge provenance manifest")
    manifest = json.loads(manifest_path.read_text())
    mandatory = {manifest[key] for key in ("source", "header", "reference", "bridge")}
    if not mandatory <= manifest["sha256"].keys():
        raise BridgeError("bridge manifest does not hash every required source/output")
    if (REACTOS_ROOT / manifest["reference"]).resolve() != reference.resolve():
        raise BridgeError("bridge manifest names a different reference file")
    for spelling, digest in manifest["sha256"].items():
        path = (REACTOS_ROOT / spelling).resolve()
        if not path.is_relative_to(REACTOS_ROOT.resolve()):
            raise BridgeError("bridge input belongs to a different worktree")
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            raise BridgeError(
                f"{path}: bridge input/output changed; regenerate and review the bridge"
            )
    bridge = REACTOS_ROOT / manifest["bridge"]
    for arch in ("i386", "amd64", "arm64"):
        for path in (reference, bridge):
            entry = _find_db_entry(path, arch)
            if entry is None:
                raise BridgeError(
                    f"{arch}: no real compilation command for {path.name}; "
                    "configure its CMake oracle target"
                )
            command, directory = entry
            argv = command.get("arguments") or shlex.split(command["command"])
            argv = [_decanon(arg) for arg in argv]
            filtered = [argv[0]]
            index = 1
            while index < len(argv):
                arg = argv[index]
                if arg in ("-o", "-MF", "-MT", "-MQ"):
                    index += 2
                    continue
                if arg not in ("-c", "-MD", "-MMD", "-MP") and arg != str(path.resolve()):
                    filtered.append(arg)
                index += 1
            completed = subprocess.run(
                [*filtered, "-fsyntax-only", str(path.resolve())],
                cwd=directory,
                capture_output=True,
                text=True,
                timeout=120,
            )
            if completed.returncode:
                raise BridgeError(
                    f"{arch}: compiler rejected {path.name}:\n{completed.stderr[-8000:]}"
                )
    return manifest
