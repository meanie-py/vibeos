"""Explicit adapter to diagnostic authorities awaiting the CLI migration.

Normal boots never import this module. Launch policy and process ownership remain native.
"""

from __future__ import annotations

import os
from pathlib import Path


def prepare(request: dict) -> dict:
    from tests.qemu.cli import pcsample, rrsample
    from tests.qemu.cli import recordreplay as rr

    os.environ.update(request.get("env", {}))
    argv = list(request["argv"])
    result: dict = {"argv": argv, "sidecars": []}
    req = rr.env_request()
    sampler = rrsample.env_sample()
    if sampler and req is None:
        raise RuntimeError("VIBEOS_QEMU_RR_SAMPLE requires VIBEOS_QEMU_RR")
    if req is not None:
        mode, trace = req
        if request["backend"] != "native":
            raise rr.RecordReplayError("record/replay requires the native backend")
        if mode == "replay":
            problems = rr.check_trace(trace)
            if any(rr.FATAL_MARKER in problem for problem in problems):
                raise rr.RecordReplayError("\n".join(problems))
        argv, media = rr.transform(
            argv,
            mode=mode,
            trace_dir=trace,
            icount_shift=rr.env_shift(),
            stage=mode == "record",
        )
        if mode == "record":
            rr.write_manifest(
                trace,
                arch=request["arch"],
                argv=argv,
                media=media,
                smp=os.environ.get("VIBEOS_QEMU_SMP", "1"),
            )
        else:
            rr.assert_same_shape(trace, argv)
            want = rr.env_break()
            if want is not None:
                result["rr_break_icount"] = rr.snap_break_icount(trace, want)
        rr.note_run(trace, mode=mode, serial_log=request["live_log"])
        result.update(boot_timeout_min=1800.0, hang_silence_min=600.0)
        if sampler:
            if (mode == "replay") != (sampler == "check"):
                raise RuntimeError("RR sampler expects seconds for record or check for replay")
            if mode == "record" and float(sampler) <= 0:
                raise RuntimeError("RR sample period must be positive")
            sock = str(Path(request["sock_dir"]) / "rrsample.sock")
            sample_argv = list(argv)
            argv += rrsample.qmp_args(Path(sock))
            result["sidecars"].append(
                dict(
                    kind="rr",
                    mode=mode,
                    value=sampler,
                    trace=str(trace),
                    argv=sample_argv,
                    sock=sock,
                    work=request["sock_dir"],
                )
            )
    period = pcsample.env_period()
    if period is not None:
        endpoint = {}
        if request["backend"] == "native":
            sock = str(Path(request["sock_dir"]) / "pcsample.sock")
            argv += ["-qmp", f"unix:{sock},server,nowait"]
            endpoint["sock"] = sock
        else:
            from tests.qemu import backend

            reserved = set(request.get("reserved_ports", []))
            for _ in range(64):
                port = backend.windows_free_tcp_port()
                if port not in reserved:
                    break
            else:
                raise RuntimeError("cannot select a unique diagnostic QMP port")
            argv += ["-qmp", f"tcp:0.0.0.0:{port},server=on,wait=off"]
            endpoint.update(host=backend.gateway_ip(), port=port)
        result["sidecars"].append(
            dict(
                kind="pc",
                period=period,
                **endpoint,
                out=str(Path(request["live_log"]).parent / pcsample.SAMPLES_FILE),
            )
        )
    result["argv"] = argv
    return result


def sidecar(request: dict) -> dict:
    """Run one requested diagnostic until its dedicated QMP connection closes."""
    from tests.qemu.cli import pcsample, rrsample
    from tests.qemu.qmp_client import QmpClient

    if request["kind"] == "rr":
        thread = rrsample.start(
            request["mode"],
            request["value"],
            Path(request["trace"]),
            request["argv"],
            Path(request["sock"]),
            Path(request["work"]),
        )
        thread.join()
        return {}
    if request["kind"] == "pc":
        qmp = QmpClient()
        if "sock" in request:
            qmp.connect(Path(request["sock"]), timeout=60.0)
        else:
            qmp.connect_tcp(request["host"], request["port"], timeout=60.0)
        try:
            pcsample.start(qmp, Path(request["out"]), request["period"]).join()
        finally:
            qmp.close()
        return {}
    raise ValueError(f"unknown diagnostic sidecar {request['kind']!r}")
