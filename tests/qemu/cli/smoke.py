"""CLI compatibility bindings for the native smoke controller."""

from __future__ import annotations

import dataclasses
import os
import sys
from pathlib import Path

from tools import test_infra_compat as native
from tools.test_infra_runner import native_run_call as _native_run_call

from . import archcfg
from . import config as cli_config
from .artifacts import Run, Stage


def _booted_kernel(text: str) -> str | None:
    return native.call("smoke_check_booted_kernel", {"text": text, "env": dict(os.environ)})


def _booted_hal(text: str) -> str | None:
    return native.call("smoke_check_booted_hal", {"text": text, "env": dict(os.environ)})


def _smp_check(text: str, arch: str) -> dict | None:
    return native.call("smoke_check_smp", {"text": text, "arch": arch, "env": dict(os.environ)})


def _mem_mb(size: str) -> int:
    return native.call("smoke_check_mem_mb", {"size": size, "env": dict(os.environ)})


def _payload_vm_lifetime(boot_budget: float, drain: float, environ=os.environ) -> int:
    return native.call(
        "smoke_check_payload_vm_lifetime",
        {"boot_budget": boot_budget, "drain": drain, "env": dict(environ)},
    )


def ahci_requested() -> bool:
    return native.call("smoke_check_ahci_requested", {"env": dict(os.environ)})


def _ahci_check(text: str) -> dict | None:
    return native.call("smoke_check_ahci", {"text": text, "env": dict(os.environ)})


def _ata_health_check(text: str) -> dict | None:
    return native.call("smoke_check_ata_health", {"text": text, "env": dict(os.environ)})


def _kernel_assert_check(text: str) -> dict | None:
    return native.call("smoke_check_kernel_assert", {"text": text, "env": dict(os.environ)})


def _legide_check(text: str) -> dict | None:
    return native.call("smoke_check_legide", {"text": text, "env": dict(os.environ)})


def _ahci_irq_check(stderr_text: str) -> dict | None:
    return native.call(
        "smoke_check_ahci_irq", {"stderr_text": stderr_text, "env": dict(os.environ)}
    )


def _net_suite_env(artifact_dir: Path) -> dict[str, str]:
    return native.call(
        "smoke_check_net_suite_env", {"artifact_dir": str(artifact_dir), "env": dict(os.environ)}
    )


def nvme_default_on(args) -> bool:
    return native.call(
        "smoke_check_nvme_default_on",
        {
            "env": dict(os.environ),
            "nvme": bool(getattr(args, "nvme", False)),
            "arch": getattr(args, "arch", ""),
        },
    )


def nvme_requested() -> bool:
    return native.call("smoke_check_nvme_requested", {"env": dict(os.environ)})


def _headless_check(text: str) -> dict | None:
    return native.call("smoke_check_headless", {"text": text, "env": dict(os.environ)})


def _boot_nvme_check(text: str) -> dict | None:
    return native.call("smoke_check_boot_nvme", {"text": text, "env": dict(os.environ)})


def _xhci_check(text: str) -> dict | None:
    return native.call("smoke_check_xhci", {"text": text, "env": dict(os.environ)})


def _nvme_release_check(text: str) -> dict | None:
    return native.call("smoke_check_nvme_release", {"text": text, "env": dict(os.environ)})


def _smmu_translate_check(text: str) -> dict | None:
    return native.call("smoke_check_smmu_translate", {"text": text, "env": dict(os.environ)})


def _gic_check(text: str) -> dict | None:
    return native.call("smoke_check_gic", {"text": text, "env": dict(os.environ)})


def _nvme_check(text: str) -> dict | None:
    return native.call("smoke_check_nvme", {"text": text, "env": dict(os.environ)})


def _virtio_mmio_check(text: str, arch: str) -> dict | None:
    return native.call(
        "smoke_check_virtio_mmio", {"text": text, "arch": arch, "env": dict(os.environ)}
    )


def _run_kwargs(run, repo):
    return {
        "run_dir": str(run.dir),
        "env": dict(os.environ),
        "python": sys.executable,
        "cli": cli_config.load(repo).data,
        "cfgs": {
            arch: dataclasses.asdict(archcfg.arch_build(arch))
            for arch in ("i386", "amd64", "arm64")
        },
    }


def smoke(args, repo: Path) -> int:
    arch = archcfg.base_arch(args.arch)
    boot_arch = archcfg.vm_arch(args.arch)
    run = Run("smoke", boot_arch if boot_arch == "amd64-bios" else arch)
    return _native_run_call(
        run,
        repo,
        "smoke_run",
        {
            **_run_kwargs(run, repo),
            "args": {key: value for key, value in vars(args).items() if not callable(value)},
        },
    )


def boot(args, repo: Path) -> int:
    arch = archcfg.base_arch(args.arch)
    boot_arch = archcfg.vm_arch(args.arch)
    run = Run("boot", boot_arch if boot_arch == "amd64-bios" else arch)
    return _native_run_call(
        run,
        repo,
        "smoke_boot",
        {
            **_run_kwargs(run, repo),
            "args": {key: value for key, value in vars(args).items() if not callable(value)},
        },
    )


def _net_stage(run: Run, repo: Path, arch: str) -> None:
    _native_run_call(run, repo, "smoke_run_net_stage", {**_run_kwargs(run, repo), "arch": arch})


def _qemu_preflight(repo: Path, boot_arch: str) -> Stage | None:
    dto = _native_run_call(
        None,
        repo,
        "smoke_run_qemu_preflight",
        {
            "run_dir": str(repo),
            "env": dict(os.environ),
            "cli": cli_config.load(repo).data,
            "boot_arch": boot_arch,
        },
    )
    return Stage(**dto) if dto is not None else None


def _payload_build_flags(repo: Path, names: list[str]) -> str:
    return native.call("smoke_run_payload_build_flags", {"names": names}, root=repo)


def _staged_payloads(repo: Path, arch: str) -> list[str]:
    return _native_run_call(None, repo, "smoke_run_staged_payloads", {"arch": arch})


def _guest_reached_desktop(results: list[dict]) -> bool:
    return native.call("smoke_run_guest_reached_desktop", {"results": results})


def _payload_stage(
    run: Run,
    repo: Path,
    arch: str,
    payloads: str,
    idx: int = 0,
    total: int = 1,
    vm_memory_override: str = "",
    boot_arch: str | None = None,
) -> None:
    _native_run_call(
        run,
        repo,
        "smoke_run_payload_stage",
        {
            **_run_kwargs(run, repo),
            "arch": arch,
            "payloads": payloads,
            "idx": idx,
            "total": total,
            "vm_memory_override": vm_memory_override,
            "boot_arch": boot_arch,
        },
    )


def _payload_crash(repo: Path, arch: str, payload_dir: Path) -> dict | None:
    return native.call(
        "smoke_run_payload_crash", {"arch": arch, "payload_dir": str(payload_dir)}, root=repo
    )


def _boot_evidence(run: Run, results: list[dict]) -> None:
    _native_run_call(
        run, native.REPO, "smoke_run_boot_evidence", {"run_dir": str(run.dir), "results": results}
    )


def _one_boot(run: Run, repo: Path, arch: str, **kwargs) -> dict:
    return _native_run_call(
        run, repo, "smoke_one_boot", {**_run_kwargs(run, repo), "arch": arch, **kwargs}
    )
