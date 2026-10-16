"""Read historical path closures without checking out or modifying a lane."""

from __future__ import annotations

import io
import posixpath
import subprocess
import tarfile
from pathlib import Path

from tools.artifact_inputs import Ineligible, tomllib


def capture(repo: Path, revision: str, body: str, destination: Path) -> str:
    """Extract tracked inputs only; generated/dirty historical inputs stay unknown."""

    def git(*args: str) -> bytes:
        return subprocess.check_output(["git", "-C", str(repo), *args], timeout=120)

    commit = git("rev-parse", "--verify", revision + "^{commit}").decode().strip()
    workspace = "sdk/lib/rust"
    packages: set[str] = set()

    def visit(path: str) -> None:
        path = posixpath.normpath(path)
        if not path.startswith(workspace + "/"):
            raise Ineligible("historical dependency leaves Rust workspace")
        if path in packages:
            return
        packages.add(path)
        manifest = tomllib.loads(git("show", f"{commit}:{path}/Cargo.toml").decode())
        for dependency in manifest.get("dependencies", {}).values():
            if isinstance(dependency, dict) and "path" in dependency:
                visit(posixpath.join(path, dependency["path"]))

    visit(posixpath.join(workspace, body))
    paths = [workspace + "/Cargo.toml", workspace + "/Cargo.lock", *sorted(packages)]
    archive = git("archive", "--format=tar", commit, "--", *paths)
    destination.mkdir(parents=True, exist_ok=False)
    with tarfile.open(fileobj=io.BytesIO(archive)) as stream:
        for member in stream:
            relative = Path(member.name)
            if relative.is_absolute() or ".." in relative.parts:
                raise Ineligible("unsafe historical archive path")
            target = destination / relative
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                source = stream.extractfile(member)
                if source is None:
                    raise Ineligible("historical archive member missing")
                target.parent.mkdir(parents=True, exist_ok=True)
                with target.open("xb") as output:
                    output.write(source.read())
            else:
                raise Ineligible("historical source symlinks/special files unsupported")
    return commit
