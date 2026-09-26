"""Explicit front door for the Windows staging cleanup that launches run automatically."""

import subprocess
from pathlib import Path

from ..portlease import launch_lock
from ..windows_staging import cleanup_locked
from . import config
from .artifacts import Run, Stage


def command(args, repo: Path) -> int:
    root = Path(config.load(repo).get("qemu.windows.staging_dir", "/mnt/c/temp/vibeos-qemu"))
    run = Run("windows-cleanup", "all")
    run.extra.update(root=str(root), dry_run=args.dry_run)
    try:
        with launch_lock():
            result = cleanup_locked(root, dry_run=args.dry_run)
        run.extra.update(result)
        action = "eligible" if args.dry_run else "removed"
        run.add(
            Stage(
                "cleanup",
                status="ok",
                detail=f"{len(result[action])} {action}, {len(result['protected'])} protected",
            )
        )
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        run.classification = "CLEANUP_REFUSED"
        run.add(Stage("cleanup", status="fail", detail=str(exc)))
    return run.finalize(repo)
