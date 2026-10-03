"""Bounded host pilot: purchase once, replay audits freely, grade with genuine references.

This experimental C++ entry point does not assemble or certify an OS module. It
records its narrower checks explicitly; COM/PE/guest-ABI gates are not applicable
to this standalone host executable. C compiler-led grading lives in pilot_base64.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import time
from dataclasses import asdict
from pathlib import Path

from .attempts import CHECKER_FINGERPRINT, digest, durable_json, record_validation
from .config import REACTOS_ROOT, RUSTCONV_RESPONSES_DIR
from .cost_ledger import discover_banks
from .fact_seal import SealedFact, SealedSource, fact_contract, seal_facts, unseal_facts
from .lex_mask import mask_code

SOURCE = Path("sdk/tools/txt2nls/main.cpp")
SOURCE_SHA256 = "74053f42959ab949e58ab27587672740f954b2ffba5e3a931223fe8f9d6c1355"
TOKEN = re.compile(r"\b__RUSTCONV_FACT_[0-9]{4}__\b")
SYSTEM = """Translate this bounded C++ host tool to standalone stable Rust 2021 using only std.
Preserve the source's behavior, integer widths, errors, table initialization and exact binary
serialization. Do not redesign or silently fix quirks in the source. Preserve source copyright.
Implement every function; no stubs, extern reference calls, subprocesses, dependencies or includes.
The command interface remains PROGRAM INPUT_TXT OUTPUT_NLS. Do not read any other files.
Return only complete Rust source. Preserve every opaque fact token in executable code or types,
including assertions and bounds; you must not substitute remembered constants. The tokens will
be restored deterministically before compiling. Char/string literals stay faithful to the source.
You may adapt std C++ stream/string/vector APIs to std Rust; calls/lifetimes are verified by
subsequent behavioral grading, not by the fact envelope. This candidate is unverified until graded.
"""


def prepared_source(source: str) -> SealedSource:
    sealed = seal_facts(source, dialect="cpp")
    # Worker sealing assumes declarations were generated already. For a host TU,
    # type declaration names are compiler facts, not opaque expression operands.
    types = set(re.findall(r"\b(?:struct|class|enum)\s+([A-Za-z_]\w*)", mask_code(source)))
    text = sealed.text
    facts = []
    for fact in sealed.facts:
        if fact.original in types:
            text = text.replace(fact.placeholder, fact.original)
        else:
            facts.append(fact)
    return SealedSource(text, tuple(facts))


def prior_output_audit(repo: Path) -> dict:
    banks, absent = discover_banks(repo)
    findings = []
    for bank in banks:
        ledger = bank / "outcomes.jsonl"
        if ledger.is_file():
            for number, line in enumerate(ledger.read_text().splitlines(), 1):
                event = json.loads(line)
                if event.get("source_file") == str(SOURCE) or event.get("target") == "txt2nls":
                    findings.append(f"{ledger}:{number}")
        # Target identity, not generic main.cpp / main symbol, is the bank key here.
        for path in bank.glob("*/meta.json"):
            if json.loads(path.read_text()).get("target") == "txt2nls":
                findings.append(str(path))
        findings.extend(str(p) for p in bank.glob("*txt2nls*") if p.is_dir())
        if (bank / "host-pilots/txt2nls/purchase.json").exists():
            findings.append(str(bank / "host-pilots/txt2nls/purchase.json"))
    shipping = repo / "sdk/lib/rust/txt2nls-rust"
    if shipping.exists():
        findings.append(str(shipping))
    if findings:
        raise ValueError(
            "Prior txt2nls work exists; replay/review, never repurchase:\n" + "\n".join(findings)
        )
    return {"banks": [str(p) for p in banks], "worktrees_without_banks": absent, "findings": []}


def generate(budget: Path, cap: str) -> Path:
    from .batch_runner import submit_single

    source_path = REACTOS_ROOT / SOURCE
    source = source_path.read_text()
    if digest(source) != SOURCE_SHA256:
        raise ValueError(
            "Pilot source changed; review source, compiler manifest and grader before payment"
        )
    audit = prior_output_audit(REACTOS_ROOT)
    directory = RUSTCONV_RESPONSES_DIR / "host-pilots/txt2nls"
    directory.mkdir(parents=True, exist_ok=True)
    # An exclusive durable claim prevents concurrent runs or a crash from repurchasing.
    durable_json(
        directory / "purchase.json",
        {
            "source": str(SOURCE),
            "source_sha256": SOURCE_SHA256,
            "budget": str(budget.resolve()),
            "cap_usd": cap,
            "scope": "host behavioral pilot; not production translation acceptance",
            "prior_output_audit": audit,
        },
    )
    start = time.monotonic()
    compiler = subprocess.run(
        ["g++", "-std=c++17", "-fsyntax-only", str(source_path)],
        capture_output=True,
        text=True,
        check=True,
    )
    sealed = prepared_source(source)
    durable_json(
        directory / "preparation.json",
        {
            "source": source,
            "source_sha256": SOURCE_SHA256,
            "sealed": asdict(sealed),
            "compiler": compiler.args,
            "elapsed_ms": round((time.monotonic() - start) * 1000),
            "checks": {
                "fact_tokens": "strict presence, no by-name rescue",
                "wide_literals": "none",
                "call_seal": "unsupported for standalone std C++ API adaptation",
                "behavior": "unverified; reference grader required",
            },
        },
    )
    os.environ["RUSTCONV_BUDGET_FILE"] = str(budget.resolve())
    os.environ["RUSTCONV_BUDGET_USD"] = cap
    prompt = (
        fact_contract(sealed) + "\n\nSource (entire host tool):\n```cpp\n" + sealed.text + "\n```"
    )
    result = submit_single(prompt, model="deepseek-v4-pro", system_prompt=SYSTEM)
    durable_json(directory / "purchase-result.json", asdict(result))
    if not result.success:
        raise ValueError(
            f"Paid attempt {result.attempt_id} failed; bytes preserved: {result.error}"
        )
    return replay(directory)


def replay(directory: Path) -> Path:
    """Only audit existing bytes: no API imports, fallback or regeneration."""
    purchase = json.loads((directory / "purchase-result.json").read_text())
    prepared = json.loads((directory / "preparation.json").read_text())
    sealed = SealedSource(
        prepared["sealed"]["text"], tuple(SealedFact(**f) for f in prepared["sealed"]["facts"])
    )
    raw = purchase["rust_code"]
    expected = {f.placeholder for f in sealed.facts if not f.dead}
    present = set(TOKEN.findall(mask_code(raw)))
    missing = expected - present
    checker = CHECKER_FINGERPRINT
    if missing:
        record_validation(
            purchase["attempt_id"], checker=checker, verdict="audit_reject", missing=sorted(missing)
        )
        raise ValueError(
            "Fact tokens missing; no paid retry or by-name rescue: " + ", ".join(sorted(missing))
        )
    restored = unseal_facts(raw, sealed, strict=True)
    output = directory / f"candidate-{digest(restored)[:16]}.rs"
    if output.exists():
        if output.read_text() != restored:
            raise ValueError("Candidate artifact changed; refusing overwrite")
    else:
        with output.open("x") as stream:
            stream.write(restored)
    record_validation(
        purchase["attempt_id"],
        checker=checker,
        verdict="fact_audit_pass",
        candidate_sha256=digest(restored),
        scope="not semantic equivalence",
    )
    return output


def retry_empty(directory: Path) -> Path:
    """One explicit non-thinking trial, only if the previous call produced zero code."""
    from .batch_runner import submit_single

    purchase = json.loads((directory / "purchase-result.json").read_text())
    claim = json.loads((directory / "purchase.json").read_text())
    attempt = RUSTCONV_RESPONSES_DIR / "attempts" / purchase["attempt_id"]
    if (
        purchase["success"]
        or purchase["rust_code"].strip()
        or (attempt / "response.partial").stat().st_size
    ):
        raise ValueError("Existing response bytes must be replayed/repaired, never repurchased")
    request = json.loads((attempt / "request.json").read_text())
    if not purchase.get("usage_known"):
        raise ValueError("Prior charge unknown; refusing a second purchase")
    retry = directory / "nonthinking"
    retry.mkdir(exist_ok=False)
    durable_json(
        retry / "purchase.json",
        {
            **claim,
            "previous_attempt_id": purchase["attempt_id"],
            "change": "thinking disabled after zero-code truncation",
        },
    )
    shutil.copyfile(directory / "preparation.json", retry / "preparation.json")
    os.environ["RUSTCONV_BUDGET_FILE"] = claim["budget"]
    os.environ["RUSTCONV_BUDGET_USD"] = claim["cap_usd"]
    os.environ["RUSTCONV_DEEPSEEK_THINKING"] = "disabled"
    result = submit_single(
        request["prompt"], model=request["model"], system_prompt=request["system_prompt"]
    )
    durable_json(retry / "purchase-result.json", asdict(result))
    if not result.success:
        raise ValueError(f"Retry failed; preserved attempt {result.attempt_id}: {result.error}")
    return replay(retry)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    buy = sub.add_parser("generate-txt2nls")
    buy.add_argument("--budget", type=Path, required=True)
    buy.add_argument("--cap-usd", required=True)
    replay_parser = sub.add_parser("replay")
    replay_parser.add_argument("directory", type=Path)
    retry_parser = sub.add_parser("retry-empty")
    retry_parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    if args.command == "retry-empty":
        print(retry_empty(args.directory))
        return
    path = (
        generate(args.budget, args.cap_usd)
        if args.command == "generate-txt2nls"
        else replay(args.directory)
    )
    print(path)


if __name__ == "__main__":
    main()
