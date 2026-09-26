#!/usr/bin/env python3
"""Replay source-derived repairs on the preserved, paid txt2nls Rust candidate."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path

ORIGINAL_SHA256 = "d4c8ac3a0b93b84e3f3efc9e185a5c03528f255d08265311c3be17150e152b8b"
ORIGINAL_RELATIVE = Path(
    "tools/rustconv/.responses/host-pilots/txt2nls/nonthinking/candidate-d4c8ac3a0b93b84e.rs"
)


def replace_exact(source: str, before: str, after: str, count: int = 1) -> str:
    observed = source.count(before)
    if observed != count:
        raise ValueError(f"repair anchor occurred {observed}, expected {count}: {before[:80]!r}")
    return source.replace(before, after)


def write_immutable(path: Path, body: str) -> None:
    if path.exists():
        if path.read_text(encoding="utf-8") != body:
            raise ValueError(f"existing derivative differs; refusing overwrite: {path}")
        return
    with path.open("x", encoding="utf-8") as stream:
        stream.write(body)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    original = args.repo.resolve() / ORIGINAL_RELATIVE
    data = original.read_bytes()
    observed = hashlib.sha256(data).hexdigest()
    if observed != ORIGINAL_SHA256:
        raise ValueError(f"paid original drifted: {observed}")
    source = data.decode("utf-8")

    # All members are naturally two-byte aligned and the size assertion still
    # verifies 26 bytes. This avoids illegal references to packed u16 fields.
    source = replace_exact(source, "#[repr(C, packed)]", "#[repr(C)]")
    source = replace_exact(
        source,
        "const WHITESPACES: &str =",
        "const MAXIMUM_LEADBYTES: usize = 12;\nconst WHITESPACES: &str =",
    )

    # All 13 parser catches mutate local state; AssertUnwindSafe permits those
    # closures while preserving the paid candidate's control-flow structure.
    source = replace_exact(
        source,
        "std::panic::catch_unwind(|| {",
        "std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {",
        13,
    )
    source = replace_exact(source, "    }) {", "    })) {", 13)
    source = replace_exact(
        source,
        "if dbcs_range_count > (MAXIMUM_LEADBYTES / 2) {",
        "if usize::from(dbcs_range_count) > (MAXIMUM_LEADBYTES / 2) {",
    )
    compile_only = output_dir / "txt2nls-compile-only.rs"
    write_immutable(compile_only, source)

    # C++ std::stol(..., base=0) accepts decimal, 0-prefixed octal and 0x hex.
    # The original generated Rust passed 0 to from_str_radix and would panic on
    # every normal table number. Parse the numeric prefix as strtol does.
    before = """    let val = i64::from_str_radix(&token, base).unwrap_or_else(|_| {
        panic!("invalid integer token");
    });

    let min = i64::from_str_radix(&format!("{}", std::i64::MIN), 10).unwrap();
    let max = i64::from_str_radix(&format!("{}", std::i64::MAX), 10).unwrap();

    if val > max || val < min {
        panic!("{} does not fit range [{}:{}]", token, min, max);
    }
"""
    after = """    let val = parse_stol_prefix(&token, base).unwrap_or_else(|| {
        panic!("invalid integer token");
    });
"""
    source = replace_exact(source, before, after)
    parser = """// Match the consumed numeric prefix of C++ std::stol(token, nullptr, base).
fn parse_stol_prefix(token: &str, base: u32) -> Option<i64> {
    let input = token.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let (negative, unsigned) = match input.as_bytes().first() {
        Some(b'-') => (true, &input[1..]),
        Some(b'+') => (false, &input[1..]),
        _ => (false, input),
    };
    let radix = if base == 0 {
        if unsigned.starts_with("0x") || unsigned.starts_with("0X") { 16 }
        else if unsigned.starts_with('0') { 8 }
        else { 10 }
    } else { base };
    let digits = if radix == 16 && (unsigned.starts_with("0x") || unsigned.starts_with("0X")) {
        &unsigned[2..]
    } else { unsigned };
    let end = digits.find(|c: char| c.to_digit(radix).is_none()).unwrap_or(digits.len());
    if end == 0 {
        // strtol("0x", base 0) consumes the leading zero.
        return if base == 0 && unsigned.starts_with('0') { Some(0) } else { None };
    }
    let magnitude = i128::from_str_radix(&digits[..end], radix).ok()?;
    let signed = if negative { -magnitude } else { magnitude };
    i64::try_from(signed).ok()
}

"""
    source = replace_exact(source, "fn tokenize_int<T>", parser + "fn tokenize_int<T>")

    # C++ promotes these u8 operands to int before subtracting. The generated
    # Rust kept u8 arithmetic, which can overflow in debug builds.
    source = replace_exact(
        source,
        "256 * (range_end - range_start + 1) as usize,",
        "256 * (range_end as usize - range_start as usize + 1),",
    )

    output = output_dir / "txt2nls-repaired.rs"
    write_immutable(output, source)

    # C++ passes token_end (an absolute offset) as substr's *length*. With
    # leading spaces, this consumes extra characters after the token. The
    # first repaired derivative intentionally kept the conventional slice and
    # was independently graded 128/129; preserve it before this final change.
    source = replace_exact(
        source,
        "    *token = line[token_start..token_end].to_string();",
        "    *token = line[token_start..line.len().min(token_start.saturating_add(token_end))]"
        ".to_string();",
    )
    faithful = output_dir / "txt2nls-faithful.rs"
    write_immutable(faithful, source)
    for path in (compile_only, output, faithful):
        print(path, hashlib.sha256(path.read_bytes()).hexdigest())


if __name__ == "__main__":
    main()
