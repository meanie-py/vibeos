#!/usr/bin/env python3
"""vendored — the reader and gate for `vendor.yaml`, the not-our-C registry.

WHY THIS EXISTS
---------------
The project's goal is Rust, so every progress number is a fraction whose denominator
is "C that is ours". Nothing computed that denominator: "vendored by policy" was prose
in `rust-port-status.md`, a good-but-single-tree manifest in `external/README-fex-vendor.md`
(now `vendor-patches/fex.md`), and a 6-entry hardcoded tuple in `verify_conv_macros.py`
that covered 5 of the 19 directories under `sdk/include/reactos/libs/` and carried one
entry (`libs/libxml`) matching nothing at all. Three lists, three contents, no source of truth.

The registry itself is `vendor.yaml`; this module is how code asks it questions.

THE REGISTRY HAS TWO HALVES
---------------------------
`trees:` is the foreign code we do NOT translate. `ours:` is foreign-COPYRIGHT code that
we DO — Microsoft-derived storage class drivers, say, which ReactOS has owned and diverged
for years. Without the second half a "this one is ours" decision has nowhere to live: it
stays prose in a commit message, `--suspects` asks the same question again next month, and
the answer gets re-derived from scratch by whoever looks next. Recording both verdicts is
what turns `--suspects` from a standing question into a RATCHET — `--check` now fails on a
foreign-copyright directory that is in NEITHER list, so a newly-imported tree has to be
classified before it can pass, exactly as a new tree in a container does.

Being in `ours:` is not a claim that the code is ReactOS-authored. It is a claim about
SCOPE: this is code the project intends to own and translate, so it belongs in the
denominator. That is why the entries carry a reason and not just a path.

TWO THINGS THIS DELIBERATELY DOES NOT DO
----------------------------------------
**It does not glob.** `sdk/include/reactos/libs/` holds `zlib` and `mbedtls` next to
`audiosrv`, `samsrv`, `umpnpmgr`, `syssetup`, `fslib`, `fmifs`, `epsapi`, `cportlib`
and `fast486` — all ReactOS's own. No path convention separated foreign code from ours
when this was written (ACPICA under `drivers/`, FreeType under `sdk/lib/3rdparty/`, FEX
under `external/`); the 2026-09 relocation gathered all but the CRT-coupled trees into
`vendor/<lib>`, but the list stays explicit and `--check` refuses an unregistered
directory under a container rather than inferring one.

**It does not store versions.** An entry says WHERE its version lives and `resolve_version`
reads it. A copied literal is a second source of truth that goes stale silently at the
next re-vendor, which is how 78K lines of Microsoft ucrt came to sit in this tree with
no version stamp anywhere. A probe that stops matching is a HARD FAILURE, not a shrug:
it means the file moved or the macro was renamed, and a stale probe reporting a stale
version is worse than reporting nothing.

USAGE
-----
    vendored.py                  # --check: registry agrees with the tree (exit 1 if not)
    vendored.py --emit           # the table: path, version, licence, LOC — both halves
    vendored.py --write-stamps   # (re)generate the VENDORED marker in each tree
    vendored.py --suspects       # foreign-copyright dirs in NEITHER half (i.e. undecided)
    vendored.py --notices [PATH] # generate THIRD-PARTY-NOTICES.md (default: stdout)

THE NOTICES FILE IS GENERATED, AND CHECKED IN ANYWAY
----------------------------------------------------
Distribution needs licence TEXT, not licence names, and a hand-maintained notices file
is one that goes stale the first time a tree is added. So it is derived from the
registry — which already knows every tree, its upstream, its version probe, its extra
licences and where its grant lives. It is *also* committed at the repo root, because a
distributor should not have to run our tooling to get it; `check()` therefore
regenerates and compares, so a checked-in generated file cannot rot.

Two shapes of grant, both handled: a standalone `license_file`, quoted whole, and a
`license_ref` — a source file whose HEADER COMMENT is the only copy of the grant, which
is the case for 8 of the trees (ACPICA, mbedtls, libtiff and friends carry no separate
licence file at all). Where neither exists the tree must carry a `license_note` saying
why, and that note is what the notices reproduce: the two Microsoft drops (WDF, ucrt)
are in that shape, and ucrt's is the honest admission that it carries no grant at all.
An entry that claims a licence it cannot point at is a check failure, because the
alternative is a notices file with a silent hole in it.

Wired into `vibeos-cli check`. Consumed by `verify_conv_macros.py` via `is_vendored`.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

import yaml

REPO = Path(__file__).resolve().parent.parent
REGISTRY = REPO / "vendor.yaml"

#: What counts as source when reporting a tree's size.
_SRC_SUFFIXES = (".c", ".cpp", ".cxx", ".cc", ".h", ".hpp", ".hxx")

#: Translatable BODIES only. The distinction matters for the `ours:` table, whose whole
#: claim is "this much is in the translation denominator": sdk/include/psdk is 247K lines
#: of headers and exactly zero of them are a translation target, so counting it the same
#: way as a driver would report a quarter-million lines of debt that does not exist.
_BODY_SUFFIXES = (".c", ".cpp", ".cxx", ".cc")

#: Written into every registered tree by --write-stamps. The point is that an agent or
#: a human who walks into the directory trips over it without having to know the
#: registry exists — the mistake this guards against (sizing up 78K lines of vendored
#: ucrt as a translation prize) is one made from inside the directory, not from the root.
_STAMP_NAME = "VENDORED"
_STAMP_TEMPLATE = """\
{name} — VENDORED THIRD-PARTY CODE. NOT A TRANSLATION TARGET.

This tree is foreign code carried by the project. It is deliberately NOT part of the
C -> Rust effort and does not count toward translation progress.

  upstream: {upstream}
  licence:  {license}

Registered in vendor.yaml; see that file for the reason and tools/vendored.py for the
checks. Do not translate, and do not "clean up" this tree — local edits make the next
re-vendor a merge instead of a copy.

@generated by tools/vendored.py --write-stamps — do not hand-edit.
"""


class RegistryError(Exception):
    """The registry is malformed. Distinct from 'the registry disagrees with the tree'."""


class Tree:
    """One registered vendored tree."""

    def __init__(self, raw: dict, index: int):
        if not isinstance(raw, dict):
            raise RegistryError(f"trees[{index}] is not a mapping")
        for field in ("path", "name", "reason"):
            if not raw.get(field):
                raise RegistryError(f"trees[{index}] is missing required field `{field}`")
        self.path: str = raw["path"]
        self.name: str = raw["name"]
        self.reason: str = raw["reason"]
        self.upstream: str = raw.get("upstream", "unknown")
        self.license: str = raw.get("license", "unknown")
        self.license_file: str | None = raw.get("license_file")
        #: A file whose HEADER carries the licence grant, for a drop that shipped no
        #: standalone licence file. 15 of the 27 trees are in that shape — ACPICA's full
        #: BSD text and mbedtls's full Apache-2.0 notice live in every source header and
        #: nowhere else — so without this the registry can record a licence NAME it
        #: cannot point at, and a generated notices file would have nothing to quote.
        self.license_ref: str | None = raw.get("license_ref")
        #: Our delta from pristine upstream, as a patch file under vendor-patches/.
        #: Present ONLY where the edit cannot be expressed as a build flag — the
        #: preferred fix is always to make the source pristine and move the change into
        #: the build (libtiff needed no flag at all, libxslt one -D, mbedtls a -D plus
        #: a file of our own). A `patch:` here is an admission that was not possible,
        #: and it says why in its own header.
        self.patch: str | None = raw.get("patch")
        #: Licences present in the tree BESIDES `license`, as {spdx or name: why}. A
        #: single `license:` field on a mixed tree is not a rounding error: wdf is
        #: recorded MIT and carries 11 ReactOS-authored GPL-2.0-or-later files, and FEX
        #: is recorded MIT and carries LGPL-2.1-or-later and GPL-2.0-only. A notices
        #: file built from the headline licence alone would be wrong about all three.
        self.license_extra: dict = dict(raw.get("license_extra") or {})
        #: Why this tree's licence grant CANNOT be quoted from the tree itself. Required
        #: when there is neither a `license_file` nor a `license_ref`, because the
        #: alternative is a notices file with a silent hole in it — and a hole is
        #: indistinguishable from "we checked and it is fine". Both current users are
        #: Microsoft drops that arrived with a copyright line and no grant, which is
        #: precisely the case you want stated in the shipped notices rather than
        #: rounded off. `--notices` prints it verbatim in place of licence text and
        #: `check()` fails if it is missing.
        self.license_note: str | None = raw.get("license_note")
        self.modified: str = str(raw.get("modified", "unknown"))
        self.headers: list[str] = list(raw.get("headers", []))
        self.version: dict | None = raw.get("version")
        #: Free text marking a classification the author was NOT confident about.
        #: Surfaced by --emit so an open question cannot go quiet inside a data file.
        self.review: str | None = raw.get("review")
        #: WE WANT THIS GONE. Distinct from `review`, which is uncertainty about what a
        #: tree IS: this is a settled judgement that the tree should not be here, plus
        #: the exit. A registry that only records what we carry cannot express intent,
        #: so an entry everyone quietly dislikes looks identical to one nobody minds —
        #: and the dislike is exactly the thing that gets lost between sessions. It is
        #: deliberately NOT a check failure: shipping something unwelcome is a decision,
        #: not a defect, and a gate that went red would only teach people to delete the
        #: field.
        self.unwelcome: str | None = raw.get("unwelcome")
        #: How far behind upstream this drop is, as of a DATED audit. Distinct from
        #: `version`, which is probed live from the tree and so is always current about
        #: OURS while saying nothing about theirs. The gap is the security-relevant
        #: number and nothing computed it: the libtiff entry carried "stock 4.1.0 with
        #: no backported fixes" as prose for a day before anyone acted on it, and three
        #: other trees turned out to be further behind than that. Deliberately not
        #: checked against the network by any gate — a lint that needs the internet is
        #: a lint that fails on a train — so it is a dated human finding, and the date
        #: is what tells you whether to re-run the audit.
        self.staleness: str | None = raw.get("staleness")
        #: Whether we mean to OWN this code eventually: `planned` (translate it, and
        #: promote it to `ours:` when the work starts), `keep` (stay vendored and bump
        #: it from upstream), or `undecided`. Not `no`: YAML reads a bare `no` as false.
        #: The owner's call, per tree. A `planned` tree stays OUT of the translation
        #: denominator until someone actually starts, so the progress figure never counts
        #: intent as work. `translate_note` carries
        #: any precondition ("bump to upstream's newest first").
        self.translate: str = str(raw.get("translate", "undecided"))
        self.translate_note: str | None = raw.get("translate_note")

    @property
    def prefixes(self) -> list[str]:
        """Every repo-relative path prefix this tree owns — its source and its headers."""
        return [self.path, *self.headers]


class OurTree:
    """One tree of foreign-COPYRIGHT code that IS ours to translate.

    Carries no version probe and gets no VENDORED stamp: both would be lies. The whole
    payload is the `reason` — a classification with no recorded argument is one the next
    reader has to make again.
    """

    def __init__(self, raw: dict, index: int):
        if not isinstance(raw, dict):
            raise RegistryError(f"ours[{index}] is not a mapping")
        for field in ("name", "reason"):
            if not raw.get(field):
                raise RegistryError(f"ours[{index}] is missing required field `{field}`")
        #: `path` for one directory, `paths` for ONE DECISION spanning many. The second
        #: exists because Wine is a single judgement — "this project translates
        #: Wine-derived code" — that happens to cover 24 shipped modules. Writing it out
        #: 24 times would make one settled argument look like 24 independent findings,
        #: and the payload of an `ours:` entry is the argument. It is NOT a licence to
        #: group unrelated directories: every path under one entry must be covered by
        #: that entry's single reason.
        self.paths: list[str] = [raw["path"]] if raw.get("path") else list(raw.get("paths") or [])
        if not self.paths:
            raise RegistryError(f"ours[{index}] needs `path` or `paths`")
        self.path: str = self.paths[0]
        self.name: str = raw["name"]
        self.reason: str = raw["reason"]
        self.origin: str = raw.get("origin", "unknown")
        #: Where this sits in the roadmap's translation plan, when it has a home there.
        self.tier: str | None = raw.get("tier")

    @property
    def prefixes(self) -> list[str]:
        return list(self.paths)


class Registry:
    def __init__(self, repo: Path, raw: dict):
        self.repo = repo
        if not isinstance(raw, dict):
            raise RegistryError("vendor.yaml is not a mapping")
        self.containers: list[str] = list(raw.get("containers", []))
        self.trees = [Tree(t, i) for i, t in enumerate(raw.get("trees", []))]
        self.ours = [OurTree(o, i) for i, o in enumerate(raw.get("ours", []))]
        if not self.trees:
            raise RegistryError("vendor.yaml registers no trees")
        seen: dict[str, int] = {}
        for i, t in enumerate(self.trees):
            if t.path in seen:
                raise RegistryError(f"duplicate path `{t.path}` (trees[{seen[t.path]}] and [{i}])")
            seen[t.path] = i
        # A path in BOTH halves is not a duplicate to shrug at — it is the registry
        # asserting a thing is and is not a translation target, so every consumer's
        # answer would depend on which list it happens to consult.
        ours_seen: dict[str, int] = {}
        for i, o in enumerate(self.ours):
            if o.path in ours_seen:
                raise RegistryError(
                    f"duplicate path `{o.path}` (ours[{ours_seen[o.path]}] and [{i}])"
                )
            ours_seen[o.path] = i
            if self.is_vendored(o.path):
                raise RegistryError(
                    f"ours[{i}] `{o.path}` is also covered by a `trees:` entry — "
                    f"it cannot be both vendored and a translation target"
                )

    def is_vendored(self, rel: str) -> bool:
        """Is this repo-relative path inside a registered tree?

        Prefix match on a path boundary, so `sdk/lib/ucrtx` is not swallowed by an
        entry for `sdk/lib/ucrt`.
        """
        rel = rel.replace("\\", "/")
        for t in self.trees:
            for p in t.prefixes:
                if rel == p or rel.startswith(p + "/"):
                    return True
        return False

    def tree_for(self, rel: str) -> Tree | None:
        """The registered tree owning this path, if any. Same boundary rule as
        `is_vendored`; the longest matching prefix wins so a nested entry beats
        the container it sits in."""
        rel = rel.replace("\\", "/")
        best: Tree | None = None
        best_len = -1
        for t in self.trees:
            for pfx in t.prefixes:
                if (rel == pfx or rel.startswith(pfx + "/")) and len(pfx) > best_len:
                    best, best_len = t, len(pfx)
        return best

    def is_declared_ours(self, rel: str) -> bool:
        """Is this path inside a tree explicitly classified as OURS to translate?

        Not the negation of `is_vendored`: the overwhelming majority of the repo is ours
        and appears in neither list. This answers the narrower question "was this foreign
        code deliberately kept in the denominator", which is what stops `--suspects` from
        re-asking a settled question.
        """
        rel = rel.replace("\\", "/")
        for o in self.ours:
            for p in o.prefixes:
                if rel == p or rel.startswith(p + "/"):
                    return True
        return False

    def is_classified(self, rel: str) -> bool:
        """Has SOMEBODY decided about this path, either way?"""
        return self.is_vendored(rel) or self.is_declared_ours(rel)

    def loc(self, tree: Tree | OurTree, suffixes: tuple[str, ...] = _SRC_SUFFIXES) -> int:
        """Source lines carried by a tree, for the --emit table only."""
        return sum(self.loc_path(p, suffixes) for p in tree.prefixes[:1])

    def loc_path(self, rel: str, suffixes: tuple[str, ...] = _SRC_SUFFIXES) -> int:
        """Source lines under one repo-relative directory, or in one registered file."""
        root = self.repo / rel
        if root.is_file():
            try:
                return root.read_text(errors="replace").count("\n")
            except OSError:
                return 0
        if not root.is_dir():
            return 0
        total = 0
        for f in root.rglob("*"):
            if f.suffix.lower() in suffixes and f.is_file():
                try:
                    total += f.read_text(errors="replace").count("\n")
                except OSError:
                    pass
        return total

    def resolve_version(self, tree: Tree) -> tuple[str, str | None]:
        """`(version, error)` — derived from the probe, never stored.

        `error` is non-None when a DECLARED probe failed to resolve, which is a check
        failure rather than an unknown: the macro moved or was renamed, and the entry
        is now lying about where to look.
        """
        spec = tree.version
        if not spec:
            return ("unknown", None)
        if "literal" in spec:
            return (str(spec["literal"]), None)
        rel = spec.get("file")
        if not rel:
            return ("unknown", f"{tree.name}: version probe has neither `literal` nor `file`")
        path = self.repo / rel
        if not path.is_file():
            return ("unknown", f"{tree.name}: version probe file is missing: {rel}")
        text = path.read_text(errors="replace")
        pats = spec.get("patterns") or ([spec["pattern"]] if spec.get("pattern") else [])
        if not pats:
            return ("unknown", f"{tree.name}: version probe names no `pattern`/`patterns`")
        parts: list[str] = []
        for p in pats:
            m = re.search(p, text)
            if not m:
                return ("unknown", f"{tree.name}: version probe found no match for /{p}/ in {rel}")
            parts.append(m.group(1))
        return (str(spec.get("join", "")).join(parts), None)


def load(repo: Path = REPO) -> Registry:
    path = repo / "vendor.yaml"
    if not path.is_file():
        # HARD FAIL rather than "nothing is vendored". A missing registry that reads as
        # an empty one would silently move every vendored tree back into the translation
        # denominator and un-exclude 1.7M lines from the scanners that consult it.
        raise RegistryError(f"vendor.yaml not found at {path}")
    return Registry(repo, yaml.safe_load(path.read_text()) or {})


def is_vendored(rel: str, repo: Path = REPO) -> bool:
    """Module-level convenience for scanners; see Registry.is_vendored."""
    return load(repo).is_vendored(rel)


#: Copyright holders that mark a file as almost certainly not ReactOS-authored. Kept to
#: unambiguous corporate holders: an individual's name proves nothing (half of ReactOS is
#: individually copyrighted) and would drown the signal.
#:
#: THE LIST IS THE LIMIT OF THE SCAN, which is easy to forget when it comes back clean.
#: It read "every one is decided" for months while 47,386 lines of SGI's GLU sat
#: unregistered in dll/opengl/glu32 — Silicon Graphics simply was not in it. Silicon
#: Graphics, Hewlett-Packard and Digital Equipment were added 2026-09-19 after GLU was
#: registered, and cost nothing: they flag no further unclassified directory.
#:
#: Wine/CodeWeavers is deliberately ABSENT and should stay absent unless someone is
#: ready to classify the result: a wider sweep found ~989 unclassified Wine-lineage
#: files across ~50 directories (mshtml, msi, jscript, wined3d, urlmon, ole32 …). Those
#: are almost all `ours:` by scope — this project TRANSLATES Wine-derived code — so
#: adding the holder would fail `--check` fleet-wide until fifty `ours:` entries were
#: written. That is a decision, not a scan tweak.
_FOREIGN_HOLDER = re.compile(
    # THE YEARS GO IN THE MIDDLE. This used to be `Copyright \(c\) (Microsoft|...)`,
    # which matches "Copyright (c) Microsoft Corporation" and misses
    # "Copyright (c) 1998-2000 Microsoft Corporation" — the far commoner form, and
    # the reason `--suspects` reported "every one is decided" while 24 files of
    # Microsoft/Intel DDK sample code sat unregistered in drivers/wdm/audio/ac97
    # (found 2026-09-20, during the audit that followed the Solitaire miss).
    r"Copyright\s*(?:\(c\)|\xa9|&copy;)?\s*"
    r"(?:[0-9]{4}\s*(?:[-,/]\s*[0-9]{4})?[\s,]*)*"
    r"(Microsoft|Intel|Apple|Oracle|IBM|Sun Microsystems|Nvidia|Adobe"
    r"|Silicon Graphics|Hewlett-Packard|Digital Equipment)"
    # Wine does not spell it with a `(c)`: its files say
    # `Copyright 2005 Jacek Caban for CodeWeavers`, so the form above cannot see it.
    r"|Copyright[^\n]{0,60}?\bfor (CodeWeavers)"
    r"|Copyright[^\n]{0,60}?\b(The Wine Project)",
    re.I,
)
#: The operative clause of a real licence — the bit that actually grants
#: something. A file that has one needs no further evidence.
_INLINE_GRANT = re.compile(
    r"Redistribution and use"
    r"|Permission is hereby granted"
    r"|Permission to use, copy"
    r"|is free software"
    r"|you (can|may) redistribute"
    r"|Licensed under the"
    r"|SPDX-License-Identifier"
    r"|(placed|released|dedicated).{0,20}public domain"
    r"|GNU (General|Lesser|Library) Public License",
    re.I,
)

#: A `LICENSE:` header naming a licence we recognise. ReactOS's own files carry
#: these instead of an inline grant, which is fine — the name is checkable.
_KNOWN_LICENCE_ID = re.compile(
    r"\b(GPL|LGPL|MIT|BSD|0BSD|Zlib|Apache|ISC|CC0|CC-BY|HPND|Unlicense|X11"
    r"|libpng|IJG|FDLIBM|SGI-B|MPL|Boost|PSF|Public\s+Domain|SPDX)\b",
    re.I,
)

#: A licence declared by POINTING at a file: "See COPYING in the top level
#: directory", "DDK - see license.txt in the root dir". The pointer is only a
#: licence if the file it names is actually in the distribution.
_LICENCE_POINTER = re.compile(
    # "see the file X" / "see file X" is as much a pointer as "see X", and the
    # strict form missed schilytools' whole CDDL notice ("See the file
    # CDDL.Schily.txt in this distribution for details") across 10 files.
    r"\bsee\s+(?:the\s+)?(?:file\s+)?([A-Za-z0-9._-]*(?:COPYING|LICEN[CS]E|LGPL|GPL)[A-Za-z0-9._-]*"
    # ...or a plain text file named outright, which is how libjpeg and the
    # DJGPP-derived CRT bodies point at theirs.
    r"|[A-Za-z0-9._-]+\.txt)",
    re.I,
)

#: Texts the tree SHIPS under a name other than the one its referrers use.
#: ReactOS's own headers have said "See LGPL.txt in the top level directory"
#: since before the file was renamed; the LGPL text is right there as
#: COPYING.LIB, so those 40 references are misspelled, not unfulfilled. An
#: alias is only honoured when its TARGET resolves, so this can excuse a
#: pointer but never invent a grant.
_POINTER_ALIASES = {
    "lgpl.txt": "copying.lib",
    "gpl.txt": "copying",
    # zlib's own contrib/minizip/skipset.h says "See MiniZip_info.txt for the
    # license"; the file beside it has always been MiniZip64_info.txt. Upstream
    # typo, vendored verbatim -- correcting it in place would mean a patch
    # against zlib for one character.
    "minizip_info.txt": "minizip64_info.txt",
}

#: The `LICENSE:` line of a ReactOS-style file header.
_LICENSE_DECL = re.compile(r"LICEN[CS]E:\s*([^\n*]{0,90})", re.I)

#: A file whose NAME is a licence text: COPYING, COPYING.LIB, LICENSE.txt,
#: License, LICENCE.md. Not `license.c`, and not a header that merely mentions one.
_LICENCE_BASENAME = re.compile(r"^(copying|licen[cs]e)([0-9]|[._-][a-z0-9._-]+)?$", re.I)

#: Directories whose licence file governs a whole COLLECTION rather than one
#: component. `modules/rosapps/COPYING` is the GPL, sitting three levels above
#: Ken Kato's Virtual Floppy Drive the same way the repo-root COPYING sits above
#: everything -- it says what the collection is licensed as, not what any
#: particular author granted. Letting it cover a subtree made 33 files of
#: third-party code with no grant of their own vanish from this report.
#: `sdk/include/{psdk,vcruntime}` are here for the same reason from the other
#: direction: each ships ChakraCore's MIT text because four Microsoft SAL
#: headers point at it ("See LICENSE file in the project root"), and that text
#: covers those four files -- not the hundreds of unrelated headers beside them,
#: and emphatically not sdk/include/ndk one level over.
_COLLECTION_ROOTS = {
    "",
    "modules/rosapps",
    "modules/rostests",
    "sdk/include/psdk",
    "sdk/include/vcruntime",
}

#: Does a POINTER name a licence text? The pointer regex deliberately accepts
#: any bare `*.txt`, because libjpeg and the NT4 DDK point at theirs that way --
#: fine for asking "does the named file ship", fatal for asking "is this dir
#: granted". uniata's id_ata.cpp says "see todo.txt", todo.txt is right there,
#: and that one line excused 36,368 lines whose only licence content is a
#: warranty disclaimer. Coverage requires a name that could BE a licence.
_LICENCE_TEXT_NAME = re.compile(
    r"copying|licen[cs]e|gpl|cddl|artistic|apache|mozilla|mpl|bsd\b|zlib|\bmit\b", re.I
)

#: The `COPYRIGHT:` line of a ReactOS-style file header.
_COPYRIGHT_DECL = re.compile(r"COPYRIGHT:\s*([^\n*]{0,90})", re.I)


def _copyright_decl(head: str) -> str:
    m = _COPYRIGHT_DECL.search(head)
    return m.group(1) if m else ""


#: Any copyright line at all, whoever holds it. Broader than _FOREIGN_HOLDER on
#: purpose: the question here is not "is this foreign" but "does anyone claim
#: this, and if so did they say what we may do with it".
_ANY_COPYRIGHT = re.compile(r"copyright\s+(\(c\)|\xa9|[0-9]{4})", re.I)

_SUSPECT_SUFFIXES = (".c", ".cpp", ".cxx", ".cc", ".h", ".hpp", ".hxx")
# `.claude` holds the harness's own subagent worktrees (`.claude/worktrees/<name>`):
# a whole second checkout of the repo, so scanning it reports ANOTHER tree's
# uncatalogued Microsoft headers as findings against THIS one, and the verdict then
# depends on which subagents happen to have run (AGENTS.md fleet-isolation rule 8).
_SUSPECT_SKIP = (".git", ".claude", "build-i386", "build-amd64", "build-arm64")


def _is_nested_checkout(d: str) -> bool:
    """True if `d` is the root of a DIFFERENT git worktree, repo or submodule.

    Agents run worktrees nested inside other worktrees (.claude/worktrees/<lane>),
    so an unguarded walk reads a sibling lane's tree at a sibling lane's commit and
    reports its files as this tree's problem. That is the read-side of the fleet
    isolation law: a verdict that changes with which worktrees happen to exist on
    disk is not a gate. A linked worktree carries a `.git` FILE and a primary
    checkout a `.git` DIRECTORY; either one means "not our files".
    """
    return os.path.exists(os.path.join(d, ".git"))


#: An SPDX tag, or a ReactOS file header's `LICENSE:` line. Both are machine-readable
#: statements by the FILE about itself, which is the only licence evidence that does not
#: require reading prose.
#: The WHOLE expression, not the first token. `([^\s*/]+)` was the first cut and it
#: truncated `Apache-2.0 OR GPL-2.0-or-later` to `Apache-2.0` — which then matched
#: mbedtls's recorded licence and passed, hiding the GPL half of a dual licence in all
#: 79 of its files. A compound expression is exactly the case this check exists for, so
#: it reads to end of line and strips the comment tail.
_SPDX = re.compile(r"SPDX-License-Identifier:[ \t]*([^\n]+)")
_ROS_LICENSE_HDR = re.compile(r"^\s*[*#/]*\s*LICENSE:\s*([^(\n]+)", re.M)


#: Licences a file states by NAME in prose, with no SPDX tag. The blind spot this closes
#: was found the hard way, twice: glu32 carries 73 files under "SGI Free Software License
#: B, Version 1.1" inside a tree recorded SGI-B-2.0, and zlib's contrib/minizip is
#: Info-ZIP inside a tree recorded Zlib. Both sat there while `--licences` printed
#: "every declared licence accounted for".
#:
#: THIS MATCHES LICENCE NAMES, NOT LICENCE TEXT, and the distinction is the whole reason
#: it can exist. Grepping for notice BODIES was rejected when this check was first
#: written, correctly: every ACPICA file reproduces a BSD notice, so a text grep reports
#: 226 hits that are all the recorded licence. A name is a much narrower thing — a file
#: says "GNU General Public License" when that is what governs it.
#:
#: FAMILY GRANULARITY, DELIBERATELY. These map to a family (GPL, LGPL, Apache) rather
#: than a version, because version detection from prose is guesswork — "either version 2
#: of the License, or (at your option) any later version" is three plausible answers —
#: and the drift that matters legally is a COPYLEFT FILE INSIDE A PERMISSIVE TREE, which
#: is a family-level fact. The two named-variant entries are exceptions: SGI-B and
#: Info-ZIP have no SPDX-shaped spelling in the wild, so the name IS the identifier.
#:
#: Order matters. Lesser/Library GPL must be tried before plain GPL or every LGPL notice
#: reports as GPL.
_PROSE_LICENCES: tuple[tuple[re.Pattern[str], str], ...] = (
    (re.compile(r"SGI\s+Free\s+Software\s+License\s+B,?\s*Version\s*1\.1", re.I), "SGI-B-1.1"),
    (re.compile(r"SGI\s+Free\s+Software\s+License\s+B,?\s*Version\s*2\.0", re.I), "SGI-B-2.0"),
    (re.compile(r"\bInfo-ZIP\s+licen[cs]e\b", re.I), "Info-ZIP"),
    (re.compile(r"GNU\s+(?:Lesser|Library)\s+General\s+Public\s+Licen[cs]e", re.I), "LGPL"),
    (re.compile(r"GNU\s+General\s+Public\s+Licen[cs]e", re.I), "GPL"),
    (re.compile(r"Apache\s+Licen[cs]e", re.I), "Apache"),
    (re.compile(r"Mozilla\s+Public\s+Licen[cs]e", re.I), "MPL"),
    (re.compile(r"Common\s+Development\s+and\s+Distribution\s+Licen[cs]e", re.I), "CDDL"),
    (re.compile(r"Boost\s+Software\s+Licen[cs]e", re.I), "BSL"),
    (re.compile(r"Eclipse\s+Public\s+Licen[cs]e", re.I), "EPL"),
)

#: How a registry entry SATISFIES a prose family. Substring matching cannot do this job:
#: "LGPL-2.1" contains "GPL", so a tree recorded LGPL would silently absorb a real GPL
#: file — the exact copyleft-escalation this check is for. The GPL pattern therefore
#: requires a GPL not preceded by an L.
_PROSE_ACCOUNTED: dict[str, re.Pattern[str]] = {
    "SGI-B-1.1": re.compile(r"SGI-B-1\.1", re.I),
    "SGI-B-2.0": re.compile(r"SGI-B-2\.0", re.I),
    "Info-ZIP": re.compile(r"Info-ZIP", re.I),
    "LGPL": re.compile(r"LGPL|Lesser|Library General", re.I),
    "GPL": re.compile(r"(?<![A-Za-z])GPL", re.I),
    "Apache": re.compile(r"Apache", re.I),
    "MPL": re.compile(r"MPL|Mozilla", re.I),
    "CDDL": re.compile(r"CDDL", re.I),
    "BSL": re.compile(r"BSL|Boost", re.I),
    "EPL": re.compile(r"EPL|Eclipse", re.I),
}


def license_drift(reg: Registry) -> list[tuple[str, str, dict]]:
    """Licences a tree's own files declare that its registry entry does not account for.

    The registry's other halves are ratchets — a new vendored dir fails `--check` until
    it is classified. Licences had no such ratchet, and a single `license:` field turns
    out to be wrong on trees nobody suspected: `vendor/wdf` is recorded MIT and
    carries 11 ReactOS-authored `LICENSE: GPL-2.0-or-later` headers; `vendor/fex` is
    recorded MIT and carries LGPL-2.1-or-later and GPL-2.0-only; `sdk/lib/ucrt` is
    recorded as Microsoft's terms and carries 5 files stamped `SPDX MIT`. Generating a
    notices file from the headline licence alone would misstate all three.

    TWO PASSES, and the second one was added on 2026-09-20 to close a blind spot that
    had already cost two findings. An SPDX tag or a ReactOS `LICENSE:` header is a
    machine-readable statement and is read first. Where a file makes none, its header is
    matched against `_PROSE_LICENCES` — a list of licence NAMES, not licence TEXT. That
    distinction is what makes the second pass possible at all: grepping for notice bodies
    was rejected when this was first written, correctly, because every ACPICA file
    reproduces a BSD notice and a text grep drowns in 226 true-but-useless hits. A file
    that NAMES the GNU General Public License is saying something much narrower.

    The prose pass reports a FAMILY (GPL, LGPL, Apache) rather than a version, because
    version detection from prose is guesswork — "either version 2 of the License, or (at
    your option) any later version" admits three answers — while the drift that matters
    legally is a copyleft file inside a permissive tree, which is a family-level fact.

    The first run of it found three unrecorded licences that this function had been
    printing all-clear over: 586 BSL-1.0 files in `vendor/fex` (it vendors range-v3),
    adns's deliberate GPL/LGPL carve-out on its public header, and — the one that
    matters — sixteen LGPL-2.1-or-later files in `dll/3rdparty/libtirpc`, the University
    of Michigan Windows/SSPI port, inside a tree recorded flat BSD-3-Clause.

    A hit is a QUESTION, as everywhere else here: the answer may be "record it in
    `license_extra`" or "those files are ours and the tree boundary is wrong".

    WHAT IS STILL INVISIBLE, stated because a green run reads like an all-clear and is
    not one. Permissive families — MIT, BSD, ISC, Zlib, public-domain dedications — are
    deliberately NOT in `_PROSE_LICENCES`: their notices get reproduced verbatim inside
    files governed by something else (a GPL file quoting the MIT terms of a snippet it
    embeds), so they are exactly where a name match stops being evidence. The omission is
    also the low-risk direction, since the finding this check exists for is copyleft
    turning up where the entry claims permissive, not the reverse. And a file that states
    its terms without ever naming a licence — bespoke prose, "all rights reserved", a
    grant written out longhand — is still invisible to both passes.
    """
    out = []
    for t in reg.trees:
        d = reg.repo / t.path
        if not d.is_dir():
            continue
        found: dict[str, int] = {}
        prose: dict[str, int] = {}
        for p in d.rglob("*"):
            if not p.is_file() or p.suffix.lower() not in _SRC_SUFFIXES:
                continue
            try:
                head = p.read_text(errors="replace")[:3000]
            except OSError:
                continue
            declared = False
            for pat in (_SPDX, _ROS_LICENSE_HDR):
                m = pat.search(head)
                if m:
                    lic = m.group(1).strip().rstrip("*/").strip().rstrip(".").strip()
                    if lic:
                        found[lic] = found.get(lic, 0) + 1
                        declared = True
            # Prose names are the FALLBACK, not a second opinion. A file that already
            # carries an SPDX tag has answered the question authoritatively; reading its
            # prose too would double-report every dual-licensed ACPICA header, once as
            # the tag and once as the family named in the notice below it.
            if declared:
                continue
            for pat, family in _PROSE_LICENCES:
                if pat.search(head):
                    key = f"{family} (prose)"
                    prose[key] = prose.get(key, 0) + 1
                    break
        # Accounted for = it IS the headline licence, or it is recorded in license_extra.
        recorded = t.license + " " + " ".join(t.license_extra)
        unaccounted = {
            lic: n
            for lic, n in found.items()
            if lic.lower() not in t.license.lower() and lic not in t.license_extra
        }
        unaccounted.update(
            {
                lic: n
                for lic, n in prose.items()
                if not _PROSE_ACCOUNTED[lic[: -len(" (prose)")]].search(recorded)
            }
        )
        if unaccounted:
            out.append((t.path, t.license, unaccounted))
    return out


def suspects(reg: Registry, min_files: int = 5) -> list[tuple[str, int]]:
    """Directories full of foreign-copyright source that the registry does NOT cover.

    This is the discovery half, and it exists because the registry's container ratchet
    only watches its containers (vendor/, sdk/lib/3rdparty, dll/3rdparty) — a vendored tree
    that lives anywhere else (ucrt under sdk/lib/, a drop an upstream merge files beside
    its consumer) can only be found by looking. Building the registry by hand turned up
    ~400K lines of Microsoft code nobody had catalogued, WDF at 290K lines the largest; a
    hand-scan that finds that much is a scan worth keeping rather than repeating from
    memory.

    A hit is a QUESTION ("is this ours to translate?") and the answer is a judgement call
    — WDF is Microsoft-authored but it is also a driver framework this OS ships, so it is
    not obviously out of scope the way FreeType is. Auto-excluding on a copyright grep
    would silently shrink the denominator, which is the exact failure this whole registry
    exists to prevent, so the scan still refuses to classify anything itself.

    What it now does is subtract the ANSWERS: a directory drops off this list when it
    lands in `trees:` or in `ours:`, whichever the answer was. That is what lets `check`
    treat a remaining hit as a finding — the list is undecided trees, not foreign ones.
    """
    hits: dict[str, int] = {}
    # os.walk rather than rglob so nested checkouts can be PRUNED instead of
    # visited and filtered: a sibling lane's tree is both wrong to report and
    # expensive to read.
    root = str(reg.repo)
    for dirpath, dirnames, filenames in os.walk(root):
        rel_dir = os.path.relpath(dirpath, root).replace(os.sep, "/")
        if rel_dir == ".":
            rel_dir = ""
            dirnames[:] = [d for d in dirnames if d not in _SUSPECT_SKIP]
        dirnames[:] = [d for d in dirnames if not _is_nested_checkout(os.path.join(dirpath, d))]
        for fn in filenames:
            if os.path.splitext(fn)[1].lower() not in _SUSPECT_SUFFIXES:
                continue
            rel = f"{rel_dir}/{fn}" if rel_dir else fn
            if reg.is_classified(rel):
                continue
            try:
                with open(os.path.join(dirpath, fn), errors="replace") as fh:
                    head = fh.read(4000)
            except OSError:
                continue
            if _FOREIGN_HOLDER.search(head):
                hits[rel_dir] = hits.get(rel_dir, 0) + 1
    return sorted(((d, n) for d, n in hits.items() if n >= min_files), key=lambda x: -x[1])


#: A ReactOS `LICENSE:` header value that names a FLOSS licence. Deliberately loose:
#: the job is not to classify precisely, it is to notice when a file declares terms
#: that are NOT one of these — "DDK", "Proprietary", "See the EULA".
_FLOSS_DECLARATION = re.compile(
    r"GPL|General Public|Lesser|MIT|BSD|Apache|Zlib|zlib|ISC|Public\s*Domain|CC0|CC-BY"
    r"|MPL|Mozilla|Unlicense|WTFPL|X11|Boost|SGI|Info-ZIP|IJG|FDLIBM|STLport"
    # "see COPYING" resolves to the repo-root GPL, which is ReactOS's own licence and
    # is actually there. A pointer to any OTHER file is NOT on this list, and that is
    # deliberate: atapi's header reads "DDK - see license.txt in the root dir", so an
    # allowance for `LICENSE.TXT` silently excused the one file this scan exists to
    # find. A pointer is not a licence — the same rule `--grants` is built on.
    r"|see COPYING|Dual-licensed",
    re.I,
)


def declared_licences(reg: Registry) -> list[tuple[str, str, int, str]]:
    """Files whose own `LICENSE:` header names terms that are not FLOSS.

    THE FIFTH RATCHET, and the one that would have caught `drivers/storage/ide/atapi`
    on day one. That driver ships, is bound by nothing, and carries the Windows NT DDK
    END-USER LICENSE AGREEMENT — indemnify-Microsoft, no-reverse-engineering, "contact
    Microsoft for the applicable royalties due". Not FLOSS, in a tree we publish.

    EVERY OTHER SCAN HERE IS STRUCTURALLY BLIND TO IT, which is the point of adding a
    fifth. `suspects()` and `grant_gaps()` both key on a COPYRIGHT line, and atapi.c
    and atapi.h contain none at all — zero, so no widening of `_FOREIGN_HOLDER` can
    ever reach them. `grant_gaps()` would be satisfied anyway, because the directory
    DOES ship a licence file; the file is just a EULA. And `license_drift()` only walks
    trees already in the registry, which atapi is not.

    What atapi does have is the one machine-readable thing in the file: ReactOS's own
    header convention, `LICENSE: DDK - see license.txt in the root dir`. The parser for
    that already existed (`_ROS_LICENSE_HDR`) and had only ever been pointed at
    registered trees. This points it at everything.

    Scoped tree-wide on purpose: a file that declares non-FLOSS terms matters most
    exactly where the registry is NOT looking. A hit is settled either by deleting the
    code, by correcting a wrong header, or by registering the tree in vendor.yaml with
    the terms recorded — the registry entry is what turns it from a finding into a
    decision someone made.
    """
    registered = tuple(t.path for t in reg.trees) + tuple(o.path for o in reg.ours)
    out: list[tuple[str, str, int, str]] = []
    seen: dict[str, tuple[str, int, str]] = {}
    root = str(reg.repo)
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [
            d
            for d in dirnames
            if not d.startswith(".")
            and not d.startswith("build-")
            and not _is_nested_checkout(os.path.join(dirpath, d))
        ]
        rel_dir = os.path.relpath(dirpath, root).replace(os.sep, "/")
        if rel_dir.startswith(registered):
            continue  # its terms are recorded and reviewed in vendor.yaml
        for fn in filenames:
            if not fn.endswith(_SRC_SUFFIXES):
                continue
            path = os.path.join(dirpath, fn)
            try:
                with open(path, encoding="utf-8", errors="replace") as fh:
                    head = fh.read(2000)
            except OSError:
                continue
            m = _ROS_LICENSE_HDR.search(head)
            if not m:
                continue
            value = m.group(1).strip().rstrip("*/").strip()
            if _FLOSS_DECLARATION.search(value):
                continue
            key = value.lower()
            rel = os.path.relpath(path, root).replace(os.sep, "/")
            if key in seen:
                v, n, ex = seen[key]
                seen[key] = (v, n + 1, ex)
            else:
                seen[key] = (value, 1, rel)
    for value, n, example in seen.values():
        out.append((value, example, n, os.path.dirname(example)))
    return sorted(out, key=lambda r: -r[2])


def grant_gaps(reg: Registry, min_files: int = 3) -> tuple[list[tuple[str, int, str]], dict]:
    """Code that claims a copyright but never says what you may do with it.

    THE FOURTH RATCHET, and it exists because the other three could not see the
    two worst things in this tree. `suspects()` keys on `Copyright (c) <KnownCorp>`
    in a `.c`/`.h`, so it is blind to exactly the shape that bit us:

      * Solitaire (2026-09-20) — J Brown freeware, 1,291 lines. The sources carried
        NO copyright line at all; the grant lived in a stray ReadMe.txt, and 26
        lang/*.rc declared `LICENSE: Freeware...` with no copyright to match. Found
        by grepping master for the author's name AFTER the library it belonged to
        had already been replaced.
      * uniata — 36,368 lines, one individual's copyright, and the BSD warranty
        disclaimer pasted in WITHOUT the "Redistribution and use ... are permitted"
        clause that is the actual licence. A disclaimer is not a grant.

    Two findings, deliberately separate because the fixes differ:

    NO GRANT — a directory the registry does not classify, whose files claim a
    copyright, where nothing in the directory carries an inline grant, names a
    licence we recognise, or points at a licence file that exists. The fix is to
    establish the licence, not to guess it.

    DANGLING POINTER — a file that declares its licence by reference to a text
    that is not in the distribution ("see license.txt in the root dir", when there
    is no such file). Whatever the grant is, a recipient cannot read it, which is
    the one thing nearly every licence requires of a redistributor.

    Returns (no_grant, dangling) so callers can report them apart.
    """
    root = str(reg.repo)

    # Which licence-ish basenames exist in WHICH directory. Resolution is then
    # against the referring file's own directory and its ancestors up to the repo
    # root, which is what "in the top level directory" and "in the root dir"
    # actually mean. Matching a basename ANYWHERE was the first version and it was
    # wrong: atapi's "see license.txt in the root dir" was satisfied by an
    # unrelated base/shell/cmd/license.txt, hiding the very finding that motivated
    # this scan.
    by_dir: dict[str, set[str]] = {}
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in _SUSPECT_SKIP]
        dirnames[:] = [d for d in dirnames if not _is_nested_checkout(os.path.join(dirpath, d))]
        rd = os.path.relpath(dirpath, root).replace(os.sep, "/")
        by_dir[("" if rd == "." else rd)] = {f.lower() for f in filenames}

    def _resolves(name: str, from_dir: str) -> bool:
        """Is `name` reachable from `from_dir` by walking up to the repo root?"""
        name = name.lower()
        # A reference WITHOUT an extension may be satisfied by any extension --
        # "see LICENSE" where the file is LICENSE.TXT is a naming quibble. One
        # WITH an extension must match exactly: COPYING.DJ is the DJGPP licence
        # and COPYING is the GPL, so letting the first resolve to the second
        # would hide a genuinely absent grant.
        loose = "." not in name
        d = from_dir
        alias = _POINTER_ALIASES.get(name)
        if alias and _resolves(alias, from_dir):
            return True
        while True:
            here = by_dir.get(d, ())
            if name in here or (loose and any(f.startswith(name + ".") for f in here)):
                return True
            if not d:
                return False
            d = d.rsplit("/", 1)[0] if "/" in d else ""

    def _component_licence(from_dir: str) -> str:
        """The nearest NON-ROOT ancestor directory shipping a licence text."""
        d = from_dir
        while d:
            if d not in _COLLECTION_ROOTS and any(
                _LICENCE_BASENAME.match(f) for f in by_dir.get(d, ())
            ):
                return d
            d = d.rsplit("/", 1)[0] if "/" in d else ""
        return ""

    per_dir: dict[str, dict] = {}
    dangling: dict[str, dict] = {}

    for dirpath, dirnames, filenames in os.walk(root):
        rel_dir = os.path.relpath(dirpath, root).replace(os.sep, "/")
        if rel_dir == ".":
            rel_dir = ""
            dirnames[:] = [d for d in dirnames if d not in _SUSPECT_SKIP]
        dirnames[:] = [d for d in dirnames if not _is_nested_checkout(os.path.join(dirpath, d))]
        for fn_ in filenames:
            if os.path.splitext(fn_)[1].lower() not in _SUSPECT_SUFFIXES:
                continue
            rel = f"{rel_dir}/{fn_}" if rel_dir else fn_
            try:
                with open(os.path.join(dirpath, fn_), errors="replace") as fh:
                    head = fh.read(4000)
            except OSError:
                continue

            # A pointer to a licence text that is not here is a finding wherever it
            # sits — including inside a registered tree, since registration records
            # what the licence IS and this is about whether it SHIPS.
            for m in _LICENCE_POINTER.finditer(head):
                # "see the file COPYING. If not, write to ..." -- the FSF's own
                # boilerplate ends the sentence, and the greedy name class ate
                # the full stop, turning a resolvable COPYING into a dangling
                # `copying.` across 40 files. No licence file ends in a dot.
                name = m.group(1).rstrip(".")
                # A pointer has to look like a FILE. Bare prose ("see license
                # terms") captured the word `license` 80 times before this, all
                # of them noise; a real reference either carries an extension or
                # uses the ALL-CAPS COPYING/LICENSE convention.
                if "." not in name and not name.isupper():
                    continue
                name = name.lower()
                if _resolves(name, rel_dir):
                    continue
                # A vendored tree's licence text often does not sit beside the
                # copy of its headers we install elsewhere: libpng's png.h says
                # "see LICENSE" from sdk/include/reactos/libs/libpng, while the
                # text ships with the sources in dll/3rdparty/libpng/docs. The
                # registry already records that path, and the question this
                # check asks -- can a recipient read the terms -- is answered
                # yes by a `license_file` that exists.
                owner = reg.tree_for(rel)
                if owner and owner.license_file and (reg.repo / owner.license_file).is_file():
                    continue
                d = dangling.setdefault(name, {"count": 0, "example": rel})
                d["count"] += 1

            if reg.is_classified(rel):
                continue
            st = per_dir.setdefault(rel_dir, {"claimed": 0, "covered": False, "example": rel})
            if _ANY_COPYRIGHT.search(head):
                st["claimed"] += 1
            if _INLINE_GRANT.search(head):
                st["covered"] = True
                continue
            decl = _LICENSE_DECL.search(head)
            if decl and _KNOWN_LICENCE_ID.search(decl.group(1)):
                st["covered"] = True
                continue
            # The pointer may sit on EITHER header line. ReactOS's own files
            # conventionally put it on COPYRIGHT: ("COPYRIGHT: See COPYING in
            # the top level directory") and say nothing under LICENSE: at all,
            # which is how hdaudbus read as ungranted while its grant was the
            # GPL sitting at the repo root exactly where it pointed.
            # ...and it may not sit on a header line at all: schilytools spells
            # its CDDL notice as free prose ("See the file CDDL.Schily.txt in
            # this distribution for details"), with no ReactOS-style header to
            # hang it on. A pointer that RESOLVES is a grant wherever it sits;
            # the whole point of the check is whether a recipient can read the
            # terms, not which line told them where.
            for m in _LICENCE_POINTER.finditer(head):
                nm = m.group(1).rstrip(".")
                if not _LICENCE_TEXT_NAME.search(nm):
                    continue
                if ("." in nm or nm.isupper()) and _resolves(nm, rel_dir):
                    st["covered"] = True
                    break
            if st["covered"]:
                continue
            # A licence text placed at a NON-ROOT ancestor covers the subtree
            # under it: someone put it there for this component, which is what
            # ncftp's LICENSE.txt (the Clarified Artistic License, one level up
            # from libncftp) is. The repo root is excluded on purpose -- letting
            # the project's own COPYING cover any directory would excuse every
            # piece of foreign code in the tree, which is the whole finding.
            if _component_licence(rel_dir):
                st["covered"] = True

    no_grant = sorted(
        (
            (d, st["claimed"], st["example"])
            for d, st in per_dir.items()
            if st["claimed"] >= min_files and not st["covered"]
        ),
        key=lambda x: -x[1],
    )
    return no_grant, dangling


#: Grant findings this tree knowingly carries, and why. The key is `dir:<path>`
#: for a directory with no grant and `ptr:<name>` for a licence text that is
#: pointed at but not shipped. Both of these want a DECISION about licensing,
#: not a change to the scanner -- which is exactly why they are written down
#: here instead of being tuned out of `grant_gaps`.
_GRANTS_ACCEPTED = {
    "ptr:copying.bsd": (
        "sdk/tools/kbdtool, 5 files: `LICENSE: BSD - See COPYING.BSD in the top "
        "level directory`. ReactOS has never shipped a COPYING.BSD -- it is absent "
        "from our history and from upstream master -- and no file in this tree "
        "carries a ReactOS-authored BSD grant to copy the wording from, so the "
        "variant is unknown. Writing one on the ReactOS Foundation's behalf would "
        "be inventing a grant, which is the thing this scan exists to catch."
    ),
}

#: The generated notices document, at the repo root where a distributor looks for it.
NOTICES_PATH = "THIRD-PARTY-NOTICES.md"

#: The generated Rust-crate half of the registry. Committed, and gated for freshness
#: against Cargo.lock the same way THIRD-PARTY-NOTICES.md is gated against vendor.yaml.
CRATES_PATH = "tools/rust-crates.yaml"

#: sdk/lib/rust/Cargo.lock — the authority on WHICH external crates exist at WHICH
#: version. Licences and the shipped/host split need `cargo metadata`, which needs a
#: populated registry cache; the lockfile needs nothing. So the snapshot carries the
#: licences and `check` verifies only the (name, version) set, which is the drift that
#: actually matters and the only part that can be checked without cargo.
CARGO_LOCK = "sdk/lib/rust/Cargo.lock"

#: Workspace members that exist to build or test the tree, never to ship in it.
#: Anything reachable ONLY from these is host-only. Kept short and explicit because
#: it cannot be derived: see crates_snapshot() for why cargo's graph cannot answer
#: this question here.
_HOST_ONLY_MEMBERS = (
    "reactos-link",  # the cargo->PE link helper; pulls `cc` as a normal dependency
    "slist-loom",  # a loom model-checking harness; pulls the whole tracing stack
)


def cargo_lock_crates(repo: Path) -> dict[tuple[str, str], None]:
    """External (name, version) pairs from Cargo.lock. No cargo invocation."""
    text = (repo / CARGO_LOCK).read_text(encoding="utf-8", errors="replace")
    out: dict[tuple[str, str], None] = {}
    for block in text.split("[[package]]")[1:]:
        name = re.search(r'^name = "(.+)"', block, re.M)
        ver = re.search(r'^version = "(.+)"', block, re.M)
        src = re.search(r'^source = "(.+)"', block, re.M)
        if name and ver and src:  # `source` present == not one of ours
            out[(name.group(1), ver.group(1))] = None
    return out


def crates_snapshot(repo: Path) -> str:
    """Regenerate the crate snapshot from `cargo metadata`. Needs cargo and the cache.

    WHY A SNAPSHOT AND NOT A LIVE QUERY: `vibeos-cli check` must work without a
    populated ~/.cargo registry, and licences only come from cargo metadata. So the
    licences are frozen here and `check` verifies the cheap half — that the set of
    (name, version) still matches Cargo.lock.

    THE SHIPPED/HOST SPLIT IS BEST-EFFORT AND SAYS SO. Cargo's dependency graph cannot
    answer it in this tree: `netstack-cargo` does not depend on `netstack-rust` in
    cargo terms at all — it pulls only panic-common and the link helper, and CMake
    links the real crate in afterwards. So walking edges from the 34 crates that
    `add_cargo_module` builds reaches zero external crates, which is obviously wrong.
    What CAN be established mechanically is the negative: a proc-macro crate runs in
    the compiler and never ships, and a crate reachable only from a `-difftest`
    harness or from _HOST_ONLY_MEMBERS is host-only too. Everything else is listed as
    shipping, which is the safe direction for a notices file — over-listing costs a
    line, under-listing is the thing that matters.
    """
    meta = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--offline"],
            cwd=repo / "sdk/lib/rust",
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    )
    pkgs = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    members = set(meta["workspace_members"])

    def nm(i: str) -> str:
        return pkgs[i]["name"]

    proc_macro = {
        i for i in pkgs if any(k == "proc-macro" for t in pkgs[i]["targets"] for k in t["kind"])
    }
    roots = [
        i for i in members if not nm(i).endswith("-difftest") and nm(i) not in _HOST_ONLY_MEMBERS
    ]
    seen: set[str] = set()
    stack = list(roots)
    while stack:
        cur = stack.pop()
        if cur in seen:
            continue
        seen.add(cur)
        if cur in proc_macro or nm(cur) in _HOST_ONLY_MEMBERS:
            # A proc-macro's dependencies are the compiler's, not the product's; a
            # host-only member's are the build's. Skipping them as ROOTS is not enough
            # — reactos-link is also a plain dependency of other members, and
            # descending through it dragged cc, shlex and find-msvc-tools in.
            continue
        for d in nodes[cur]["deps"]:
            if any(k.get("kind") is None for k in d["dep_kinds"]):
                stack.append(d["pkg"])

    rows = []
    for i, p in pkgs.items():
        if not p.get("source"):
            continue
        ships = i in seen and i not in proc_macro
        why = "" if ships else ("proc-macro" if i in proc_macro else "build/test only")
        rows.append((p["name"], p["version"], p.get("license") or "SEE LICENSE FILE", ships, why))
    rows.sort(key=lambda r: (r[0], r[1]))

    out = [
        "# GENERATED by tools/vendored.py --write-crates. Do not hand-edit.",
        "#",
        "# The Rust half of the vendor registry: every crates.io dependency that enters",
        "# this tree, with the licence cargo reports for it. vendor.yaml covers vendored",
        "# C; this covers what `cargo` brings in, which nothing tracked until 2026-09-20",
        "# — smoltcp, the TCP/IP stack, among them.",
        "#",
        "# `ships:` is BEST-EFFORT and errs toward true. Cargo's graph cannot settle it",
        "# here (see crates_snapshot() in tools/vendored.py): a crate is marked false only",
        "# when it is provably host-only — a proc-macro, or reachable only from a",
        "# -difftest harness or a build helper. Treat false as certain and true as",
        "# 'assume it ships'.",
        "#",
        f"# Regenerate with: tools/vendored.py --write-crates   ({len(rows)} crates)",
        "crates:",
    ]
    for name, ver, lic, ships, why in rows:
        out.append(f"  - name: {name}")
        out.append(f"    version: {ver}")
        out.append(f"    license: {lic!r}" if "'" in lic or '"' in lic else f"    license: {lic}")
        out.append(f"    ships: {'true' if ships else 'false'}")
        if why:
            out.append(f"    host_only_because: {why}")
    return "\n".join(out) + "\n"


def crates_rows(repo: Path) -> list[tuple[str, str, str, bool]]:
    """(name, version, licence, ships) from the committed snapshot."""
    path = repo / CRATES_PATH
    if not path.exists():
        return []
    rows, cur = [], {}
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line.startswith("- name:"):
            if cur:
                rows.append(cur)
            cur = {"name": line.split(":", 1)[1].strip()}
        elif ":" in line and cur and not line.startswith("#"):
            k, v = line.split(":", 1)
            cur[k.strip()] = v.strip()
    if cur:
        rows.append(cur)
    return [
        (
            r["name"],
            r.get("version", "?"),
            r.get("license", "?").strip("'\""),
            r.get("ships") == "true",
        )
        for r in rows
    ]


#: The values `translate:` may take; see Tree.translate.
TRANSLATE_VALUES = ("planned", "keep", "undecided")


def check(reg: Registry) -> list[str]:
    """Every way the registry can disagree with the tree. Returns problem strings."""
    problems: list[str] = []
    registered = {t.path for t in reg.trees}

    for t in reg.trees:
        if not (reg.repo / t.path).exists():
            problems.append(f"{t.name}: registered path does not exist: {t.path}")
        if t.translate not in TRANSLATE_VALUES:
            problems.append(
                f"{t.name}: translate: {t.translate!r} is not one of {', '.join(TRANSLATE_VALUES)}"
            )
        if t.license_file and not (reg.repo / t.license_file).is_file():
            problems.append(f"{t.name}: license_file does not exist: {t.license_file}")
        if t.license_ref and not (reg.repo / t.license_ref).is_file():
            problems.append(f"{t.name}: license_ref does not exist: {t.license_ref}")
        # A recorded licence NAME we cannot point at is the gap that makes a generated
        # notices file quietly wrong. Either the grant is quotable from the tree, or we
        # say in writing why it is not. Scoped to trees that actually CLAIM a licence:
        # for one still recorded `unknown` the honest complaint is the unknown licence,
        # not a missing copy of it, and demanding the text first would just invite a
        # placeholder. Every tree in the committed registry names a licence.
        if t.license != "unknown" and not (t.license_file or t.license_ref or t.license_note):
            problems.append(
                f"{t.name}: records licence `{t.license}` but has no `license_file`, no "
                f"`license_ref` and no `license_note` — nothing for --notices to quote. "
                f"Point at the grant, or state in `license_note` why the tree does not "
                f"carry one."
            )
        if t.patch:
            pf = reg.repo / t.patch
            if not pf.is_file():
                problems.append(f"{t.name}: patch does not exist: {t.patch}")
            elif not pf.read_text(errors="replace").strip():
                problems.append(f"{t.name}: patch is empty: {t.patch}")
        for h in t.headers:
            if not (reg.repo / h).exists():
                problems.append(f"{t.name}: header path does not exist: {h}")
        _, err = reg.resolve_version(t)
        if err:
            problems.append(err)
        # The stamp is a generated artifact, so it gets a drift check like every other
        # one in this repo. A stamp naming the wrong upstream is worse than none: it is
        # the copy a reader trusts because it is sitting right there in the directory.
        #
        # A SINGLE-FILE tree gets no stamp: there is no directory to put one in, and a
        # stamp beside it in the parent would claim the parent's other files. Upstream
        # ReactOS vendors a few loose files (setup's treelist.c, the libdxg header) and
        # they still need registering; this entry is their only marker until they move.
        if (reg.repo / t.path).is_file():
            continue
        stamp = reg.repo / t.path / _STAMP_NAME
        want = _STAMP_TEMPLATE.format(name=t.name, upstream=t.upstream, license=t.license)
        if not stamp.is_file():
            problems.append(f"{t.name}: missing {t.path}/{_STAMP_NAME} — run --write-stamps")
        elif stamp.read_text() != want:
            problems.append(f"{t.name}: {t.path}/{_STAMP_NAME} is STALE — run --write-stamps")

    # The ratchet: a new library dropped into a container must be registered before it
    # can pass. Without this the registry is a snapshot that silently stops being true.
    for c in reg.containers:
        cdir = reg.repo / c
        if not cdir.is_dir():
            problems.append(f"container does not exist: {c}")
            continue
        for child in sorted(p for p in cdir.iterdir() if p.is_dir()):
            rel = child.relative_to(reg.repo).as_posix()
            if rel not in registered:
                problems.append(
                    f"UNREGISTERED vendored tree: {rel}\n"
                    f"    Add an entry to vendor.yaml (or, if this is OUR code, move it "
                    f"out of {c}/)."
                )

    for o in reg.ours:
        for op in o.paths:
            if not (reg.repo / op).is_dir():
                problems.append(f"{o.name}: `ours:` path does not exist: {op}")

    # The second ratchet, and the reason `ours:` exists. The container ratchet above only
    # watches three directories; foreign code imported anywhere else (WDF under sdk/lib,
    # the storage class drivers under drivers/) could only ever be found by looking, and
    # a scan nobody is required to run is a scan that goes stale. Now every hit has to be
    # answered ONE way or the other before the gate passes — and neither answer is one
    # this tool picks.
    for d, n in suspects(reg):
        problems.append(
            f"UNCLASSIFIED foreign-copyright code: {d} ({n} files)\n"
            f"    Decide, and record it in vendor.yaml: a `trees:` entry if it is\n"
            f"    vendored and upstream-tracked, an `ours:` entry if it is a translation\n"
            f"    target. Do not answer this by copyright grep — see tools/vendored.py."
        )

    # The THIRD ratchet. Same argument as the two above, one level down: it is not enough
    # that a tree is classified, its recorded LICENCE has to survive contact with what the
    # files say about themselves. Three trees failed this the day it was written.
    problems.extend(structural_problems(reg.repo / "vendor.yaml"))

    for path, recorded, unaccounted in license_drift(reg):
        listed = ", ".join(f"{lic} x{n}" for lic, n in sorted(unaccounted.items()))
        problems.append(
            f"UNDECLARED LICENCE in {path}: recorded `{recorded}`, but its own files\n"
            f"    declare: {listed}\n"
            f"    Record them in `license_extra:` with a reason, or move those files out\n"
            f"    of the tree if they are ours. A notices file built from the headline\n"
            f"    licence alone would be wrong about this tree."
        )

    # The notices document is a DERIVED artifact that is also a shipped legal one, so
    # it is checked in (a distributor needs it without running our tooling) AND gated
    # (a checked-in generated file with no freshness check is one that silently rots —
    # the registry gains a tree, the notices keep claiming the old list).
    # Staleness only, and only if the file is there. Whether a given repo OUGHT to ship
    # notices is repo policy, not registry-vs-tree agreement, and `--repo` is pointed at
    # synthetic fixture trees by the scanner controls; demanding a generated legal
    # document from each of those would be a false alarm about the fixture. That the
    # committed repo ships one is asserted directly in test_vendored.py.
    # THE FIFTH RATCHET. A file that names its own terms and names something that is
    # not FLOSS is the one thing none of the other four can see: they all key on a
    # copyright line or on a tree already in the registry, and atapi has neither.
    for value, example, n, _dir in declared_licences(reg):
        problems.append(
            f"NON-FLOSS LICENCE DECLARED by {n} file(s): `{value}`\n"
            f"    e.g. {example}\n"
            f"    A ReactOS `LICENSE:` header naming terms that are not a FLOSS licence.\n"
            f"    Settle it: delete the code, correct the header if it is wrong, or add a\n"
            f"    vendor.yaml entry recording what the terms actually are and why we ship it."
        )

    # The FOURTH ratchet, and the newest. vendor.yaml sees C; nothing saw the crates
    # cargo pulls in, which is how smoltcp — the TCP/IP stack — shipped unrecorded.
    # Checked against Cargo.lock rather than against live `cargo metadata`, so this
    # gate needs no cargo and no populated registry cache: the (name, version) set is
    # the drift that matters, and it is the half a lockfile can answer.
    lock_path = reg.repo / CARGO_LOCK
    if lock_path.exists():
        want = set(cargo_lock_crates(reg.repo))
        have = {(n, v) for n, v, _lic, _s in crates_rows(reg.repo)}
        if not (reg.repo / CRATES_PATH).exists():
            problems.append(
                f"{CRATES_PATH} is MISSING — the Rust half of the registry is not "
                f"recorded. Generate: tools/vendored.py --write-crates"
            )
        elif want != have:
            added = ", ".join(f"{n} {v}" for n, v in sorted(want - have)) or "none"
            gone = ", ".join(f"{n} {v}" for n, v in sorted(have - want)) or "none"
            problems.append(
                f"{CRATES_PATH} is STALE against {CARGO_LOCK}.\n"
                f"    in the lockfile but not recorded: {added}\n"
                f"    recorded but no longer in the lockfile: {gone}\n"
                f"    Regenerate: tools/vendored.py --write-crates"
            )

    # The grant ratchet gates, and it gates BOTH WAYS. Until now `--grants` was
    # advisory, which meant a licence gap in this tree survived exactly as long
    # as somebody remembered to run it by hand -- and a rule with no command
    # behind it is a rule you will break. Every finding must be listed below
    # with the argument for tolerating it; a NEW one fails, and so does an
    # accepted one that has since been FIXED, because a stale acceptance is how
    # a list like this turns into a list of things nobody has re-read.
    no_grant, dangling = grant_gaps(reg)
    found = {f"dir:{d}" for d, _n, _e in no_grant} | {f"ptr:{n}" for n in dangling}
    # The acceptance list is a statement about THIS tree, so it is only
    # reconciled against this tree. `check()` is also run against synthetic
    # repos by the tests, where every accepted key is trivially absent and
    # would report as stale -- noise that would drown the finding the test is
    # actually about.
    accepted = set(_GRANTS_ACCEPTED) if reg.repo.resolve() == REPO else set()
    for key in sorted(found - accepted):
        problems.append(
            f"NEW licence-grant finding `{key}` — code that claims a copyright and "
            f"never says what you may do with it, or a licence text it points at "
            f"that does not ship. Establish the terms (see tools/vendored.py "
            f"--grants), then either fix it or add it to _GRANTS_ACCEPTED with the "
            f"argument for tolerating it."
        )
    for key in sorted(accepted - found):
        problems.append(
            f"`{key}` is listed in _GRANTS_ACCEPTED but --grants no longer reports "
            f"it. If it was resolved, delete the entry; a tolerated finding that "
            f"has quietly gone away is one nobody will re-read."
        )

    notices_file = reg.repo / NOTICES_PATH
    if notices_file.is_file() and not any("--notices to quote" in p for p in problems):
        # Only meaningful once every tree has a quotable grant; otherwise the real
        # complaint is the missing grant, and a stale-notices line on top is noise.
        current = notices_file.read_text(encoding="utf-8")
        if current != notices(reg):
            problems.append(
                f"{NOTICES_PATH} is STALE — vendor.yaml or a licence file changed since it "
                f"was generated. Regenerate: tools/vendored.py --notices {NOTICES_PATH}"
            )
    return problems


#: Every field name an entry may carry. A key can only begin a line; if one appears
#: mid-line it was absorbed into the previous value.
_FIELD_NAMES = (
    "path|paths|name|upstream|license|license_file|license_ref|license_extra|"
    "modified|headers|version|reason|review|unwelcome|origin|tier|license_note|"
    "staleness|translate|translate_note"
)
_JOINED_FIELD = re.compile(rf"\S[ \t]{{2,}}((?:{_FIELD_NAMES}):)")


def structural_problems(path: Path) -> list[str]:
    """Fields silently swallowed into the previous value by a bad hand-edit.

    This exists because I broke vendor.yaml this way THREE TIMES in one session while
    editing it programmatically. The failure mode is nasty precisely because it is not
    a syntax error: `    license: MIT    modified: yes` still PARSES — `modified` just
    quietly stops being a field and becomes part of the licence string, and a block
    scalar will happily swallow a whole `reason:` the same way. The registry then
    reports clean while carrying less than it appears to.

    Two of the three were caught only by re-measuring afterwards and one by the loader
    complaining about a missing required field, which is luck rather than a check. So
    it is a check now.
    """
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as e:
        return [f"cannot read {path}: {e}"]
    out = []
    for n, line in enumerate(text.splitlines(), 1):
        m = _JOINED_FIELD.search(line)
        if m:
            out.append(
                f"vendor.yaml:{n}: `{m.group(1).strip()}` is not at the start of a line "
                f"— it was absorbed into the value above it, and has stopped being a "
                f"field. Line: {line.strip()[:90]}"
            )
    return out


#: How much of a `license_ref` file to scan for the grant. The grant is a header
#: comment; 200 lines is far past the longest one in this tree (mbedtls's Apache-2.0
#: notice) and stops us pasting an entire 6000-line source file into the notices.
_REF_SCAN_LINES = 200

#: A comment line's leading decoration. Every `license_ref` in this registry points at
#: a C or C++ file, so `#` and `;` are deliberately NOT comment leads here: treating
#: `#` as one made the ACPICA extraction run straight past the end of the licence and
#: start emitting the header's `#ifndef`/`#include` lines with their hashes stripped.
_COMMENT_LEAD = re.compile(r"^\s*(/\*+|\*+/|\*+|//+)\s?")


def license_text(reg: Registry, t: Tree) -> tuple[str, str]:
    """The licence grant to reproduce for `t`, as (source_description, text).

    Prefers a standalone `license_file`, quoted whole. Falls back to `license_ref`,
    where the grant is a comment header on a source file and has to be lifted out of
    it — 15 of the trees are in that shape, so this is the common path, not the
    exception. Returns the `license_note` verbatim when the tree carries no grant.
    """
    if t.license_file:
        path = reg.repo / t.license_file
        raw = path.read_text(encoding="utf-8", errors="replace")
        # Per-line rstrip: several upstream licence files carry trailing spaces, which
        # would land in a committed generated file and trip the whitespace lint. Not
        # legally meaningful, and doing it here keeps `--notices` output byte-equal to
        # what is checked in.
        return t.license_file, "\n".join(line.rstrip() for line in raw.splitlines()).strip()

    if t.license_ref:
        path = reg.repo / t.license_ref
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()[:_REF_SCAN_LINES]
        out: list[str] = []
        in_block = False
        for line in lines:
            stripped = line.strip()
            if in_block:
                out.append(_COMMENT_LEAD.sub("", line).rstrip())
                if "*/" in stripped:
                    in_block = False
                continue
            if not stripped:
                # A blank line BETWEEN comment blocks is a paragraph break; a blank line
                # after the last one is just leading whitespace on the code below, and
                # .strip() at the end removes it either way.
                out.append("")
                continue
            if stripped.startswith("/*"):
                out.append(_COMMENT_LEAD.sub("", line).rstrip())
                if "*/" not in stripped[2:]:
                    in_block = True
                continue
            if stripped.startswith("//"):
                out.append(_COMMENT_LEAD.sub("", line).rstrip())
                continue
            # First line of actual code — the header is over.
            break
        text = "\n".join(out).strip()
        return f"{t.license_ref} (header comment)", text

    # check() guarantees this exists; guard anyway so a hand-edited registry cannot
    # produce a notices file with an empty section rather than an error.
    if not t.license_note:
        raise RegistryError(f"{t.name}: no license_file, license_ref or license_note")
    return "NO GRANT IN TREE", t.license_note.strip()


def notices(reg: Registry) -> str:
    """The THIRD-PARTY-NOTICES document, generated from the registry.

    Distribution needs the licence TEXT, not a licence name, and hand-maintaining that
    list is how it goes stale — the registry already knows every tree, its upstream,
    its version probe and where its grant lives, so the document is derived rather
    than written. Trees with additional licences beyond the headline one list them:
    a notices file built from `license:` alone would be wrong about FEX and GLU.
    """
    out: list[str] = []
    out.append("# Third-party notices")
    out.append("")
    out.append(
        "This product includes third-party software. **This file is GENERATED** from "
        "`vendor.yaml` by `tools/vendored.py --notices` — edit the registry, not this "
        "file. Each entry reproduces the licence grant as it appears in the tree."
    )
    out.append("")
    out.append(f"{len(reg.trees)} vendored C/C++ components:")
    out.append("")
    for t in sorted(reg.trees, key=lambda x: x.name.lower()):
        version, err = reg.resolve_version(t)
        out.append(
            f"- [{t.name}](#{_anchor(t.name)}) — {version if not err else 'unknown'} — {t.license}"
        )
    out.append("")

    for t in sorted(reg.trees, key=lambda x: x.name.lower()):
        version, err = reg.resolve_version(t)
        source, text = license_text(reg, t)
        out.append("")
        out.append(f"## {t.name}")
        out.append("")
        out.append(f"- Upstream: {t.upstream}")
        out.append(f"- Version: {version if not err else 'unknown'}")
        out.append(f"- In this tree: `{'`, `'.join(t.prefixes)}`")
        out.append(f"- Licence: {t.license}")
        for lic, why in sorted(t.license_extra.items()):
            out.append(f"- Also contains: {lic} — {why}")
        if t.patch:
            out.append(f"- Modified by us: yes, see `{t.patch}`")
        out.append(
            f"- Grant reproduced from: `{source}`"
            if source != "NO GRANT IN TREE"
            else "- Grant reproduced from: **the tree carries no licence grant**"
        )
        out.append("")
        out.append("```")
        out.append(text)
        out.append("```")
    out.append("")

    # The Rust half. Licence TEXT is not reproduced here the way it is for the C
    # trees: a crate's grant lives in ~/.cargo, not in this repo, so there is nothing
    # in the tree to quote and quoting a remote file would be inventing provenance.
    # The SPDX expression cargo reports is what we can state truthfully.
    rows = crates_rows(reg.repo)
    if rows:
        ships = [r for r in rows if r[3]]
        out.append("## Rust crates")
        out.append("")
        out.append(
            f"This product also links {len(ships)} third-party Rust crates, out of "
            f"{len(rows)} that enter the build. Generated from `cargo metadata` into "
            f"`{CRATES_PATH}`; see that file for the ones used only to build or test, "
            "and for why the shipped/host split errs toward listing more."
        )
        out.append("")
        out.append("| crate | version | licence |")
        out.append("| --- | --- | --- |")
        for name, ver, lic, _ in ships:
            out.append(f"| {name} | {ver} | {lic} |")
        out.append("")
    return "\n".join(out)


def _anchor(name: str) -> str:
    """GitHub-flavoured markdown heading anchor."""
    return re.sub(r"[^a-z0-9_\- ]", "", name.lower()).replace(" ", "-")


def write_stamps(reg: Registry) -> list[str]:
    """(Re)generate the VENDORED marker in each registered tree. Returns paths written."""
    written = []
    for t in reg.trees:
        root = reg.repo / t.path
        if not root.is_dir():
            continue
        dest = root / _STAMP_NAME
        dest.write_text(_STAMP_TEMPLATE.format(name=t.name, upstream=t.upstream, license=t.license))
        written.append(dest.relative_to(reg.repo).as_posix())
    return written


def emit(reg: Registry) -> int:
    rows = []
    for t in reg.trees:
        version, err = reg.resolve_version(t)
        rows.append(
            (
                t.path,
                t.name,
                version if not err else "PROBE FAILED",
                t.license,
                reg.loc(t),
                t.translate,
            )
        )
    w = max(len(r[0]) for r in rows)
    print(f"{len(rows)} vendored trees registered in vendor.yaml\n")
    print(f"  {'path'.ljust(w)}  {'version':<22} {'licence':<22} {'LOC':>9}  translate")
    print(f"  {'-' * w}  {'-' * 22} {'-' * 22} {'-' * 9}  {'-' * 9}")
    total = 0
    for path, _name, version, lic, loc, translate in rows:
        total += loc
        print(f"  {path.ljust(w)}  {version[:22]:<22} {lic[:22]:<22} {loc:>9,}  {translate}")
    print(f"  {' ' * w}  {'':<22} {'TOTAL':<22} {total:>9,}")
    for value in TRANSLATE_VALUES:
        picked = [r for r in rows if r[5] == value]
        print(f"  translate={value}: {len(picked)} tree(s), {sum(r[4] for r in picked):,} lines")
    for t in reg.trees:
        if t.translate_note:
            print(f"  note — {t.name}: {' '.join(t.translate_note.split())}")

    unwanted = [t for t in reg.trees if t.unwelcome]
    if unwanted:
        print(f"\n{len(unwanted)} tree(s) flagged UNWELCOME — carried, but we want them gone:")
        for t in unwanted:
            print(f"  {t.name} ({t.path})")
            for line in t.unwelcome.strip().splitlines():
                print(f"      {line.strip()}")

    if reg.ours:
        orows = [
            (
                o.path if len(o.paths) == 1 else f"{o.path} (+{len(o.paths) - 1} more)",
                o.tier or "-",
                sum(reg.loc_path(p, _BODY_SUFFIXES) for p in o.paths),
                sum(reg.loc_path(p) for p in o.paths),
            )
            for o in reg.ours
        ]
        ow = max(len(r[0]) for r in orows)
        print(f"\n{len(orows)} foreign-copyright trees classified as OURS to translate")
        print("  (the complement of the table above; `bodies` is what enters the denominator)\n")
        print(f"  {'path'.ljust(ow)}  {'roadmap':<22} {'bodies':>9} {'w/ headers':>11}")
        print(f"  {'-' * ow}  {'-' * 22} {'-' * 9} {'-' * 11}")
        for path, tier, bodies, loc in orows:
            print(f"  {path.ljust(ow)}  {tier[:22]:<22} {bodies:>9,} {loc:>11,}")
        print(
            f"  {' ' * ow}  {'TOTAL':<22} {sum(r[2] for r in orows):>9,} "
            f"{sum(r[3] for r in orows):>11,}"
        )

    unknown = [t.name for t in reg.trees if not t.version]
    if unknown:
        print(f"\n  no version probe ({len(unknown)}): {', '.join(unknown)}")
    stale = [t for t in reg.trees if t.staleness]
    if stale:
        print(f"\n{len(stale)} tree(s) recorded as BEHIND UPSTREAM (dated audit, not live):")
        for t in stale:
            version, err = reg.resolve_version(t)
            print(f"    {t.name} — ours {version if not err else '?'}")
            for line in t.staleness.strip().splitlines():
                print(f"        {line.strip()}")

    flagged = [t for t in reg.trees if t.review]
    if flagged:
        print(f"\n  NEEDS REVIEW ({len(flagged)}) — classification not settled:")
        for t in flagged:
            print(f"    {t.name} ({t.path})")
            for line in t.review.strip().splitlines():
                print(f"        {line.strip()}")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--emit", action="store_true", help="print the registry table")
    ap.add_argument("--write-stamps", action="store_true", help="regenerate VENDORED markers")
    ap.add_argument(
        "--declared",
        action="store_true",
        help="files whose own LICENSE: header names terms that are not FLOSS",
    )
    ap.add_argument(
        "--write-crates",
        action="store_true",
        help="regenerate the Rust-crate snapshot from cargo metadata (needs cargo)",
    )
    ap.add_argument(
        "--grants",
        action="store_true",
        help="code that claims a copyright but never says what you may do with it, "
        "plus licence pointers naming a text that is not in the distribution",
    )
    ap.add_argument(
        "--suspects",
        action="store_true",
        help="unregistered dirs full of foreign-copyright source (a question, not a verdict)",
    )
    ap.add_argument(
        "--licences",
        action="store_true",
        help="licences a tree's own files declare that its registry entry does not record",
    )
    ap.add_argument(
        "--notices",
        nargs="?",
        const="-",
        metavar="PATH",
        help="generate THIRD-PARTY-NOTICES.md from the registry (default: stdout)",
    )
    ap.add_argument("--repo", type=Path, default=REPO)
    args = ap.parse_args(argv)

    try:
        reg = load(args.repo)
    except RegistryError as e:
        print(f"[vendored] REGISTRY ERROR: {e}", file=sys.stderr)
        return 2

    if args.notices:
        # A registry that cannot name a grant cannot produce honest notices, so this
        # refuses rather than emitting a document with a hole in it.
        blocking = [p for p in check(reg) if "--notices to quote" in p]
        if blocking:
            print("[vendored] cannot generate notices:", file=sys.stderr)
            for b in blocking:
                print(f"  - {b}", file=sys.stderr)
            return 1
        text = notices(reg)
        if args.notices == "-":
            print(text)
        else:
            dest = Path(args.notices)
            dest.write_text(text, encoding="utf-8")
            print(f"  wrote {dest} ({len(text.splitlines())} lines, {len(reg.trees)} components)")
        return 0
    if args.emit:
        return emit(reg)
    if args.write_crates:
        path = args.repo / CRATES_PATH
        path.write_text(crates_snapshot(args.repo), encoding="utf-8")
        rows = crates_rows(args.repo)
        shipping = sum(1 for r in rows if r[3])
        print(f"  wrote {path} ({len(rows)} crates, {shipping} of them shipping)")
        return 0
    if args.write_stamps:
        for p in write_stamps(reg):
            print(f"  wrote {p}")
        return 0
    if args.licences:
        drift = license_drift(reg)
        if not drift:
            print("every declared licence in every vendored tree is accounted for")
            return 0
        print(f"{len(drift)} tree(s) declare a licence their registry entry does not record:\n")
        for path, recorded, un in drift:
            listed = ", ".join(f"{lic} x{n}" for lic, n in sorted(un.items()))
            print(f"  {path}\n      recorded: {recorded}\n      in-tree:  {listed}")
        print(
            "\nRecord each in `license_extra:` with a reason, or move the files out if they\n"
            "are ours. `--check` fails until then."
        )
        return 1

    if args.grants:
        no_grant, dangling = grant_gaps(reg)
        if not no_grant and not dangling:
            print("no grant gaps — every claimed body says what you may do with it")
            return 0
        if dangling:
            print(f"{len(dangling)} DANGLING licence pointer(s) — the text is not shipped:\n")
            for name, info in sorted(dangling.items(), key=lambda x: -x[1]["count"]):
                print(f"  {info['count']:>4} refs -> {name:24} e.g. {info['example']}")
            print()
        if no_grant:
            print(f"{len(no_grant)} dir(s) claiming copyright with NO grant anywhere:\n")
            w = max(len(d) for d, _, _ in no_grant)
            for d, n, ex in no_grant:
                print(f"  {d.ljust(w)}  {n:>4} files  e.g. {ex}")
        print(
            "\nA disclaimer is not a grant and a pointer is not a licence. Each of these\n"
            "is either owed a licence text in the tree, a corrected reference, or a\n"
            "vendor.yaml entry recording what the terms actually are."
        )
        return 1

    if args.declared:
        rows = declared_licences(reg)
        if not rows:
            print("no file declares a non-FLOSS licence outside the registry")
            return 0
        print(f"{len(rows)} non-FLOSS licence declaration(s):\n")
        for value, example, n, _d in rows:
            print(f"  {n:4} file(s)  `{value}`\n           e.g. {example}")
        print(
            "\nA ReactOS `LICENSE:` header is the code naming its own terms. Delete it,"
            "\ncorrect it, or record it in vendor.yaml. `--check` fails until then."
        )
        return 1

    if args.suspects:
        found = suspects(reg)
        if not found:
            print("no unclassified foreign-copyright directories — every one is decided")
            return 0
        print(f"{len(found)} UNCLASSIFIED dir(s) with >=5 foreign-copyright files:\n")
        w = max(len(d) for d, _ in found)
        for d, n in found:
            print(f"  {d.ljust(w)}  {n:>4} files")
        print(
            "\nThese are QUESTIONS, and `--check` now fails until each is answered. Each\n"
            "is either (a) a vendored tree owed a `trees:` entry, or (b) foreign-derived\n"
            "code that IS a translation target and belongs in `ours:`, staying in the\n"
            "denominator. Decide per tree; do not bulk-add, and do not decide by grep."
        )
        return 1

    problems = check(reg)
    if problems:
        # The registry is named ABSOLUTELY here: with --repo in play there can be more
        # than one, and "vendor.yaml disagrees" is useless if you cannot tell which.
        print(
            f"[vendored] {args.repo / 'vendor.yaml'} disagrees with the tree "
            f"({len(problems)} problem(s)):\n"
        )
        for p in problems:
            print(f"  - {p}")
        print("\n  vendor.yaml is the source of truth for which C is NOT ours to translate.")
        return 1
    # STDOUT IS THE FINDINGS CHANNEL for `vibeos-cli check` (lints.py `findings_from`),
    # and every non-empty stdout line that is not a recognised tally counts as one. A
    # chatty success line here would read as a permanent new finding and hold the stage
    # red. The status belongs on stderr; silence on stdout is the pass.
    print(
        f"[vendored] OK — {len(reg.trees)} vendored trees + {len(reg.ours)} classified ours, "
        f"registry agrees with the tree",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
