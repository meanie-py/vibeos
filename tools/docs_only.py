"""Is a set of changed paths INERT — content no build, test or scanner ever executes?

Two places grant a docs-only exemption: the `pre-push` hook (a docs-only delta needs no
fresh gate stamp) and `land`'s lint preflights (a docs-only lane has nothing to lint).
They must agree, so the predicate lives here once and both import it. Stdlib only, no
package imports: the hook runs in a bare clone before anything is installed.

WHY AN EXTENSION ALLOWLIST AND NOT `docs/**`. The old predicate was "under `docs/` OR
ends in `.md`", which is a claim about WHERE a file lives rather than what it is -- and
this tree has seven files under `docs/` that are not documents, including three `.rs`
(`docs/engineering/rtl-threadpool-raw-rustconv/*.raw.rs`) and four `.json`. None is
compiled today, so nothing has gone wrong yet; but "no reader found" is a property of
the tree this afternoon, not of the rule. A scanner that starts reading
`docs/engineering/rustconv-anchor-survey.json` tomorrow would inherit an exemption
nobody re-examined, and the exemption's whole job is to be safe without re-examination.

So: judge the FILE, never the directory. A path qualifies only when its suffix is on
this list, wherever it lives -- which also means a `.md` outside `docs/` (a crate
README) correctly qualifies, where the old rule needed the `*.md` clause bolted on to
catch it.

ONE `.py` OR `.rs` AMONG A HUNDRED `.md` MAKES THE WHOLE DELTA NON-DOCS, even when the
source change is only a comment. Deciding that a comment-only edit is inert means
parsing the language to prove it, and a predicate that can be wrong in the unsafe
direction is worth far less than one that occasionally makes you gate a typo fix.
Gating something inert costs minutes; not gating something live costs a boot.

DELIBERATELY NOT HERE: extensionless files (`LICENSE`, `Makefile`), `.json`, `.toml`,
`.yaml`, `.svg`. Config and data are read by tools, an extensionless file is usually a
script, and SVG is XML that can carry script. Each is arguable and none is worth it --
the cost of omitting a genuinely inert type is one gate, paid by the person adding it.
"""

from __future__ import annotations

# Suffixes whose content is never executed, parsed, linked or scanned by this repo's
# build, tests or source scanners. Lowercase; comparison folds case.
INERT_SUFFIXES = frozenset(
    {
        ".md",
        ".txt",
        ".rst",
        ".png",
        ".jpg",
        ".jpeg",
        ".gif",
    }
)


def is_inert(path: str) -> bool:
    """True when this one path is content nothing ever runs."""
    lowered = path.lower()
    return any(lowered.endswith(suffix) for suffix in INERT_SUFFIXES)


def docs_only(paths) -> bool:
    """True when `paths` is non-empty and EVERY entry is inert.

    Empty is False, not True: "nothing changed" is not a docs-only change, and a caller
    that cannot compute a delta must not be handed an exemption by an empty list.
    """
    paths = [p for p in paths if p and p.strip()]
    return bool(paths) and all(is_inert(p) for p in paths)
