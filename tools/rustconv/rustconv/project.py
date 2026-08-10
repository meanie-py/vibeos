"""Find the PROJECT rustconv is operating on, by anchor file rather than by guesswork.

rustconv resolves a lot of things relative to "the project": where the sources are, where
the output crates go, and — the expensive one — where the durable bank of paid API
responses lives. Historically each of those was inferred: from the module's own location
on disk, or by sniffing the current git toplevel for a directory named `ntoskrnl`.

Both inferences break the moment rustconv is not physically inside the tree it is
translating, which is exactly what extracting it as a standalone tool does:

  * module-relative resolution follows the INSTALL, so a pip-installed rustconv reads and
    writes a `.responses` next to the package instead of the one holding your paid work
    (this was live — see response_cache._resolve_responses_dir);
  * a `pip install -e` of one checkout shadows every other worktree, which is a documented
    vibeos footgun: `python -m rustconv` from worktree B silently used worktree A's paths.

So: a project declares itself with a `rustconv.toml` at its root, and everything
project-scoped resolves relative to THAT FILE. Discovery walks up from the current
directory, so the answer depends on where you invoke rustconv — which is what a user
means — and never on where the package happens to be installed.

The file is optional. Without one, callers fall back to the legacy inference, so an older
checkout keeps working unchanged. It is the ANCHOR that is new, not a requirement.

    # rustconv.toml
    [project]
    name      = "vibeos"
    responses = "tools/rustconv/.responses"   # paid artifacts, pinned explicitly
    plugin    = "rustconv.projects.reactos"   # dotted module supplying targets
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path

try:  # 3.11+
    import tomllib  # type: ignore[import-not-found]  # absent on 3.10, hence the fallback
except ModuleNotFoundError:  # 3.10
    import tomli as tomllib

ANCHOR = "rustconv.toml"


class ProjectError(RuntimeError):
    """The anchor exists but does not describe a usable project."""


@dataclass(frozen=True)
class Project:
    """One codebase rustconv translates, as declared by its anchor file."""

    name: str
    root: Path  # the directory holding rustconv.toml
    responses: Path  # durable bank of paid API responses
    plugin: str | None  # dotted module path supplying targets/policy
    anchor: Path  # the rustconv.toml itself — for error messages

    def load_plugin(self):
        """Import the project's plugin module, or None if it declares none."""
        if not self.plugin:
            return None
        from importlib import import_module

        try:
            return import_module(self.plugin)
        except ImportError as error:
            raise ProjectError(
                f"{self.anchor}: plugin {self.plugin!r} is not importable: {error}"
            ) from error


def find_anchor(start: Path | None = None) -> Path | None:
    """The nearest `rustconv.toml` at or above ``start`` (default: CWD).

    Walking UP from the invocation directory is what makes a worktree resolve to ITSELF:
    the answer follows where you are, not where the package was installed from.
    """
    here = (start or Path(os.getcwd())).resolve()
    for candidate in (here, *here.parents):
        anchor = candidate / ANCHOR
        if anchor.is_file():
            return anchor
    return None


def load(start: Path | None = None) -> Project | None:
    """The project anchored at or above ``start``, or None if there is no anchor.

    A MALFORMED anchor is an error, never a silent fallback — if a project went to the
    trouble of declaring itself, quietly ignoring the declaration and inferring something
    else is precisely the silent-wrong-path failure this module exists to remove.
    """
    anchor = find_anchor(start)
    if anchor is None:
        return None

    try:
        raw = tomllib.loads(anchor.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ProjectError(f"{anchor}: unreadable or malformed: {error}") from error

    section = raw.get("project")
    if not isinstance(section, dict):
        raise ProjectError(f"{anchor}: missing a [project] table")

    root = anchor.parent
    name = section.get("name")
    if not name:
        raise ProjectError(f"{anchor}: [project] needs a name")

    def _under_root(value, default: str) -> Path:
        """Resolve a declared path against the ANCHOR's directory.

        Relative-to-the-anchor is the whole point: the same rustconv.toml then means the
        same thing in every worktree and in a fresh clone.
        """
        path = Path(value) if value else Path(default)
        return path if path.is_absolute() else (root / path).resolve()

    return Project(
        name=str(name),
        root=root,
        responses=_under_root(section.get("responses"), ".responses"),
        plugin=section.get("plugin"),
        anchor=anchor,
    )
