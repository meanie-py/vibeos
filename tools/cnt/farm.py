#!/usr/bin/env python3
"""cnt-rust AI farm (test run): per function, feed the *real* ReactOS
source + the source-grounded extraction prompt to Sonnet, capture the
manifest row. Output is diff-harness-verified downstream — this step
just extracts the contract.

Usage:
    python3 tools/cnt/farm.py            # all symbols in SYMBOLS
    python3 tools/cnt/farm.py memchr atol
"""
from __future__ import annotations

import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools/rustconv"))

# Load .env (nothing else does it for us).
for line in (REPO / ".env").read_text().splitlines():
    line = line.strip()
    if line and not line.startswith("#") and "=" in line:
        k, v = line.split("=", 1)
        import os

        os.environ.setdefault(k, v)

from rustconv.config import CLAUDE_MODEL_SONNET  # noqa: E402


def ask_sonnet(system: str, user: str):
    """Direct Anthropic call — capture RAW text. (rustconv's
    submit_single runs a rust-centric extractor that drops non-```rust
    output, which silently ate the TOML for code-leading replies.)"""
    import anthropic

    client = anthropic.Anthropic()
    text, tin, tout = "", 0, 0
    with client.messages.stream(
        model=CLAUDE_MODEL_SONNET,
        max_tokens=2048,
        system=[{"type": "text", "text": system,
                 "cache_control": {"type": "ephemeral"}}],
        messages=[{"role": "user", "content": user}],
    ) as stream:
        for ev in stream:
            t = getattr(ev, "type", "")
            if t == "content_block_delta" and hasattr(ev.delta, "text"):
                text += ev.delta.text
            elif t == "message_start":
                tin = ev.message.usage.input_tokens
            elif t == "message_delta" and hasattr(ev, "usage"):
                tout = ev.usage.output_tokens
    return text, tin, tout

PROMPT = (REPO / "tools/cnt/extract_prompt.md").read_text()
OUT = REPO / "tools/cnt/out"
CRT = REPO / "sdk/lib/crt"

# symbol -> source files to provide as ground truth (delegates included)
SYMBOLS: dict[str, list[str]] = {
    "memchr": ["mem/memchr.c"],
    "memcmp": ["mem/memcmp.c"],
    "memmove": ["mem/memmove.c"],
    "memset": ["mem/memset.c"],
    "atoi64": ["string/atoi64.c"],
    "atol": ["string/atol.c", "string/atoi64.c"],
    "strnlen": ["string/strnlen.c", "string/tcsnlen.h"],
    "strncmp": ["string/strncmp.c", "string/tcsncmp.h"],
    "strchr": ["string/strchr.c", "string/tcschr.h"],
    "strrchr": ["string/strrchr.c", "string/tcsrchr.h"],
    # --- irregular batch 2 (~33): parse-endptr, int→str(+_s), case-
    #     insensitive, multibyte, oddballs. Sources resolved from the
    #     liblibcntpr.a symbol→object map (authoritative).
    "_strtoi64": ["string/strtoi64.c"],
    "_strtoi64_l": ["string/strtoi64.c"],
    "_strtoul_l": ["string/strtoul.c"],
    "strtoul": ["string/strtoul.c"],
    "wcstoul": ["string/wcstoul.c"],
    "_i64toa": ["string/itoa.c"],
    "_i64toa_s": ["string/itoa.c"],
    "_i64tow": ["string/itow.c"],
    "_i64tow_s": ["string/itow.c"],
    "_itoa": ["string/itoa.c"],
    "_itoa_s": ["string/itoa.c"],
    "_itow": ["string/itow.c"],
    "_itow_s": ["string/itow.c"],
    "_ltoa": ["string/itoa.c"],
    "_ltoa_s": ["string/itoa.c"],
    "_ltow": ["string/itow.c"],
    "_ltow_s": ["string/itow.c"],
    "_ui64toa": ["string/itoa.c"],
    "_ui64toa_s": ["string/itoa.c"],
    "_ui64tow": ["string/itow.c"],
    "_ui64tow_s": ["string/itow.c"],
    "_ultoa": ["string/itoa.c"],
    "_ultow": ["string/itow.c"],
    "_stricmp": ["string/_stricmp_nt.c"],
    "_strnicmp": ["string/_strnicmp_nt.c"],
    "_wcsicmp": ["wstring/_wcsicmp_nt.c"],
    "_wcsnicmp": ["wstring/_wcsnicmp_nt.c"],
    "mbstowcs": ["string/mbstowcs_nt.c"],
    "mbtowc": ["string/mbstowcs_nt.c"],
    "wcstombs": ["string/wcstombs_nt.c"],
    "wctomb": ["string/wcstombs_nt.c"],
    "toupper_nt_mb": ["string/toupper_nt_mb.c"],
    "_invalid_parameter": ["stdlib/_invalid_parameter_nt.c"],
}


def gather(files: list[str]) -> str:
    parts = []
    for f in files:
        p = CRT / f
        if p.exists():
            parts.append(f"// ===== sdk/lib/crt/{f} =====\n{p.read_text()}")
    return "\n\n".join(parts)


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    want = sys.argv[1:] or list(SYMBOLS)
    for sym in want:
        src = gather(SYMBOLS[sym])
        user = (
            f"Extract the manifest row for `{sym}`. Provided ReactOS "
            f"source (the ONLY source of truth):\n\n```c\n{src}\n```"
        )
        print(f"[{sym}] {len(src)} src bytes → Sonnet ...", flush=True)
        text, tin, tout = ask_sonnet(PROMPT, user)
        (OUT / f"{sym}.raw.md").write_text(text)
        print(f"  tok in={tin} out={tout}")
        # pull the first ```toml block
        toml = ""
        if "```toml" in text:
            toml = text.split("```toml", 1)[1].split("```", 1)[0].strip()
        (OUT / f"{sym}.toml").write_text(toml + "\n")
        print(f"  wrote out/{sym}.toml ({len(toml)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
