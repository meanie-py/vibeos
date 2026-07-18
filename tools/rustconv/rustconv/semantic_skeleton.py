"""Experimental cross-language semantic-skeleton comparison.

This is an observational metric, not an acceptance oracle. It deliberately
compares language-neutral control/mutation shape instead of raw C++/Rust AST
edit distance, which mostly measures syntax choices and FFI ceremony.
"""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path

import tree_sitter_rust
from tree_sitter import Language, Node, Parser

from .callgraph import extract_call_graph
from .candidate_oracle import function_span, load_manifest
from .config import REACTOS_ROOT
from .scanner import CPP_LANG, scan_file

RUST_LANG = Language(tree_sitter_rust.language())

_CONTROL = {
    "if_statement": "if",
    "if_expression": "if",
    "conditional_expression": "if",
    "switch_statement": "switch",
    "match_expression": "switch",
    "for_statement": "loop",
    "for_expression": "loop",
    "while_statement": "loop",
    "while_expression": "loop",
    "do_statement": "loop",
    "loop_expression": "loop",
}
_RETURNS = {"return_statement", "return_expression"}
_JUMPS = {
    "break_statement": "break",
    "break_expression": "break",
    "continue_statement": "continue",
    "continue_expression": "continue",
}
_ASSIGNMENTS = {"assignment_expression", "compound_assignment_expr"}


@dataclass(frozen=True)
class Skeleton:
    controls: tuple[str, ...]
    assignments: int
    dereferences: int
    members: int
    calls: int
    max_control_depth: int
    parse_errors: int


def _walk(node: Node):
    yield node
    for child in node.named_children:
        yield from _walk(child)


def extract_skeleton(source: str, language: Language) -> Skeleton:
    encoded = source.encode("utf-8", "replace")
    root = Parser(language).parse(encoded).root_node
    controls: list[str] = []
    assignments = dereferences = members = calls = max_depth = 0

    def visit(node: Node, control_depth: int = 0) -> None:
        nonlocal assignments, dereferences, members, calls, max_depth
        kind = _CONTROL.get(node.type)
        next_depth = control_depth
        if kind is not None:
            controls.append(f"{kind}:{control_depth}")
            next_depth += 1
            max_depth = max(max_depth, next_depth)
        elif node.type in _RETURNS and control_depth > 0:
            controls.append(f"early_return:{control_depth}")
        elif node.type in _JUMPS:
            controls.append(f"{_JUMPS[node.type]}:{control_depth}")
        if node.type in _ASSIGNMENTS:
            assignments += 1
        elif node.type == "field_expression":
            members += 1
        elif node.type == "call_expression":
            calls += 1
        elif node.type == "unary_expression":
            text = encoded[node.start_byte : node.end_byte].lstrip()
            if text.startswith(b"*"):
                dereferences += 1
        for child in node.named_children:
            visit(child, next_depth)

    visit(root)
    errors = sum(node.type == "ERROR" or node.is_missing for node in _walk(root))
    return Skeleton(
        tuple(controls),
        assignments,
        dereferences,
        members,
        calls,
        max_depth,
        errors,
    )


def _lcs(left: tuple[str, ...], right: tuple[str, ...]) -> int:
    row = [0] * (len(right) + 1)
    for left_item in left:
        prior = 0
        for index, right_item in enumerate(right, 1):
            saved = row[index]
            if left_item == right_item:
                row[index] = prior + 1
            else:
                row[index] = max(row[index], row[index - 1])
            prior = saved
    return row[-1]


def compare_skeletons(c: Skeleton, rust: Skeleton) -> dict:
    total_controls = len(c.controls) + len(rust.controls)
    control_score = (
        1.0 if total_controls == 0 else 2 * _lcs(c.controls, rust.controls) / total_controls
    )
    shape_scores = []
    for field in ("assignments", "dereferences", "members", "max_control_depth"):
        left, right = getattr(c, field), getattr(rust, field)
        shape_scores.append(1.0 - abs(left - right) / max(left, right, 1))
    shape_score = sum(shape_scores) / len(shape_scores)
    score = 0.7 * control_score + 0.3 * shape_score
    return {
        "score": round(score, 4),
        "control_score": round(control_score, 4),
        "shape_score": round(shape_score, 4),
        "control_exact": c.controls == rust.controls,
        "c": asdict(c),
        "rust": asdict(rust),
    }


def run_corpus(manifest: Path) -> dict:
    data = json.loads(manifest.read_text())
    specs, _, _ = load_manifest(manifest)
    by_name = {spec.name: spec for spec in specs}
    results = []
    for case in data.get("cases", []):
        source_path = Path(case["source"])
        if not source_path.is_absolute():
            source_path = REACTOS_ROOT / source_path
        deps = extract_call_graph(scan_file(source_path))
        dep = next((item for item in deps if item.name == case["function"]), None)
        if dep is None:
            raise ValueError(f"{case['id']}: source function not found: {case['function']}")
        spec = by_name[case["id"]]
        rust_file = spec.candidate.read_text()
        start, end = function_span(rust_file, spec.function)
        comparison = compare_skeletons(
            extract_skeleton(dep.body_text, CPP_LANG),
            extract_skeleton(rust_file[start:end], RUST_LANG),
        )
        results.append({"id": case["id"], "band": case["band"], **comparison})
    report = {
        "schema": 1,
        "manifest": str(manifest),
        "status": "experimental-not-an-acceptance-gate",
        "results": results,
    }
    output = manifest.parent / "skeleton-report.json"
    report["output"] = str(output)
    output.write_text(json.dumps(report, indent=2, sort_keys=True))
    return report


def print_report(report: dict, as_json: bool = False) -> None:
    if as_json:
        print(json.dumps(report, indent=2, sort_keys=True))
        return
    results = report["results"]
    exact = sum(item["control_exact"] for item in results)
    print(f"skeleton-check: {exact}/{len(results)} exact control skeletons (experimental)")
    for item in sorted(results, key=lambda value: (value["score"], value["id"])):
        print(
            f"  {item['id']}: score={item['score']:.4f} "
            f"control={item['control_score']:.4f} shape={item['shape_score']:.4f}"
        )
    print(f"  durable report: {report['output']}")
