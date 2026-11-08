"""Pre-port argument/signature corpus checked against the shipping native implementation."""

import json
from pathlib import Path

from tools import build_infra_compat as build
from tools import qemu_infra_compat as test

FIXTURES = Path(__file__).with_name("fixtures")


def test_pre_port_argument_corpus():
    cases = json.loads((FIXTURES / "native_arguments.json").read_text())
    with build.DedicatedSession("--api-jsonl") as session:
        for case in cases:
            try:
                result = session.call("cli_arguments_parse", {"argv": case["argv"]})
            except build.NativeError as error:
                assert case.get("code") == 2, (case["argv"], str(error))
            else:
                assert "arguments" in case, (case["argv"], result)
                assert result["arguments"] == case["arguments"], case["argv"]


def test_pre_port_signature_corpus():
    cases = json.loads((FIXTURES / "native_signatures.json").read_text())
    with test.DedicatedSession() as session:
        for case in cases:
            assert (
                session.call("cli_report_boot_signature", {"boot": case["boot"]})
                == case["expected"]
            ), case
