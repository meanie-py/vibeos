"""Closed-set sealing for runtime callee occurrences in body-worker prompts."""

from __future__ import annotations

import itertools
import re
from collections.abc import Mapping
from dataclasses import dataclass

from .dead_regions import dead_region_spans, in_dead_region
from .lex_mask import comment_spans, directive_spans, mask_code


class CallSealError(RuntimeError):
    pass


@dataclass(frozen=True)
class SealedCall:
    placeholder: str
    original: str
    # This call site sits in code the build never executes -- overwhelmingly a
    # DUMMY_PSEH `RpcExcept` handler. Defaults False so anything not proven dead stays
    # required. See dead_regions.py.
    dead: bool = False
    # Top-level arguments at the C call site, or None where counting them cannot be
    # trusted. The Rust site must agree -- see `_argument_count` for why that is the
    # oracle that catches a SHIFTED envelope, which token accounting alone cannot.
    argc: int | None = None
    # Upper bound when the callee has C++ DEFAULT ARGUMENTS: the C site may omit them, a
    # faithful Rust lowering cannot. None means "no defaults, exact match required".
    argc_max: int | None = None
    # This callee is a C++ METHOD, so `argc` above already counts the leading `this` that
    # the C site spells implicitly. Recorded because the Rust site can then be REPAIRED
    # rather than rejected -- see `repair_dropped_receivers`.
    receiver: bool = False
    # This callee is an ATL CWindow method, so `argc` counts a leading window HANDLE the C++
    # site spells implicitly. Recorded for the same reason -- see `repair_window_receivers`.
    hwnd_receiver: bool = False


@dataclass(frozen=True)
class SealedCalls:
    text: str
    calls: tuple[SealedCall, ...]
    # Function-like macros this body calls. They get no token (see `seal_calls`), and
    # the envelope has to SAY so by name: a model that assumes every call it emits
    # needs a token numbers the macro too and shifts every real call by one.
    macros: tuple[str, ...] = ()


_PLACEHOLDER_RE = re.compile(r"\b__RUSTCONV_CALL_[0-9]{4}__\b")
# Superseded by `macro_authority.function_like_macros`, which derives this set from
# the real preprocessor instead of remembering it. Kept as a floor for the two cases
# the preprocessor cannot name: `sizeof` is a KEYWORD, not a macro, and the debug
# macros are stripped from the body before sealing anyway. Everything else here is a
# function-like macro the derived set already covers — see that module for why a
# macro can never satisfy the "exactly once as a called identifier" envelope.
_EXCLUDED = {
    "TRACE",
    "WARN",
    "ERR",
    "FIXME",
    "MESSAGE",
    "SUCCEEDED",
    "FAILED",
    "FAILED_UNEXPECTEDLY",
    "sizeof",
    "_countof",
}
# Never worth naming in the envelope's "these have no token" list: `sizeof` is a C
# KEYWORD, and the debug macros are stripped from the body before sealing, so the model
# never sees either as a call it might try to tokenize.
_NEVER_DISCLOSED = {"sizeof", "TRACE", "WARN", "ERR", "FIXME", "MESSAGE"}
_PURE_REUSABLE_CALLS = {"strlen", "wcslen"}
_ANY_SEAL_TOKEN_RE = re.compile(r"\b__RUSTCONV_(?:CALL|FACT|WIDE)_[0-9]{4}__\b")


# Shared with fact_seal and cpp_com. It masks Rust as well as C++, which the three
# private copies it replaced did not: each treated `'cleanup:` as an open char
# literal and blanked the body to the next apostrophe. See `lex_mask`.
_code_mask = mask_code


def scrub_comment_token_echoes(text: str) -> str:
    """Neutralize seal tokens repeated only in model-authored comments."""
    out = list(text)
    comment = comment_spans(text)
    for match in _ANY_SEAL_TOKEN_RE.finditer(text):
        if all(comment[match.start() : match.end()]):
            out[match.start() : match.end()] = "COMMENT_TOKEN_ECHO".ljust(
                match.end() - match.start(), "_"
            )
    return "".join(out)


def extern_declaration_spans(text: str) -> list[tuple[int, int]]:
    """Character spans of `extern "..." { … }` DECLARATION blocks.

    A forward declaration names a callee but does not call it, and the mechanical
    prompt explicitly ORDERS one: "emit a forward `extern "system" { fn Name(args...)
    -> Ret; }` block immediately above the function". So a correct mechanical
    translation writes every sealed callee twice — once declared, once called — and
    the "exactly once" duplicate check fails through no fault of the model.

    The shell32 COM path never hit this because `normalize_worker_body` deletes
    model-authored declaration-only extern blocks before unsealing. Plain C KEEPS
    them (they are how the assembler learns the signature), so the extra occurrence
    is sanctioned here instead — and only the extra one: a token must still appear
    exactly once in executable code.

    Only DECLARATION blocks count: a block whose `fn` items all end at `;`. An
    `extern "C" { fn f() { … } }` with a body is not one.
    """
    spans: list[tuple[int, int]] = []
    for match in re.finditer(r'\bextern\s*"[^"]*"\s*\{', text):
        depth = 0
        i = start = match.end() - 1
        while i < len(text):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        end = min(i + 1, len(text))
        if not re.search(r"\bfn\b[^;{]*\{", text[start:end]):
            spans.append((start, end))
    return spans


_IDENT_RE = re.compile(r"\b[A-Za-z_]\w*\b")


def _argument_count(masked: str, name_end: int) -> tuple[int, str] | None:
    """Top-level argument count and argument text of the call starting at ``name_end``.

    None when the parens do not balance inside ``masked`` -- an unparseable site is
    never a finding, only a skip.

    This is the oracle for a SHIFTED envelope, which nothing else can see. When a model
    consumes a token for an untokenized macro call, every later token slides onto the
    PREVIOUS call's site: the count of tokens is still right, each appears exactly once,
    and the last one runs out -- whereupon the by-name recovery below rescues it and the
    audit passes. `ApplyOrCancelChanges` shipped that way, restoring `swprintf`'s name
    onto `StringFromCLSID`'s site and `CoTaskMemFree`'s onto `swprintf`'s. Arity catches
    it on the first slid site, because a shift only lines up by coincidence.
    """
    opening = masked.find("(", name_end)
    if opening < 0 or masked[name_end:opening].strip():
        return None
    depth = 0
    segments: list[str] = []
    segment_start = opening + 1
    for pos in range(opening, len(masked)):
        ch = masked[pos]
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
            if depth == 0:
                segments.append(masked[segment_start:pos])
                # rustfmt puts a trailing comma on every multi-line call, and a
                # six-argument `RegQueryValueExW` written across seven lines counted as
                # seven arguments. That is not an argument; it is a line ending.
                while segments and not segments[-1].strip():
                    segments.pop()
                inner = masked[opening + 1 : pos]
                # A `#if` arm INSIDE the argument list makes the count depend on which arm
                # compiles: comctl32's TOOLTIPS_Show passes CreateRoundRectRgn's fourth
                # argument once per `__REACTOS__` arm, so the C text reads as 7 arguments
                # and the model's correct 6 was refused on every sample. Skip, never judge.
                if re.search(r"(?m)^[ \t]*#[ \t]*(?:if|ifdef|ifndef|elif|else|endif)\b", inner):
                    return None
                return len(segments), inner
        elif depth == 1 and ch == ",":
            segments.append(masked[segment_start:pos])
            segment_start = pos + 1
    return None


def seal_calls(
    text: str,
    required_calls: set[str],
    function_like_macros: frozenset[str] = frozenset(),
    tu_source: str = "",
    receiver_callees: frozenset[str] = frozenset(),
    callee_cpp_names: Mapping[str, str] | None = None,
    hwnd_receiver_callees: Mapping[str, int] | None = None,
) -> SealedCalls:
    """Replace each known runtime-callee occurrence with a unique token.

    ``function_like_macros`` names every `#define NAME(...)` visible to the TU (see
    `macro_authority`). Those are excluded: a macro occurrence has no 1:1 Rust call
    to carry the token, so sealing one makes the envelope unsatisfiable while
    protecting nothing — the macro's name is not in the output to begin with.
    """
    untokenized = _EXCLUDED | function_like_macros
    required = required_calls - untokenized
    if not required:
        return SealedCalls(text, ())
    masked = _code_mask(text)
    # C++ default arguments are declared at FILE scope, not inside the body being sealed,
    # so the arity range has to be read from the whole TU when we have it.
    decl_scope = _code_mask(tu_source) if tu_source else masked
    body_start = masked.find("{")
    # `sizeof` is a KEYWORD and the debug macros are stripped from the body before
    # sealing, so neither is a name the model can be confused by. Everything else that
    # occurs here is one the envelope must disclaim by name.
    named_macros = sorted(
        {
            match.group(1)
            for match in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", masked)
            if match.group(1) in untokenized
            and match.start(1) > body_start
            and match.group(1) not in _NEVER_DISCLOSED
        }
    )
    # A `#define NAME(x)` in the body is a DEFINITION, not a call occurrence, and its name is
    # not a runtime callee at all. See lex_mask.directive_spans.
    directive = directive_spans(text)
    spans = []
    for match in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", masked):
        name = match.group(1)
        if match.start(1) <= body_start or name not in required:
            continue
        if directive[match.start(1)]:
            continue
        # C++ direct initialization looks call-like to a lexical scanner:
        # ``CComHeapPtr<ITEMIDLIST> pidl(factory())``. The declarator name is
        # storage, not a runtime callee.
        statement_start = max(
            body_start,
            masked.rfind(";", body_start, match.start(1)),
            masked.rfind("{", body_start, match.start(1)),
            masked.rfind("}", body_start, match.start(1)),
        )
        prefix = masked[statement_start + 1 : match.start(1)].lstrip()
        # A C body may carry a block-scope ``extern RET Callee(...)`` prototype.
        # It is ABI context, not an executable call occurrence.
        if re.fullmatch(r"extern\s+[^;{}=()]+", prefix):
            continue
        if re.fullmatch(r"[A-Z_]\w*(?:\s*<[^;{}()]+>)?\s+\**\s*", prefix):
            continue
        spans.append((match.start(1), match.end(1), name))

    dead_spans = dead_region_spans(text, masked)
    result = text
    calls = []
    for index, (start, end, original) in reversed(list(enumerate(spans))):
        placeholder = f"__RUSTCONV_CALL_{index:04d}__"
        counted = _argument_count(masked, end)
        # A macro in ARGUMENT position is the one case where C and Rust legitimately
        # disagree on the count: shell32's `IID_PPV_ARG(IProgressDialog, &ppv)` is one C
        # argument and two Rust ones. Don't count those sites at all.
        argc = (
            None
            if counted is None or any(i in untokenized for i in _IDENT_RE.findall(counted[1]))
            else counted[0]
        )
        argc_max = _defaulted_param_count(decl_scope, original)
        # A sibling METHOD is sealed under its LOWERED name (`CDefView_LoadColumn_body`), a
        # spelling that appears nowhere in the C++ source -- so the default-argument lookup
        # above silently found nothing and the tolerance it exists to provide never applied to
        # a method at all. Free functions escaped this only because their name is not
        # rewritten. `LoadColumn(UINT, UINT, BOOL, UINT ForceWidth = 0)` called with three
        # arguments then rejected the only correct five-argument lowering, and no retry could
        # ever have passed: three of CDefView's six survivors were this one bug.
        if argc_max is None and callee_cpp_names:
            cpp_name = callee_cpp_names.get(original)
            if cpp_name:
                argc_max = _defaulted_param_count(decl_scope, cpp_name)
        # An ATL CWindow method reaches Rust as the Win32 API of the same name with the window
        # handle prepended, so its faithful site has one more argument than the C++ one -- the
        # same implicit receiver as a sibling method, wearing a base class's name. Declared
        # counts come from atlwin.h (see cpp_core.cwindow_method_arity), never a remembered
        # list, and only for a class that actually derives from CWindow, where the member hides
        # the global and a bare call cannot mean the Win32 function.
        # `::` is how the C++ says which one it means, and ignoring it inverted the fix: the
        # SAME name is a member call bare (`ScreenToClient(&pt)`, handle implicit) and the
        # Win32 API when qualified (`::GetWindowRect(hWndHdr, &r)`, handle already an
        # argument). Adding a receiver to the qualified form broke three bodies that had been
        # correct, which is the tell that the rule -- not the model -- was wrong.
        qualified = masked[:start].rstrip().endswith("::")
        if hwnd_receiver_callees and original in hwnd_receiver_callees and not qualified:
            declared = hwnd_receiver_callees[original]
            if argc is not None:
                argc += 1
                argc_max = max(declared + 1, argc)
        # A C++ METHOD call carries an implicit receiver that the Rust lowering spells as an
        # explicit leading `this`, so a faithful Rust call site has exactly one MORE argument
        # than the C one. Counting the C site alone both rejects correct output and -- the
        # reason this matters -- ACCEPTS a body that dropped `this`: `CDrivesFolderEnum_dtor`
        # shipped `DeleteList_body()` against a fn whose first parameter is `this`, and the
        # envelope called it good because 0 matched the C site's 0.
        is_receiver = original in receiver_callees
        if is_receiver:
            if argc is not None:
                argc += 1
            if argc_max is not None:
                argc_max += 1
        calls.append(
            SealedCall(
                placeholder,
                original,
                in_dead_region(dead_spans, start, end),
                argc,
                argc_max,
                is_receiver,
                bool(hwnd_receiver_callees) and original in hwnd_receiver_callees and not qualified,
            )
        )
        result = result[:start] + placeholder + result[end:]
    calls.reverse()
    return SealedCalls(result, tuple(calls), tuple(named_macros))


_GOTO_RE = re.compile(r"\bgoto\s+([A-Za-z_]\w*)\s*;")
_LABEL_RE = re.compile(r"(?m)^[ \t]*([A-Za-z_]\w*)[ \t]*:(?!:)")


def _name_satisfies(token: str, sealed: SealedCalls, replicable: set[str], exec_text: str) -> bool:
    """Whether a token's own callee NAME, written out in call position, stands in.

    Only inside the `goto` replication region, where occurrence identity is already
    conceded (see `_replicable_after_goto`). Duplicating the cleanup tail makes the
    tail's tokens ambiguous, and a model that has just written the same block four
    times routinely stops threading opaque tokens through it and writes what the
    callee-context block told it the callee is called:

        HeapFree(GetProcessHeap(), 0, nFileOp.pFrom as *mut c_void)

    That is not a dropped call, which is the only thing `missing` exists to catch —
    and it is character-for-character what unsealing would have produced anyway, so
    accepting it changes no output byte. A name the model INVENTED still fails: it
    would not match `original`. Outside the region, "exactly once" is untouched.
    """
    if token not in replicable:
        return False
    original = next((call.original for call in sealed.calls if call.placeholder == token), None)
    if not original:
        return False
    return re.search(rf"(?<!fn )\b{re.escape(original)}\s*\(", exec_text) is not None


_CASE_RE = re.compile(r"(?m)^[ \t]*(?:case\b[^:;]*|default)[ \t]*:(?!:)")
# A C arm that ends in one of these does NOT fall through to the next label.
_TERMINATED_RE = re.compile(r"(?:\bbreak\b|\breturn\b[^;]*|\bgoto\b[^;]*|\bcontinue\b)\s*;\s*$")


def _first_fallthrough_case(masked: str) -> int | None:
    """Offset of the first `case`/`default` label whose arm FALLS THROUGH, or None.

    Rust `match` has no fall-through, so the only faithful lowering of a C arm that
    runs on into the next one is to DUPLICATE the next arm's body into it — which
    writes every call token in that body twice. Same phenomenon as `goto`
    (see `_replicable_after_goto`), reached by a different route, and it needs the
    same positional concession: `ntmarta.c`'s `AccpGetTrusteeSid` falls
    TRUSTEE_IS_OBJECTS_AND_NAME through into TRUSTEE_IS_NAME and duplicated all five
    of its tokens, so the whole paid body was rejected.

    Conservative on purpose. An arm is treated as falling through unless it visibly
    ends in `break`/`return`/`goto`/`continue`; an arm ending in `}` is AMBIGUOUS (the
    jump may sit inside the block, which is exactly the ntmarta shape) and counts as a
    fall-through. Over-tolerating here weakens only the DUPLICATE half of the envelope
    — `missing` still catches a dropped call and `invented` still catches a
    hallucinated token — which is the same trade `_replicable_after_goto` documents.
    An EMPTY region between two labels is a shared arm (`case A: case B:`), not a
    fall-through: nothing is duplicated, so it does not concede anything.
    """
    labels = list(_CASE_RE.finditer(masked))
    for prev, nxt in itertools.pairwise(labels):
        region = masked[prev.end() : nxt.start()].strip()
        if not region:
            continue  # `case A:` immediately followed by `case B:` — a shared arm.
        if _TERMINATED_RE.search(region):
            continue
        return prev.start()
    return None


def _replicable_after_goto(sealed: SealedCalls) -> set[str]:
    """Tokens a structured lowering of C `goto` may legitimately write more than once.

    Rust has no `goto`, so every lowering DUPLICATES code, and how much is not a
    property the translation is free to choose consistently. Three consecutive paid
    responses for the same two functions produced three different faithful shapes:

      * the label's tail inlined at each jump site (`SHFileOperationA`, 4 copies);
      * the same, 3 copies (`SHELL32_FileOperation`);
      * a nested cascade duplicating the WHOLE remainder per path — which replicates
        calls that sit nowhere near the label.

    A count-based budget was tried first (`#goto + 1`, tail only) and the third shape
    walked straight through it. So the rule is positional, not numeric: from the point
    where duplication can begin — the earlier of the first `goto` and its target label,
    which also covers the backwards retry-loop idiom — a token may repeat.

    "Exactly once" survives everywhere else, which is most bodies. And this weakens
    only the DUPLICATE half of the envelope: `missing` still catches a dropped or
    commented-out call, and `invented` still catches a hallucinated token — that half
    caught three invented FACT tokens on the very run this was written for.

    Residual risk, stated plainly: inside the replication region this cannot tell "the
    same call on N distinct exit paths" (correct) from "the same call twice on one
    path" (a double free). Neither could the old check — it rejected the whole TU
    either way, and one rejected worker assembles no file at all.
    """
    masked = _code_mask(sealed.text)
    gotos = _GOTO_RE.findall(masked)
    fallthrough = _first_fallthrough_case(masked)
    if not gotos:
        if fallthrough is None:
            return set()
        return {
            call.placeholder
            for call in sealed.calls
            if sealed.text.find(call.placeholder) >= fallthrough
        }
    first = _GOTO_RE.search(masked)
    assert first is not None
    start = first.start()
    if fallthrough is not None:
        start = min(start, fallthrough)
    targets = set(gotos)
    for match in _LABEL_RE.finditer(masked):
        # `case X:` / `default:` / a ternary are not jump targets.
        if match.group(1) in targets:
            start = min(start, match.start())
    # Positions are taken in the SEALED text, where each placeholder is unique; a
    # pre-seal offset diverges as soon as a token replaces a shorter name.
    return {
        call.placeholder for call in sealed.calls if sealed.text.find(call.placeholder) >= start
    }


def call_contract(sealed: SealedCalls) -> str:
    if not sealed.calls:
        return ""
    tokens = ", ".join(call.placeholder for call in sealed.calls)
    # "Do not duplicate" is a lie when the C body uses `goto`, and the models paid to
    # obey it acted accordingly: needing the cleanup tail on three paths and forbidden
    # to repeat a token, they MINTED NEW NUMBERS past the end of the list
    # (`dde_connect` invented three, `SIC_IconAppend` two). Saying what is actually
    # true costs nothing and removes the incentive.
    # A `switch` arm that FALLS THROUGH is the same lie by another route: Rust `match`
    # has no fall-through, so the next arm's body has to be written into this one as
    # well. ntmarta's `AccpGetTrusteeSid` did exactly that and repeated all five of its
    # tokens; the audit rejected the whole body, which assembles no file for the TU.
    _masked = _code_mask(sealed.text)
    _dup_reason = (
        "`goto`, which Rust has no equivalent for"
        if _GOTO_RE.search(_masked)
        else "a `switch` arm that falls through, which Rust `match` has no equivalent for"
    )
    if _GOTO_RE.search(_masked) or _first_fallthrough_case(_masked) is not None:
        rule = (
            "Preserve EACH token as the called function/method identifier in executable "
            f"Rust. This body uses {_dup_reason}, so a faithful "
            "lowering may need the same call on more than one path: where that happens, "
            "REPEAT the identical token — repetition is expected here. Never invent a token "
            "number that is not listed below; the list is complete."
        )
    else:
        rule = (
            "Preserve EACH token exactly once as the called function/method identifier in "
            "executable Rust. Do not duplicate one. Never invent a token number that is not "
            "listed below; the list is complete."
        )
    return (
        "\n## Deterministic call envelope\n"
        "Each identifier below is one opaque runtime callee occurrence owned by rustconv. "
        f"{rule} Do not rename, define, merge, move into a comment, or omit one; rustconv "
        "restores the callee names after your response.\n"
        # The list is complete for calls the sealer can SEE, and it cannot see through a
        # function-like macro: `MSVCRT_CHECK_PMT(x)` expands to a call to
        # _invalid_parameter, and gets no token. A model that assumes every call it emits
        # needs one then numbers that macro `..._0000__` and shifts every real call by
        # one, so the last token has nowhere to go and it invents the next index. That is
        # not a translation error -- the Rust is usually right -- but the audit rejects
        # it, and one rejected worker assembles no file for the whole TU. Say so, rather
        # than let the model deduce it.
        "A call that appears only inside a function-like macro has NO token: translate it "
        "as an ordinary named call and do not consume or invent a token for it.\n"
        # The generic sentence above was not enough on its own: netcfgx's
        # `ReadParameterValue` invented three tokens and `ApplyOrCancelChanges` shifted
        # its whole envelope by one, both because the untokenized names sat INTERLEAVED
        # with tokenized ones and the model could not tell which was which. Naming them
        # is free and leaves nothing to infer.
        + (
            "The following identifiers in this body are function-like macros, not sealed "
            f"callees: {', '.join(sealed.macros)}. They deliberately have no token — write "
            "each as an ordinary named call and do not shift the numbering to cover one.\n"
            if sealed.macros
            else ""
        )
        + f"Call tokens ({len(sealed.calls)}): {tokens}\n"
    )


def _doubled_after_unseal(rust: str, sealed: SealedCalls) -> list[str]:
    """Callees the model wrote by NAME *around* the sealed token, doubling the call.

    The "exactly once" rule counts TOKENS, and this shape uses the token exactly once -- the
    duplicate is the callee's NAME, hand-written by the model as a wrapper. Substitution then
    turns `NAME(<token>(args))` into `NAME(NAME(args))`, which is a second call to the same
    function with the first call's result as its only argument.

    Seen once, in CCPLItemMenu::InvokeCommand: eight callees doubled in one accepted body,
    including a zero-argument one (`_ILCreateControlPanel(_ILCreateControlPanel())`). It does
    not compile, so it fails loudly rather than shipping -- but it is a PAID artifact that the
    envelope passed, and the whole point of the envelope is that accepted output is trustworthy.

    Only flagged when the inner call is the outer's ENTIRE argument list, which is the
    corruption's signature; a genuine `f(f(x), y)` is left alone.
    """
    found: list[str] = []
    for name in sorted({call.original for call in sealed.calls}):
        for m in re.finditer(rf"\b{re.escape(name)}\s*\(", rust):
            open_paren = m.end() - 1
            try:
                close = _match_paren(rust, open_paren)
            except ValueError:
                continue
            inner = rust[open_paren + 1 : close].strip()
            if re.fullmatch(rf"{re.escape(name)}\s*\(.*\)", inner, re.S):
                found.append(name)
                break
    return found


def _match_paren(text: str, open_idx: int) -> int:
    depth = 0
    for i in range(open_idx, len(text)):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return i
    raise ValueError("unbalanced")


def _defaulted_param_count(masked: str, callee: str) -> int | None:
    """Total parameter count of `callee`, if its declaration gives any parameter a DEFAULT.

    C++ default arguments are one more construct Rust cannot spell. `SHELL_GetAutoRunInfPath(
    PCWSTR, PWSTR, BOOL ForInvoke = FALSE)` is called with two arguments in the C and MUST be
    called with three in Rust, so the exact-arity check rejects a correct lowering -- it did,
    on two different CDrivesFolder functions at once, which is the tell that the fault is in
    the checker rather than the model.

    Returns the full parameter count so the audit can accept anything from the C site's count
    up to it; None when the callee has no defaults, which keeps exact matching everywhere else.
    """
    for m in re.finditer(rf"\b{re.escape(callee)}\s*\(", masked):
        depth, i, n = 0, m.end() - 1, len(masked)
        while i < n:
            if masked[i] == "(":
                depth += 1
            elif masked[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        else:
            continue
        params = masked[m.end() : i]
        # A `=` at the top level of a parameter list is a default; `==` is not, and nested
        # parens/templates are skipped by the depth test.
        top, d, has_default = [], 0, False
        for j, ch in enumerate(params):
            if ch in "(<":
                d += 1
            elif ch in ")>":
                d -= 1
            elif d == 0 and ch == "=" and params[j - 1 : j] != "=" and params[j + 1 : j + 2] != "=":
                has_default = True
            elif d == 0 and ch == ",":
                top.append(j)
        if has_default and params.strip():
            return len(top) + 1
    return None


def _mask_turbofish_commas(text: str) -> str:
    """Blank the commas inside `::<...>` spans, leaving every other byte and the LENGTH alone.

    `_argument_count` tracks `()[]{}` depth but not `<>`, so `transmute::<A, B>(x)` reads as
    two arguments where the C had one. The old defence was to skip any site whose argument
    text mentioned `::<` at all -- which also skipped every site containing a harmless
    `size_of::<T>()`, and that is not a corner: it is how nearly every `memcpy` lowering
    spells its length. thread.c's `_beginthread_trampoline` shipped through that hole with
    `msvcrt_get_thread_data` -- a ZERO-argument function -- wearing the memcpy's three
    arguments, and no finding was raised.

    A callee-side turbofish needs no handling here: `_argument_count` already returns None
    when anything sits between the name and the `(`.

    Length is preserved so callers can keep indexing the unmasked string.
    """
    out = list(text)
    i = 0
    while (start := text.find("::<", i)) >= 0:
        depth, pos = 0, start + 2
        while pos < len(text):
            ch = text[pos]
            if ch == "<":
                depth += 1
            # `->` inside a `fn(A) -> B` type argument is not a closing angle bracket.
            elif ch == ">" and text[pos - 1] != "-":
                depth -= 1
                if depth == 0:
                    break
            pos += 1
        else:
            break
        for p in range(start, pos):
            if out[p] == ",":
                out[p] = " "
        i = pos + 1
    return "".join(out)


def _arity_disagreements(exec_mask: str, sealed: SealedCalls) -> list[str]:
    """Tokens whose Rust call site takes a different number of arguments than the C one.

    A shifted envelope passes every other check in this file — right token count, each
    used once, none invented — because the shift is a permutation, not a loss. What it
    cannot preserve is the ARGUMENTS: slide `swprintf`'s token onto `StringFromCLSID`'s
    site and it now wraps two arguments where the C had three.

    Skipped rather than judged wherever the count is not a fair comparison: a C site
    whose arguments mention a function-like macro (`argc is None`, see `seal_calls`) and
    an unbalanced Rust site. A turbofish used to skip the site too; it now only has its
    own commas blanked, because discarding the whole site is what let a three-argument
    call to a zero-argument callee ship — see `_mask_turbofish_commas`. A token that
    appears more than once — the `goto` replication region — needs only ONE occurrence to
    agree, since the copies are the same call.
    """
    exec_mask = _mask_turbofish_commas(exec_mask)
    findings = []
    for call in sealed.calls:
        if call.argc is None:
            continue
        counts = [
            counted
            for match in re.finditer(rf"\b{re.escape(call.placeholder)}\b", exec_mask)
            if (counted := _argument_count(exec_mask, match.end())) is not None
        ]
        hi = call.argc_max if call.argc_max is not None else call.argc
        # An OVERLOADED callee can make the range empty, and an empty range rejects every
        # possible body forever. `argc` is counted at the C CALL SITE; `argc_max` comes from
        # looking the NAME up among the declarations, which for an overload set can land on a
        # different overload than the one being called. filetypes.cpp has both
        # `DoExtractIcon(LPCWSTR, INT, UINT, UINT)` and
        # `DoExtractIcon(LPCWSTR, INT = 0, BOOL = FALSE)`, so a four-argument site was checked
        # against the three-argument declaration and reported the giveaway "wants 4, got 4".
        # The default-argument tolerance can only ever WIDEN the accepted range, so a maximum
        # below the minimum is not a tighter rule, it is a lookup that missed.
        hi = max(hi, call.argc)
        if not counts or any(call.argc <= count <= hi for count, _ in counts):
            continue
        got = "/".join(str(count) for count, _ in counts)
        findings.append(f"{call.placeholder}={call.original} wants {call.argc}, got {got}")
    return findings


def repair_dropped_receivers(rust: str, sealed: SealedCalls) -> tuple[str, list[str]]:
    """Insert the explicit `this` a sibling METHOD call site is missing.

    The C++ receiver is implicit, so the prepared C reads `_DoCallback_body(a, b, c)`
    while the Rust worker's first parameter is `this`. Making that explicit is a
    mechanical lowering the tool owns -- `normalize_worker_semantics` has always done it
    on the unsealed path -- but it is keyed to the callee NAME, and under `--seal-calls`
    the name is an opaque token, so it could never fire. The envelope's arity check then
    rejected a body whose ONLY defect was the receiver the tool was supposed to add: five
    of seven rejects on CDefaultContextMenu, each re-billed on retry, for a repair that
    costs nothing.

    Deliberately narrow. A site is repaired only when it is short by EXACTLY the receiver
    and does not already lead with `this`; anything else is a real disagreement and stays
    a hard failure. The receiver can only ever be `this` here, because the helper binder
    refused any sibling call with a different receiver long before this point.
    """
    if not any(call.receiver for call in sealed.calls):
        return rust, []
    insertions: list[tuple[int, str, str]] = []
    masked = _code_mask(rust)
    for call in sealed.calls:
        if not call.receiver or call.argc is None:
            continue
        for match in re.finditer(rf"\b{re.escape(call.placeholder)}\b", masked):
            counted = _argument_count(masked, match.end())
            if counted is None or "::<" in counted[1]:
                continue
            count, text = counted
            if count != call.argc - 1:
                continue
            if re.match(r"\s*this\b", text):
                continue
            opening = masked.find("(", match.end())
            insertions.append(
                (
                    opening + 1,
                    "this" + (", " if count else ""),
                    f"{call.placeholder}={call.original}",
                )
            )
    for position, insertion, _ in sorted(insertions, reverse=True):
        rust = rust[:position] + insertion + rust[position:]
    return rust, sorted({repaired for _, _, repaired in insertions})


def repair_window_receivers(rust: str, sealed: SealedCalls) -> tuple[str, list[str]]:
    """Turn `(*obj).Method(args)` into `Method(obj, args)` for an ATL CWindow method.

    `m_ListView.DestroyWindow()` has a receiver that is a WINDOW, not an object: the field is
    an `HWND`, so Rust method-call syntax on it does not exist and the only lowering is the
    Win32 API with the handle as argument zero. A model that has just written the implicit
    form correctly still reaches for method syntax when the C++ names a receiver -- CDefView's
    `DestroyViewWindow` did exactly that, one site, in an otherwise faithful body.

    Mechanical and unambiguous, so the tool owns it rather than asking the prompt to. Scoped
    to callees the CWindow header proves are window methods, which no COM interface method can
    be, and applied only to a site whose receiver parenthesis is balanced and whose argument
    list is then short by exactly the handle.
    """
    if not any(call.hwnd_receiver for call in sealed.calls):
        return rust, []
    windowed = {call.placeholder for call in sealed.calls if call.hwnd_receiver}
    repaired: list[str] = []
    for call in sealed.calls:
        if call.placeholder not in windowed or call.argc is None:
            continue
        while True:
            masked = _code_mask(rust)
            match = re.search(rf"\(\s*\*\s*(?P<recv>.+?)\)\s*\.\s*{call.placeholder}\b", masked)
            if match is None:
                break
            receiver = rust[match.start("recv") : match.end("recv")]
            if receiver.count("(") != receiver.count(")"):
                break
            counted = _argument_count(masked, match.end())
            if counted is None or counted[0] != call.argc - 1:
                break
            opening = masked.find("(", match.end())
            separator = ", " if counted[0] else ""
            rust = (
                rust[: match.start()]
                + call.placeholder
                + rust[match.end() : opening + 1]
                + receiver
                + separator
                + rust[opening + 1 :]
            )
            repaired.append(f"{call.placeholder}={call.original}")
            break
    # The implicit form of the same omission: a BARE member call short by exactly the handle.
    # `ScreenToClient(&pt)` inside a CWindowImpl class is `this->m_hWnd`'s window, and the model
    # writes the C++ arity even when it has the extern declaration in front of it -- twice, in
    # two independent samples of CDefView_OnContextMenu. The receiver can only be `this->m_hWnd`
    # here (an explicit one is handled above, a qualified `::` call is not a member at all), so
    # inserting it is a mechanical lowering rather than a guess, exactly as
    # `repair_dropped_receivers` inserts the sibling `this` it is the analogue of.
    masked = _code_mask(rust)
    for call in sealed.calls:
        if call.placeholder not in windowed or call.argc is None:
            continue
        for match in re.finditer(rf"\b{re.escape(call.placeholder)}\b", masked):
            if masked[: match.start()].rstrip().endswith((".", ">")):
                continue
            counted = _argument_count(masked, match.end())
            if counted is None or "::<" in counted[1] or counted[0] != call.argc - 1:
                continue
            if re.match(r"\s*\(\s*\*\s*this\s*\)\s*\.\s*m_hWnd\b", counted[1]):
                continue
            opening = masked.find("(", match.end())
            insertion = "(*this).m_hWnd" + (", " if counted[0] else "")
            rust = rust[: opening + 1] + insertion + rust[opening + 1 :]
            repaired.append(f"{call.placeholder}={call.original}")
            masked = _code_mask(rust)
            break
    return rust, sorted(set(repaired))


def unseal_calls(rust: str, sealed: SealedCalls, *, dead_omissions: list[str] | None = None) -> str:
    """Verify the closed call set and restore the callee names.

    `dead_omissions`, when given, collects `TOKEN=CALLEE` for each sealed call the model
    omitted that lives only in non-executed code -- accepted, but reported.
    """
    rust = scrub_comment_token_echoes(rust)
    rust, _repaired = repair_dropped_receivers(rust, sealed)
    rust, _windowed = repair_window_receivers(rust, sealed)
    expected = {call.placeholder for call in sealed.calls}
    masked = _code_mask(rust)
    executable = _PLACEHOLDER_RE.findall(masked)
    missing_tokens = expected - set(executable)
    allowed_duplicates: set[str] = set()

    # A model sometimes sees a generated sibling-worker declaration in context
    # and writes that exact callee name instead of preserving its opaque token.
    # Recover only the mechanically unambiguous case: every missing token for one
    # callee has exactly one matching executable direct-call identifier.  Comments,
    # declarations, duplicates, and approximate spellings remain hard failures.
    missing_by_original: dict[str, list[SealedCall]] = {}
    for call in sealed.calls:
        if call.placeholder in missing_tokens:
            missing_by_original.setdefault(call.original, []).append(call)
    replacements: list[tuple[int, int, str]] = []
    for original, calls in missing_by_original.items():
        matches = []
        spellings = (original, f"{original}_body")
        pattern = "|".join(re.escape(spelling) for spelling in spellings)
        for match in re.finditer(rf"\b(?:{pattern})\s*(?=\()", masked):
            prefix = masked[max(0, match.start() - 8) : match.start()]
            if re.search(r"\bfn\s*$", prefix):
                continue
            matches.append(match)
        if len(matches) == len(calls):
            replacements.extend(
                (match.start(), match.end(), call.placeholder)
                for match, call in zip(matches, calls, strict=True)
            )
    for start, end, placeholder in sorted(replacements, reverse=True):
        rust = rust[:start] + placeholder + rust[end:]

    # Placeholder identity distinguishes source occurrences, not callee
    # semantics. If a model repeats one token while preserving the exact total
    # executable call count for that same callee, deterministically relabel the
    # occurrences in output order. Arguments and control flow remain untouched.
    for original in dict.fromkeys(call.original for call in sealed.calls):
        family = [call for call in sealed.calls if call.original == original]
        if len(family) < 2:
            continue
        family_names = {call.placeholder for call in family}
        masked = _code_mask(rust)
        matches = [
            match
            for match in _PLACEHOLDER_RE.finditer(masked)
            if match.group(0) in family_names and re.match(r"\s*\(", masked[match.end() :])
        ]
        if len(matches) != len(family):
            continue
        for match, call in reversed(list(zip(matches, family, strict=True))):
            rust = rust[: match.start()] + call.placeholder + rust[match.end() :]

    # C switch fallthrough becomes duplicated syntax in structured Rust when
    # the shared tail has side effects. Accept that one-to-many spelling only
    # for two byte-identical calls in mutually exclusive ``if/else if`` arms;
    # all other duplicated opaque calls remain a hard failure.
    if "switch" in sealed.text and "case" in sealed.text:
        masked = _code_mask(rust)
        for token in expected:
            occurrences = list(re.finditer(rf"\b{re.escape(token)}\s*\(", masked))
            if len(occurrences) != 2:
                continue
            calls = []
            for occurrence in occurrences:
                opening = masked.find("(", occurrence.start())
                depth = 0
                closing = -1
                for pos in range(opening, len(masked)):
                    if masked[pos] == "(":
                        depth += 1
                    elif masked[pos] == ")":
                        depth -= 1
                        if depth == 0:
                            closing = pos
                            break
                if closing < 0:
                    break
                calls.append(re.sub(r"\s+", "", masked[occurrence.start() : closing + 1]))
            between = masked[occurrences[0].end() : occurrences[1].start()]
            if len(calls) == 2 and calls[0] == calls[1] and re.search(r"}\s*else\s+if\b", between):
                allowed_duplicates.add(token)

    # Cheap models occasionally common-subexpression-eliminate a repeated pure
    # string-length call: ``let len = TOKEN_A(arg); ... len ... len``. The C
    # call envelope intentionally preserves occurrences, so restore TOKEN_B at
    # the final reuse only when the source calls the same proven-pure function,
    # exactly one token is missing, and the Rust binding/uses are unambiguous.
    masked = _code_mask(rust)
    executable_tokens = set(_PLACEHOLDER_RE.findall(masked))
    ordered = list(sealed.calls)
    for original in _PURE_REUSABLE_CALLS:
        family = [call for call in ordered if call.original == original]
        missing_family = [call for call in family if call.placeholder not in executable_tokens]
        if len(missing_family) != 1:
            continue
        missing_call = missing_family[0]
        missing_index = ordered.index(missing_call)
        prior = next(
            (
                call
                for call in reversed(ordered[:missing_index])
                if call.original == original and call.placeholder in executable_tokens
            ),
            None,
        )
        if prior is None:
            continue
        binding = re.search(
            rf"\blet\s+(?:mut\s+)?(?P<name>[A-Za-z_]\w*)"
            rf"(?:\s*:[^=;]+)?\s*=\s*{re.escape(prior.placeholder)}\s*"
            r"\(\s*(?P<arg>[^(),;]+?)\s*\)[^;]*;",
            masked,
        )
        if binding is None:
            continue
        name = binding.group("name")
        uses = list(re.finditer(rf"\b{re.escape(name)}\b", masked[binding.end() :]))
        if len(uses) < 2:
            continue
        use = uses[-1]
        start = binding.end() + use.start()
        end = binding.end() + use.end()
        arg = rust[binding.start("arg") : binding.end("arg")].strip()
        rust = rust[:start] + f"{missing_call.placeholder}({arg})" + rust[end:]
        masked = _code_mask(rust)
        executable_tokens.add(missing_call.placeholder)

    # Occurrences inside a forward `extern "..." { … }` declaration are not calls. The
    # mechanical contract REQUIRES those declarations, so they must not count toward
    # "exactly once" — but they must not satisfy it either, or a model could declare a
    # callee it never calls. Blank the declaration spans and judge both checks on
    # what is left, so a token still has to appear exactly once in executable code.
    exec_mask = list(_code_mask(rust))
    for a, b in extern_declaration_spans(rust):
        exec_mask[a:b] = " " * (b - a)
    executable = _PLACEHOLDER_RE.findall("".join(exec_mask))
    replicable = _replicable_after_goto(sealed)
    exec_text = "".join(exec_mask)
    absent = {
        token
        for token in expected - set(executable)
        if not _name_satisfies(token, sealed, replicable, exec_text)
    }
    # Of what is genuinely absent, call sites the build never ENTERS are permitted to be
    # absent rather than demanded -- see dead_regions.py. Orthogonal to `_name_satisfies`
    # above, which concerns a call the model wrote by NAME instead of by token: that one is
    # present-but-unsealed, this one is legitimately gone. Both are reported, never silent.
    optional = {call.placeholder for call in sealed.calls if call.dead}
    omitted_dead = absent & optional
    if omitted_dead and dead_omissions is not None:
        dead_omissions.extend(
            f"{call.placeholder}={call.original}"
            for call in sealed.calls
            if call.placeholder in omitted_dead
        )
    missing = sorted(absent - optional)
    duplicates = sorted(
        token
        for token in expected
        if executable.count(token) > 1
        and token not in allowed_duplicates
        and token not in replicable
    )
    all_present = _PLACEHOLDER_RE.findall(rust)
    unknown = sorted(set(all_present) - expected)
    findings = []
    if missing:
        findings.append("missing/non-executable: " + ", ".join(missing))
    if duplicates:
        findings.append("not exactly once: " + ", ".join(duplicates))
    if unknown:
        findings.append("invented: " + ", ".join(unknown))
    shifted = _arity_disagreements(exec_text, sealed)
    if shifted:
        findings.append("wrong call site (argument count): " + ", ".join(shifted))
    if findings:
        raise CallSealError(
            "model damaged deterministic call envelope (" + "; ".join(findings) + ")"
        )
    for call in sealed.calls:
        rust = rust.replace(call.placeholder, call.original)
    doubled = _doubled_after_unseal(rust, sealed)
    if doubled:
        raise CallSealError(
            "model damaged deterministic call envelope (callee written BOTH by name and by "
            "token, so unsealing produced NAME(NAME(..)): " + ", ".join(doubled) + ")"
        )
    return rust
