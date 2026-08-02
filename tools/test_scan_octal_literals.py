"""Tests for scan_octal_literals — the C-octal-literal-read-as-decimal lint.

The must-NOT-fire half is what this lint lives or dies on, and the reason is arithmetic: the
tree has exactly ONE true positive in 3,064 files (the `_fstat` mode bits, already fixed), so
any false-positive rate at all buries it. Two versions of the scanner proved that:

  * the first matched `0[0-7]+` with no value test, and cnt-rust's math tables alone gave 78
    findings — every one of them the digits of a float exponent (`6.93e-01`);
  * the second stripped comments and literals with a sequence of regex passes and reported
    `"0777"` inside a string, in a scanf INPUT CORPUS. Rust spells lifetimes with the same
    character as char literals, so `&'static` opened a quote the regex closed on the next
    tick in the file, and the string that should have been blanked was not.

So each exclusion below is pinned by the shape that produced it, and the fire-cases are
pinned by value rather than by shape — `07` is octal AND decimal seven, and a lint that
reports it is reporting a difference that does not exist.
"""

from __future__ import annotations

import subprocess
import sys
import textwrap
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_octal_literals import scan_text, strip_noise

TOOL = Path(__file__).resolve().parent / "scan_octal_literals.py"


def lits(src: str) -> list[str]:
    return [lit for _, lit, _, _ in scan_text(textwrap.dedent(src))]


# ── fires ────────────────────────────────────────────────────────────────────────────


def test_the_shipped_bug():
    """The literal pair that shipped in msvcrt's `_fstat*` for the life of wave 17."""
    found = scan_text("(*buf).st_mode = (_S_IFREG | 0444) as u16;\n(*buf).st_mode |= 0222;")
    assert [f[1] for f in found] == ["0444", "0222"]
    # The message has to carry BOTH readings or it does not tell you what to change.
    assert found[0][2] == 444 and found[0][3] == 0o444


def test_suffixed_literal_still_fires():
    """A suffix does not change the base, and the typed sites are the ones worth checking."""
    assert lits("let mode: u32 = 0644u32;") == ["0644u32"]


def test_minimum_disagreeing_value():
    """`010` is the smallest literal whose two readings differ: 10 vs 8."""
    assert lits("let x = 010;") == ["010"]


# ── does not fire: the value is the same in both bases ───────────────────────────────


def test_identical_in_both_bases_is_not_a_finding():
    """0, 00, 07, 007 read the same either way. Reporting them trains people to ignore it."""
    assert lits("let a = 0; let b = 00; let c = 07; let d = 007;") == []


# ── does not fire: not actually a leading-zero decimal ───────────────────────────────


def test_float_exponents():
    """The 78-finding false positive. `e-01` / `e+00` digits are not literals."""
    src = """\
        const LN2_HI: f64 = 6.93147180369123816490e-01;
        const INV_LN2: f64 = 1.44269504088896338700e+00;
        const T: f32 = 9.9831822808e-07;
    """
    assert lits(src) == []


def test_other_bases_and_floats():
    assert lits("let a = 0x1FE; let b = 0o444; let c = 0b0101; let d = 0.5;") == []


def test_tuple_index():
    """`x.0` and `x.01` are field access, not numeric literals."""
    assert lits("let a = pair.0; let b = t.0.0;") == []


# ── does not fire: inside a comment or a literal ─────────────────────────────────────


def test_line_comment():
    assert lits("// buf->st_mode = _S_IFREG | 0444;\nlet x = 1;") == []


def test_block_comment_and_nesting():
    """Rust block comments NEST, so a plain search for `*/` closes one too early."""
    assert lits("/* 0444 */ let x = 1;") == []
    assert lits("/* outer /* 0444 */ still comment 0222 */ let x = 1;") == []


def test_string_literals():
    """The regression that forced a real tokenizer: a scanf corpus entry."""
    assert lits('let input = ascii(*r.pick(&["0x1234abcd", "0777", "-42"]));') == []
    assert lits('let id = b"0640\\0".as_ptr();') == []


def test_raw_strings():
    assert lits('let s = r"0444"; let t = r#"a "0444" b"#;') == []


# ── does not fire, and does not SWALLOW the rest of the file either ──────────────────


def test_lifetime_does_not_open_a_quote():
    """`&'static` is a lifetime, not an unterminated char literal.

    The version that got this wrong blanked everything from the tick to the next one, which
    silently removed real code from the scan — the failure mode that reports nothing and
    looks like success.
    """
    src = """\
        fn f(s: &'static str) -> &'a u8 { s }
        let mode = 0444;
    """
    assert lits(src) == ["0444"]


def test_url_in_string_does_not_start_a_comment():
    src = """\
        let url = "http://example.com/x";
        let mode = 0444;
    """
    assert lits(src) == ["0444"]


def test_char_literal_is_still_a_literal():
    assert lits(r"let c = '\''; let mode = 0444;") == ["0444"]


# ── line numbers ─────────────────────────────────────────────────────────────────────


def test_line_numbers_survive_multiline_constructs():
    """Blanking must preserve newlines: a finding on the wrong line sends you to innocent
    code, and the reader concludes the tool is noise."""
    src = '/*\n\n\n*/\nlet s = "a\\\nb";\nlet mode = 0444;\n'
    found = scan_text(src)
    assert [(f[0], f[1]) for f in found] == [(7, "0444")]


def test_strip_noise_preserves_length_per_line():
    src = 'let a = "0444"; // 0222\nlet b = 010;\n'
    out = strip_noise(src)
    assert out.count("\n") == src.count("\n")
    assert "010" in out.split("\n")[1]


# ── the output contract ──────────────────────────────────────────────────────────────


def test_tally_goes_to_stderr_not_stdout():
    """The tally must NOT reach stdout.

    `lints.py` reads stdout as the finding list, so a tally line printed there counts as a
    finding — which is how this scanner's neighbour reported `1 new` on a clean tree.

    Scoped to one crate rather than the whole tree on purpose: "the repo has zero findings"
    is what `vibeos-cli check` asserts against the empty baseline, every run, and doing it
    again here cost 85 seconds of every `check-tooling` for a second opinion on the same
    fact.
    """
    cp = subprocess.run(
        [sys.executable, str(TOOL), "--root", "sdk/lib/rust/msvcrt-crt-rust"],
        capture_output=True,
        text=True,
    )
    assert cp.returncode == 0
    assert cp.stdout.strip() == "", f"unexpected findings:\n{cp.stdout}"
    assert "findings" in cp.stderr
