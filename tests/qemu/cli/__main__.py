"""vibeos-cli entry point: `python -m tests.qemu.cli <subcommand>`.

Locates the repo root from this file (not cwd-dependent), dispatches to the
command modules. No default --arch (plan principle #4).
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from . import build as build_mod  # noqa: F401 -- compatibility export for CLI importers
from . import diskspace, maintenance
from . import killvm as kill_mod

# repo root = .../tests/qemu/cli/__main__.py -> up 3
REPO = Path(__file__).resolve().parents[3]


def _ps(args, repo: Path) -> int:
    ps = repo / "qemu" / "ps"
    if not ps.is_file():
        print("qemu/ps not found")
        return 127
    as_json = getattr(args, "json", False)
    cmd = ["python3", str(ps)] + (["--json"] if as_json else [])
    if not as_json:
        rc = subprocess.run(cmd, cwd=repo).returncode
        _print_untracked_windows()
        return rc
    out = subprocess.run(cmd, cwd=repo, capture_output=True, text=True)
    if out.returncode != 0:
        print(out.stdout, end="")
        return out.returncode
    data = json.loads(out.stdout)
    orphans, err = kill_mod.untracked_windows_orphans()
    data["untracked_windows"] = orphans
    if err:
        data["untracked_windows_error"] = err
    print(json.dumps(data, indent=2))
    return 0


def _print_untracked_windows() -> None:
    """qemu/ps reads lockfiles, so it cannot show a qemu.exe whose lock is gone. List those
    from the Windows side: they burn a core each until someone kills them."""
    orphans, err = kill_mod.untracked_windows_orphans()
    if err:
        print(f"untracked Windows QEMUs: UNKNOWN ({err})")
        return
    if not orphans:
        return
    print(f"untracked Windows QEMUs (no lockfile, launcher gone): {len(orphans)}")
    for o in orphans:
        uid = kill_mod.win_uid_of(o)
        print(f"  win_pid={o['win_pid']:<7} {o['arch']:<8} since {o['created']}  {uid}")
    print("  stop them: vibeos-cli kill --stale")


class NativeParser:
    """Namespace compatibility over the shared native argument grammar."""

    def parse_args(self, argv=None, namespace=None):
        import importlib

        from tools import build_infra_compat as native

        try:
            result = native.call(
                "cli_arguments_parse", {"argv": list(sys.argv[1:] if argv is None else argv)}
            )
        except native.NativeError as error:
            print(error, file=sys.stderr)
            raise SystemExit(2) from error
        if result.get("help") is not None:
            print(result["help"], end="")
            raise SystemExit(0)

        def decode(value: Any) -> Any:
            if isinstance(value, dict) and set(value) == {"path"}:
                return Path(value["path"])
            if isinstance(value, dict) and set(value) == {"callable"}:
                module, name = value["callable"].split(":", 1)
                return (
                    globals()[name]
                    if module == "tests.qemu.cli.__main__"
                    else getattr(importlib.import_module(module), name)
                )
            if isinstance(value, list):
                return [decode(v) for v in value]
            if isinstance(value, dict):
                return {k: decode(v) for k, v in value.items()}
            return value

        arguments = decode(result["arguments"])
        if namespace is None:
            return argparse.Namespace(**arguments)
        for key, value in arguments.items():
            setattr(namespace, key, value)
        return namespace

    def print_help(self):
        from tools import build_infra_compat as native

        print(native.call("cli_arguments_schema", {})["help"], end="")


def build_parser() -> NativeParser:
    return NativeParser()


def _cmd_reap(args, repo: Path) -> int:
    """`vibeos-cli reap` -- and `qemu/reap.sh`, which is now a wrapper around it. Action lines
    go to stderr as `[reap] ...` (start.sh passes them through to the user)."""
    from tests.qemu.vm import reap_orphan_qemus

    actions = reap_orphan_qemus(dry_run=bool(getattr(args, "dry_run", False)))
    for line in actions:
        print(f"[reap] {line}", file=sys.stderr)
    failed = [a for a in actions if a.startswith("FAILED")]
    print(f"reap: {len(actions) - len(failed)} action(s), {len(failed)} failed")
    return 1 if failed else 0


def main(argv: list[str] | None = None) -> int:
    # Before anything can take a CPU permit: guarantee a policy EXISTS. It is durable
    # now, so this is a no-op on a healthy box -- it is here for the fresh clone and for
    # the case where the file was lost, which used to mean "silently build uncapped".
    from . import buildpolicy

    buildpolicy.heal()
    args = build_parser().parse_args(argv)
    if args.cmd not in maintenance.UNLOCKED and args.cmd != "retention":
        try:
            with maintenance.activity(REPO):
                return _dispatch(args)
        except maintenance.MaintenanceError as exc:
            # Reuse the storage-admission infrastructure classification: no
            # check ran, so the conductor must not blame a source regression.
            print(f"DISK_ADMISSION_FAIL: maintenance: {exc}", flush=True)
            return 2
    return _dispatch(args)


def _dispatch(args) -> int:
    # The except must stay OUTSIDE the `in REQUESTS` test: a DEFERRED kind takes its own
    # reservation deep inside the command (build, once the CPU broker admits it), and that
    # AdmissionError has to reach the same stdout line and exit 2 as one raised here, or the
    # conductor would classify an infrastructure refusal as a source regression.
    try:
        if args.cmd in diskspace.DEFERRED:
            # Fail fast if the host is hopeless, then let the command claim its real
            # budget once the CPU broker admits it.
            diskspace.refuse_if_hopeless(REPO, args.cmd)
        elif args.cmd in diskspace.REQUESTS:
            with diskspace.reserve(REPO, args.cmd):
                return args.fn(args, REPO)
        return args.fn(args, REPO)
    except diskspace.AdmissionError as exc:
        # stdout is the durable fallback when even summary.json cannot be
        # created. The conductor classifies this as infrastructure, not code.
        print(f"{diskspace.FAILURE}: {exc}", flush=True)
        return 2


def _cli_exit_code(argv: list[str] | None = None) -> int:
    """main(), but argparse's own usage exits come back as a value like any other."""
    try:
        return main(argv)
    except SystemExit as exc:  # argparse usage errors exit(2) from inside parse_args
        code = exc.code
        if code is None:
            return 0
        return code if isinstance(code, int) else 1


def cli_main(argv: list[str] | None = None) -> int:
    """Entry point that puts the exit code in the OUTPUT, not just the status.

    A pipeline reports the exit status of its LAST stage, so `vibeos-cli ... | tail`
    yields 0 no matter how the CLI exited -- agents have three times in one night
    reported a finding manufactured by reading a green pipeline off a red command.
    A rule against piping has not stopped it (it is in the always-loaded push channel,
    and one report quoted the rule while breaking it), so the number is emitted into
    the stream the reader is actually looking at. It survives `| tail`, `| tail -1`
    and a redirect alike, and unlike a not-a-tty warning it has no false positives:
    it is not a warning, it is the missing datum.

    Nothing parses our stdout tail today -- the last-line consumers in gate.py and
    land.py read `bash build.sh`, pytest and `git push`, never a vibeos-cli child.
    But this line is now RESERVED: it is always last, so a consumer that shells out
    to vibeos-cli and wants the salient diagnostic must reverse-SEARCH for it the way
    gate.py:392 finds its pytest summary, never take `.splitlines()[-1]`. Blind-tailing
    a vibeos-cli child gets `exit=N` and looks like the CLI stopped reporting failures.
    (land.py:287 and gate.py:258 do blind-tail, and are safe only because of what they
    happen to read -- copy gate.py:392's idiom instead, not theirs.)

    `main()` stays free of this on purpose: unit tests call it directly, and a plain
    return value is easier to assert than captured stdout. Don't tidy the two together.
    """
    rc = 1
    try:
        rc = _cli_exit_code(argv)
    finally:
        print(f"vibeos-cli: exit={rc}", flush=True)
    return rc


if __name__ == "__main__":
    sys.exit(cli_main())
