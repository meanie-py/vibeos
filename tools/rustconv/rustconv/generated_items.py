"""Hand edits made inside a generated prelude block survive the next run.

A stage that rebuilds a `// ==== ... ====` block from scratch used to undo every fix made
inside it: reactos-rust's `GetDiskSizeInBytes` needed C's integer promotion spelled out
(`as ULONGLONG`), and a `port macros` re-run put the unwidened, non-compiling lowering back.

Each generated item now carries a `// gen:<digest>` stamp of its own text. On the next write,
an item whose text no longer matches its stamp, or whose stamp was deleted, is a HAND PORT:
it is moved out of the block, just above it, and the stage's existing "defined outside the
block -> hand-owned" rule keeps the generator off it from then on. Deleting the stamp is how
to claim an item without changing it; deleting the moved-out item takes the generated
version back.

An item written before stamps existed has nothing to check against, so it is compared with
what the generator produces now: a difference is kept as a hand port too. That can also
catch an item the generator merely spells differently today -- conservative, and printed,
so a person can delete the moved copy.
"""

from __future__ import annotations

import hashlib
import re
from collections.abc import Callable
from pathlib import Path

_STAMP = re.compile(r"[ \t]*// gen:([0-9a-f]{12})[ \t]*$", re.M)

HAND_NOTE = (
    "// hand-edited, so kept out of the generated block below (generated_items.py);"
    " delete to regenerate"
)


def _bare(item: str) -> str:
    """The item without its stamp or any whitespace (a reformat is not an edit)."""
    return re.sub(r"\s+", "", _STAMP.sub("", item))


def digest(item: str) -> str:
    return hashlib.sha256(_bare(item).encode()).hexdigest()[:12]


def stamp(item: str) -> str:
    """`item` with `// gen:<digest>` at the end of its first line."""
    item = _STAMP.sub("", item)
    head, nl, rest = item.partition("\n")
    return f"{head} // gen:{digest(item)}{nl}{rest}"


def edited(old: str, new: str | None) -> bool:
    """Whether `old` (an item as it stands in the prelude) was changed by hand."""
    m = _STAMP.search(old)
    if m:
        return m.group(1) != digest(old)
    # Unstamped: written before stamps, or a stamp a person deleted to claim the item. With
    # nothing regenerated to compare against, the generator dropped it -- as before stamps.
    return new is not None and _bare(old) != _bare(new)


def preserve(
    prelude: Path,
    begin: str,
    end: str,
    split: Callable[[str], dict[str, str]],
    new: dict[str, str],
    epilogue: Callable[[str, str], str] = lambda _n, _t: "",
) -> dict[str, str]:
    """Move the block's hand-edited items above it; return `new` without them.

    `split` maps a block body to {name: item text}; `new` is what the stage is about to
    write, stamped. `epilogue(name, item)` is extra text a moved item needs outside the block
    (the SDK block's `pub use crate::NAME;`, which the block otherwise spells for it).
    """
    text = prelude.read_text() if prelude.is_file() else ""
    m = re.search(re.escape(begin) + r"\n?(.*?)" + re.escape(end), text, re.S)
    if not m:
        return new
    hand = {n: t for n, t in split(m.group(1)).items() if edited(t, new.get(n))}
    if not hand:
        return new
    moved = "".join(
        f"{HAND_NOTE}\n{_STAMP.sub('', hand[n]).rstrip()}\n{epilogue(n, hand[n])}"
        for n in sorted(hand)
    )
    body = m.group(1)
    for item in hand.values():  # split returns each item as a verbatim run of the body
        body = body.replace(item + "\n", "", 1)
    block = text[m.start() : m.start(1)] + body + text[m.end(1) : m.end()]
    prelude.write_text(text[: m.start()] + moved + "\n" + block + text[m.end() :])
    print(f"hand edits kept (moved above the generated block): {', '.join(sorted(hand))}")
    return {n: t for n, t in new.items() if n not in hand}
