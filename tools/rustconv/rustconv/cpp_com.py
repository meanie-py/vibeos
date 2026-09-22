"""Deterministic C++/ATL COM-class analysis and Rust scaffold generation.

The language model translates method *bodies*.  It must never invent COM ABI:
interface slot order, vtable signatures, QueryInterface routing, interface-to-object
offset recovery, refcounting, or activation metadata.  This module derives that
surface from two compiler-facing sources:

* the class header (``BEGIN_COM_MAP`` is the authoritative IID/interface map), and
* WIDL's generated C header (the authoritative flattened vtable shape).

Only the deliberately small, proven subset is accepted.  Unsupported ATL map
entries or state fields are reported and make ``require_supported`` fail loudly.
"""

from __future__ import annotations

import ast
import hashlib
import json
import re
from collections import Counter, defaultdict
from collections.abc import Mapping, Sequence
from dataclasses import asdict, dataclass, field, replace
from pathlib import Path
from typing import TYPE_CHECKING

from .cpp_core import (
    _RUST_SCALARS,
    CppTranslationError,
    _abi_c_type_key,
    _abi_types_compatible,
    _argument_c_type,
    _cpp_literal_spans,
    _eval_c_int_constexpr,
    _expand_default_helper_arguments,
    _masked_cpp,
    _matching_paren,
    _named_call_spans,
    _numeric_array_constants,
    _sdk_numeric_definitions,
    _split_args,
    c_type_to_rust,
    integer_cannot_bind_pointer,
    is_c_integer,
)
from .fact_seal import _named_cast_spans

if TYPE_CHECKING:
    from .callgraph import FunctionDep


# The historical spelling, and the one every caller uses. Same class, not a subclass:
# `except CppComError` must still catch what the generic C++ layer raises, since the
# generic layer is where most of the deterministic proofs now live.
CppComError = CppTranslationError


@dataclass(frozen=True)
class ComMapEntry:
    iid: str
    interface: str
    macro: str


@dataclass(frozen=True)
class InterfaceMethod:
    name: str
    return_type: str
    params: tuple[tuple[str, str], ...]


@dataclass(frozen=True)
class InterfaceSpec:
    name: str
    methods: tuple[InterfaceMethod, ...]
    source: str
    base: str | None = None


# Win32 pointer typedefs whose spelling doesn't match the L?P/H prefix convention
# (function-pointer callback types are still plain C pointer state).
_KNOWN_POINTER_TYPEDEFS = frozenset(
    {"WNDPROC", "DLGPROC", "FARPROC", "PROC", "TIMERPROC", "HOOKPROC"}
)


# ATL CONTAINER members whose Rust lowering already exists in `shell32-rust/src/atl.rs`.
#
# `CComPtr` and `CComHeapPtr` are handled a layer down as raw pointers, because the
# generated Release can express their whole destructor in one line. A container cannot be
# flattened that way — its destructor walks storage the scaffold does not own — so the
# field keeps the container TYPE and its `Drop` does the work.
#
# Being absent from this table is not neutral, exactly as for `_LOCAL_RAII_TYPES`: an
# unlisted container is a hard pre-payment refusal naming the type, because the
# alternative is the model inventing an API for it. `CAtlList` was added with its Rust
# side; do not list a type before `atl.rs` implements it.
#
# `usage` is handed to every worker whose body touches the field. C++ container spellings
# do not survive translation on their own — `m_Storage[n]` has no Rust meaning, and a
# model asked to guess produced `.0`-style wrapper artifacts before now.
#: `Tmpl<Elem> name` or `Tmpl<Elem*> name` as an object state field.
#:
#: ONE definition, used by both the state-field parser and `_discover_pod_state_types`. They
#: carried identical copies, which is a standing invitation to teach one of them a shape the
#: other still rejects -- exactly what a pointer element did: discovery could not see
#: `CWatchItem` inside `CSimpleArray<CWatchItem*>`, so the parser then refused the element as
#: "not a proven layout" and the whole TU read as untranslatable.
_RAII_CONTAINER_FIELD_RE = re.compile(
    r"\s*(?P<template>[A-Za-z_]\w*)\s*<\s*(?P<element>[A-Za-z_]\w*)\s*(?P<ptr>\*?)\s*>"
    r"\s+(?P<name>[A-Za-z_]\w*)\s*"
)

_RAII_CONTAINER_LOWERINGS: dict[str, tuple[str, str]] = {
    "CSimpleArray": (
        "CSimpleArray::new()",
        "`Add(t) -> BOOL` (FALSE on allocation failure — check it, the C++ does), "
        "`GetSize() -> i32`, `GetData() -> *mut T`, `GetAt(i32) -> *mut T`, "
        "`RemoveAll()`. C++ `arr[n]` is `(*arr.GetAt(n))` and C++ `&arr[n]` is "
        "`arr.GetAt(n)`.",
    ),
    "CAtlList": (
        "CAtlList::new()",
        "`AddHead(t)` / `AddTail(t)`, `IsEmpty() -> bool`, `GetCount() -> usize`, "
        "`RemoveAll()`, `GetHeadPosition() -> POSITION`, "
        "`GetNext(&mut POSITION) -> *mut T` (returns the CURRENT element and advances "
        "the position, exactly like ATL), `FindIndex(i32) -> POSITION`, "
        "`GetAt(POSITION) -> *mut T`. A `POSITION` is null at the end of the list.",
    ),
    # Not a container and not a template — ATL's owning BSTR wrapper (atlbase.h), which is
    # one `m_str: BSTR` member plus a destructor that calls SysFreeString. It rides this
    # table because what it needs is exactly what a container needs: keep the field's TYPE
    # so `Drop` can reproduce the member destructor the translation would otherwise lose.
    # The bindgen island already renders it as `ATL_CComBSTR { m_str: BSTR }`, so the Rust
    # type in atl.rs has the C++ layout for free.
    "CComBSTR": (
        "CComBSTR::new()",
        "ONE public field, `m_str: BSTR`, and a `Drop` that calls `SysFreeString`. "
        "C++ `m_name.m_str = p` is an ADOPT of an already-allocated BSTR — lower it as "
        "the same single store `(*this).m_name.m_str = p;`, never as a copy, or the "
        "ownership transfer changes and the string is leaked or freed twice. C++ reads "
        "that pass the object where a `BSTR` is expected (ATL's `operator BSTR`) are "
        "`(*this).m_name.m_str`.",
    ),
}


def _no_viable_conversion(
    actual_key: str, expected_key: str, field_types: dict[str, dict[str, str]] | None
) -> bool:
    """Whether C++ has NO conversion from ``actual_key`` to ``expected_key``.

    Only proven impossibilities count. A candidate this rejects was never in the
    compiler's overload set, so rejecting it can never change which method a
    well-formed call resolves to -- it only breaks ties that scoring cannot.
    """
    if integer_cannot_bind_pointer(actual_key, expected_key):
        return True
    # An integer does not convert to a C aggregate, by value or by reference: a plain
    # C struct has no converting constructor. `_PidlByItem(LRESULT)` against
    # `_PidlByItem(LVITEM&)` is CDefView's case.
    referent = expected_key.rstrip("*& ")
    return bool(
        field_types
        and "*" not in expected_key
        and is_c_integer(actual_key)
        and referent in field_types
    )


def _shapes_provably_disagree(
    source_params: Sequence[tuple[str, str]],
    vtable_params: Sequence[tuple[str, str]],
    type_aliases: dict[str, str] | None = None,
) -> bool:
    """True when some parameter pair has no conversion in either direction.

    Used to constrain an ARITY-only vtable match. Deliberately one-sided: it reports a
    disagreement only for a pair it can prove (an integer against a pointer), so a merely
    unfamiliar spelling still binds exactly as before.
    """
    for (source_type, _), (vtable_type, _) in zip(source_params, vtable_params, strict=True):
        left = _canonical_sdk_type(source_type, type_aliases)
        right = _canonical_sdk_type(vtable_type, type_aliases)
        if integer_cannot_bind_pointer(left, right) or integer_cannot_bind_pointer(right, left):
            return True
    return False


def _canonical_sdk_type(c_type: str, type_aliases: dict[str, str] | None) -> str:
    """Resolve an SDK macro type alias, ignoring a C++ reference and pointer suffix.

    Both sides of an overload comparison need this: a struct field is typed from the
    island (`LVITEMW`) while the candidate parameter is spelled with the macro the
    source uses (`LVITEM&`). Comparing those raw is a miss, and a miss on EVERY
    candidate is reported as an ambiguity — the failure this resolves.
    """
    if not type_aliases:
        return c_type
    text = " ".join(c_type.split())
    suffix = ""
    while text.endswith(("*", "&")):
        suffix = ("*" if text.endswith("*") else "") + suffix
        text = text[:-1].strip()
    return f"{type_aliases.get(text, text)}{(' ' + suffix) if suffix else ''}"


def _bind_sibling_overload(
    body: str,
    start: int,
    end: int,
    candidates: list[FunctionDep],
    caller: FunctionDep,
    state_fields: list[PlainStateField],
    class_name: str,
    source_name: str,
    call_returns: dict[str, str] | None = None,
    field_types: dict[str, dict[str, str]] | None = None,
    type_aliases: dict[str, str] | None = None,
) -> FunctionDep:
    """Bind an implicit ``this`` call using source-visible argument types."""
    if len(candidates) == 1:
        return candidates[0]
    opening = body.find("(", end)
    closing = _matching_paren(_masked_cpp(body), opening)
    arguments = _split_args(body[opening + 1 : closing])
    symbols = {name.lstrip("&*"): c_type for c_type, name in caller.params if name}
    symbols.update({field.name: field.c_type for field in state_fields})
    prefix = body[:start]
    for declaration in re.finditer(
        r"\b(?P<type>[A-Za-z_]\w*(?:\s*\*)*)\s*(?P<name>[A-Za-z_]\w*)\s*"
        r"(?P<array>\[[^]]*\])?\s*(?:=|;)",
        prefix,
    ):
        c_type = " ".join(declaration.group("type").split())
        if declaration.group("array"):
            if c_type == "WCHAR":
                c_type = "LPWSTR"
            elif c_type == "CHAR":
                c_type = "LPSTR"
        symbols.setdefault(declaration.group("name"), c_type)
    for com_ptr in re.finditer(
        r"\bCComPtr\s*<\s*(?P<interface>I[A-Za-z0-9_]*)\s*>\s+"
        r"(?P<name>[A-Za-z_]\w*)\b",
        prefix,
    ):
        symbols[com_ptr.group("name")] = f"{com_ptr.group('interface')} *"
    for argument in arguments:
        identifier = re.fullmatch(r"(?:this\s*->\s*)?([A-Za-z_]\w*)", argument.strip())
        if identifier is None or identifier.group(1) in symbols:
            continue
        name = identifier.group(1)
        com_ptr = re.search(
            rf"\bCComPtr\s*<\s*(?P<interface>I[A-Za-z0-9_]*)\s*>\s+"
            rf"{re.escape(name)}\b",
            prefix,
        )
        if com_ptr is not None:
            symbols[name] = f"{com_ptr.group('interface')} *"
            continue
        declaration = re.search(
            rf"\b(?P<type>[A-Za-z_]\w*(?:\s*\*)*)\s*{re.escape(name)}\s*"
            rf"(?P<array>\[[^]]*\])?\s*(?:=|;)",
            prefix,
        )
        if declaration is not None:
            c_type = " ".join(declaration.group("type").split())
            if declaration.group("array"):
                if c_type == "WCHAR":
                    c_type = "LPWSTR"
                elif c_type == "CHAR":
                    c_type = "LPSTR"
            symbols[name] = c_type

    scores: list[tuple[int, FunctionDep]] = []
    viable: list[FunctionDep] = []
    for candidate in candidates:
        score = 0
        rejected = False
        for argument, (expected, _name) in zip(arguments, candidate.params, strict=True):
            actual = _argument_c_type(argument, symbols, call_returns, field_types, type_aliases)
            if actual is None:
                continue
            actual_key = _canonical_sdk_type(actual, type_aliases)
            expected_key = _canonical_sdk_type(expected, type_aliases)
            if _abi_types_compatible(actual_key, expected_key):
                score += 1
            elif _no_viable_conversion(actual_key, expected_key, field_types):
                # Not merely a worse match -- no such conversion exists, so C++ never
                # had this candidate in the set at all. Scoring alone could not separate
                # `f(int)` from `f(PCUITEMID_CHILD)` for a UINT argument, because an
                # exact match is absent from BOTH.
                rejected = True
        if not rejected:
            viable.append(candidate)
        scores.append((score, candidate))
    if len(viable) == 1 and len(viable) < len(candidates):
        return viable[0]
    if scores:
        best = max(score for score, _ in scores)
        winners = [candidate for score, candidate in scores if score == best]
        if best > 0 and len(winners) == 1:
            return winners[0]
    choices = ", ".join(
        f"{candidate.name}({', '.join(c_type for c_type, _ in candidate.params)})"
        for candidate in candidates
    )
    raise CppComError(
        f"{class_name}: cannot bind overloaded sibling call {source_name}"
        f"({', '.join(argument.strip() for argument in arguments)}) in {caller.name}; "
        f"candidates: {choices or '<none>'}"
    )


@dataclass(frozen=True)
class PlainStateField:
    """One C-layout object field safe to spell directly in the Rust owner."""

    declaration: str
    c_type: str
    name: str
    com_ptr_interface: str | None = None
    owned_cstring: bool = False
    # CComHeapPtr<T> member: raw *mut T in the Rust owner, CoTaskMemFree'd by the
    # generated Release when still non-null (the C++ member dtor's implicit free).
    owned_heap_ptr: bool = False
    array_len: int | None = None
    # `Tmpl<Elem>` from _RAII_CONTAINER_LOWERINGS: the field keeps the container type and
    # its `Drop` restores the C++ member destructor the translation dropped.
    raii_container: str | None = None

    @property
    def rust_type(self) -> str:
        if self.raii_container is not None:
            return self.raii_container
        if self.com_ptr_interface is not None:
            rust = f"*mut {self.com_ptr_interface}"
        else:
            rust = c_type_to_rust(self.c_type)
        return f"[{rust}; {self.array_len}]" if self.array_len is not None else rust

    @property
    def owns_resource(self) -> bool:
        """True when the C++ member has a non-trivial destructor of its own."""
        return (
            self.com_ptr_interface is not None
            or self.owned_cstring
            or self.owned_heap_ptr
            or self.raii_container is not None
        )

    @property
    def owned_rust_type(self) -> str:
        """The field spelling where RUST DROP GLUE owns the lifetime.

        This differs from `rust_type`, which is the COM *owner's* spelling: an owner's
        ATL members are released by explicitly generated code in `Release`, so a raw
        `*mut T` there is complete. A POD has no generated code at all — its only
        destructor is the drop glue Rust derives from its field types — so an ATL member
        flattened to a raw pointer here is a leak with nothing red anywhere. Keeping the
        owning type makes Rust's field-drop order reproduce C++'s implicit destructor.
        Every type used is `repr(transparent)` over the same pointer, so layout is
        unchanged.
        """
        if self.raii_container is not None:
            return self.raii_container
        if self.com_ptr_interface is not None:
            rust = f"CComPtr<{self.com_ptr_interface}>"
        elif self.owned_heap_ptr:
            rust = f"CComHeapPtr<{c_type_to_rust(self.c_type).removeprefix('*mut ').strip()}>"
        else:
            return self.rust_type
        return f"[{rust}; {self.array_len}]" if self.array_len is not None else rust

    @property
    def init_expression(self) -> str:
        """The struct-literal initialiser. Every other field is `zeroed()`, which is the
        C++ default ctor for a POD and is documented-valid for atl.rs's pointer types;
        a container has a real `new()` and must use it."""
        if self.raii_container is not None:
            template = self.raii_container.split("<", 1)[0]
            return _RAII_CONTAINER_LOWERINGS[template][0]
        if self.owned_cstring:
            return "CStringW::new()"
        return "core::mem::zeroed()"

    @property
    def assignment_helper(self) -> str | None:
        if self.com_ptr_interface is None:
            return None
        return f"cpp_com_assign_{self.name}"

    @property
    def string_assignment_helper(self) -> str | None:
        if not self.owned_cstring:
            return None
        return f"cpp_com_assign_{self.name}"


@dataclass(frozen=True)
class PodStateType:
    """A source-proven C POD embedded by value in the COM owner."""

    name: str
    fields: tuple[PlainStateField, ...]
    packed: bool = False

    @property
    def owns_resources(self) -> bool:
        """True when this struct's own fields give it a non-trivial C++ destructor."""
        return any(state.owns_resource for state in self.fields)


def pods_with_drop(pods: Sequence[PodStateType]) -> frozenset[str]:
    """POD names whose Rust struct has real drop glue — directly, or transitively through
    a by-value field of another such POD.

    C++'s implicit destructor is transitive and so is Rust's, so a guard that looked only
    one level deep would wave through a wrapper struct and leak everything inside it.
    """
    names = {pod.name for pod in pods if pod.owns_resources}
    changed = True
    while changed:
        changed = False
        for pod in pods:
            if pod.name in names:
                continue
            for state in pod.fields:
                # A POINTER to a POD owns nothing — C++ destroys neither end. Only a
                # by-value member (an array of them included) runs the element destructor.
                if "*" in state.c_type:
                    continue
                if state.c_type.strip() in names:
                    names.add(pod.name)
                    changed = True
                    break
    return frozenset(names)


@dataclass(frozen=True)
class EnumStateType:
    """A source-proven nested C/C++ enum used by object-owned state."""

    name: str
    variants: tuple[tuple[str, int], ...]


@dataclass
class CppComClass:
    name: str
    source: Path
    header: Path
    com_map: list[ComMapEntry] = field(default_factory=list)
    registry_resource: str | None = None
    not_aggregatable: bool = False
    object_map_clsid: str | None = None
    # Exact shellutils.h `ShellObjectCreator<T>` construction uses `_CComObject<T>`:
    # a private, non-aggregating 0-ref allocation followed by AddRef/QI/Release.
    # It proves internal activation but never authorizes an OBJECT_ENTRY adapter.
    internal_shell_object_creator: bool = False
    # Derives ATL's `CComObjectRootEx<>`, which carries `m_dwRef` and the
    # InternalAddRef/InternalRelease pair — so the object IS refcounted even when it exposes no
    # COM interface at all. CMenuFocusManager is exactly that: no BEGIN_COM_MAP, but
    # `obj->AddRef()` / `if (!obj->Release())` in AcquireManager/ReleaseManager.
    atl_refcounted: bool = False
    # ShellObjectCreatorInit<T>(args..., riid, ppv) must initialize before QI.
    # None is the ordinary creator; a tuple is the proved Initialize surface.
    internal_initializer_params: tuple[tuple[str, str], ...] | None = None
    stgmedium_lifecycle_fields: tuple[str, ...] = ()
    # C++ `virtual` methods the class declares itself. A COM class has these by definition
    # and they are handled by the WIDL vtable machinery; the field exists for the --plain
    # path, where a vtable is exactly what is being asserted NOT to exist.
    virtual_methods: tuple[str, ...] = ()
    # Parameters of the PRIMARY constructor -- the one the scaffold names `<Class>_ctor_body`.
    # Empty for the ordinary default-constructed class. A plain helper whose ONLY constructor
    # takes arguments (`CDirectoryWatcher(HWND, LPCWSTR, BOOL)`) needs them on `_new`, which
    # otherwise emits a call passing one argument to a four-argument body.
    ctor_params: tuple[tuple[str, str], ...] = ()
    final_construct: bool = False
    method_names: list[str] = field(default_factory=list)
    base_classes: list[str] = field(default_factory=list)
    state_fields: list[str] = field(default_factory=list)
    pod_state_types: list[PodStateType] = field(default_factory=list)
    enum_state_types: list[EnumStateType] = field(default_factory=list)
    array_constants: dict[str, int] = field(default_factory=dict)
    unsupported: list[str] = field(default_factory=list)
    # Base interfaces the class implements but does NOT QI-expose (absent from the
    # COM map), e.g. IContextMenuCB on CFSFolder. They contribute a vtable subobject
    # in the object layout that the class hands out via static_cast, but they are
    # never returned by QueryInterface.
    layout_bases: list[str] = field(default_factory=list)
    # Concrete non-template base classes defined in the same header as this class
    # (e.g. CCopyMoveToMenu under CMoveToMenu), flattened into this translation:
    # their state fields precede ours, their methods become methods of this class
    # (renamed at scan-extend time), and their virtual calls devirtualize onto our
    # overrides through the ordinary same-file arity binder.
    local_base_classes: list[str] = field(default_factory=list)
    # Non-trivial concrete-base destructors run after the derived destructor,
    # in C++ base-destruction order. Scan extension renames them into ordinary
    # workers and records those worker names here for Release sequencing.
    base_dtor_workers: list[str] = field(default_factory=list)
    # SDK type names the per-arch bindgen islands agree are plain integer aliases
    # (see island_integer_aliases). Empty when no islands were supplied, which keeps
    # an unproven type a loud preflight failure rather than an assumed int.
    island_scalar_types: frozenset[str] = frozenset()
    # Struct names the islands agree on (see island_struct_types). Used only to prove an
    # ATL container's element type is a C struct — i.e. trivially destructible.
    island_struct_types: frozenset[str] = frozenset()
    # Field C types the islands agree on, per struct (see island_struct_field_types).
    # Lets the overload binder type a member-access argument. Empty when no islands were
    # supplied, which leaves such an argument untyped — i.e. an ambiguity, not a guess.
    island_struct_field_types: dict[str, dict[str, str]] = field(default_factory=dict)
    # SDK macro type aliases (see sdk_type_macro_aliases), used to canonicalize BOTH
    # sides of an overload comparison: the island types a field `LVITEMW` while the C++
    # candidate spells the same type `LVITEM&`.
    sdk_type_aliases: dict[str, str] = field(default_factory=dict)

    def interface_names(self) -> list[str]:
        """Ordered, de-duplicated interfaces to materialize: QI'd (com_map) then
        layout-only bases. Both need a vtable subobject; only the former is QI'd."""
        return list(
            dict.fromkeys([*(entry.interface for entry in self.com_map), *self.layout_bases])
        )

    def plain_state_fields(self) -> list[PlainStateField]:
        pod_names = {item.name for item in self.pod_state_types}
        return [
            _parse_plain_state_field(
                item,
                allowed_pod_types=pod_names,
                array_constants=self.array_constants,
                island_scalar_types=self.island_scalar_types,
                island_struct_types=self.island_struct_types,
            )
            for item in self.state_fields
        ]

    def require_supported(self, *, allow_internal: bool = False, allow_plain: bool = False) -> None:
        """Refuse a class whose ABI surface is not fully proven.

        ``allow_plain`` is for a file-local C++ helper class that is not COM at all — no
        vtable, no QI, no activation, reached only by direct call from its own TU. All three
        COM requirements below are then vacuous, and demanding them refused a class that is
        strictly SIMPLER than the supported case. The state layout, RAII, and STGMEDIUM
        checks still apply: those are about owning memory correctly, which a plain class does
        just as much as a COM one.

        A plain class is asserted, never inferred: if the caller says ``allow_plain`` and the
        class turns out to HAVE a com_map, that is a mis-invocation, not a licence to skip the
        vtable — refuse rather than silently emit a COM class with no interfaces.
        """
        problems = list(self.unsupported)
        if allow_plain:
            if self.com_map:
                problems.append(
                    f"--plain was passed but {self.name} has BEGIN_COM_MAP entries "
                    f"({', '.join(entry.interface for entry in self.com_map)}); it is a COM "
                    "class — drop --plain so its vtable and activation are generated"
                )
            # A COM map is not the only thing that makes a vtable, and `--plain` does not
            # actually mean "no vtable" -- a HAND-ROLLED COM class (interfaces, no
            # BEGIN_COM_MAP) uses this path, and the scaffold renders its WIDL vtables and
            # stores the vptr. What has no model at all is a BESPOKE C++ virtual hierarchy:
            # virtuals belonging to no interface, so there is no WIDL declaration to derive
            # a slot order from. The scaffold then emits a struct with no vptr, every field
            # lands at the wrong offset, and overrides silently bind to the base -- none of
            # which is a compile error in either language.
            #
            # The discriminator is therefore an INTERFACE behind the virtuals, not the
            # presence of virtuals: `RecycleBinGeneric` derives `IRecycleBin` (a WIDL
            # interface, recorded in `layout_bases`) and is correctly translatable, while
            # `CMenuToolbarBase` derives only `CWindowImplBaseT` and declares 24 virtuals of
            # its own -- six PURE, with two concrete subclasses overriding them.
            #
            # A virtual DESTRUCTOR alone never trips this: it adds no callable slot the
            # scaffold must name, and it rides whatever vtable the interfaces already give.
            bespoke_virtuals = sorted(
                {name for name in self.virtual_methods if not name.startswith("~")}
            )
            if bespoke_virtuals and not self.com_map and not self.layout_bases:
                shown = ", ".join(bespoke_virtuals[:6])
                problems.append(
                    f"--plain was passed but {self.name} declares virtual method(s) "
                    f"({shown}) belonging to no interface; it HAS a vtable, and a plain "
                    "struct would place every field at the wrong offset and drop virtual "
                    "dispatch. A non-COM C++ virtual hierarchy needs a deterministic "
                    "vtable mapping"
                )
        else:
            if not self.com_map:
                problems.append("no BEGIN_COM_MAP entries found")
            internal_ok = allow_internal and self.internal_shell_object_creator
            if not self.not_aggregatable and not internal_ok:
                problems.append(
                    "aggregation policy is unsupported: DECLARE_NOT_AGGREGATABLE is required"
                )
            if not self.object_map_clsid and not internal_ok:
                problems.append(
                    "no OBJECT_ENTRY activation found; internal COM-object construction is "
                    "not mapped"
                )
        for declaration in self.state_fields:
            match = re.fullmatch(r"\s*STGMEDIUM\s+([A-Za-z_]\w*)\s*", declaration)
            if match and match.group(1) not in self.stgmedium_lifecycle_fields:
                problems.append(
                    "bindgen-backed STGMEDIUM state lacks source-proven constructor/"
                    f"destructor lifecycle: {declaration}"
                )
        for pod in self.pod_state_types:
            if pod.packed and pod.owns_resources:
                # rustc refuses to drop a field of a packed struct (it cannot take a
                # reference to a misaligned place), so this would not even compile —
                # but saying so here names the C++ construct instead of dumping E0793.
                problems.append(
                    f"packed struct `{pod.name}` has a member with a destructor "
                    f"({', '.join(s.declaration for s in pod.fields if s.owns_resource)}); "
                    "Rust cannot drop a packed field"
                )
        for declaration in self.state_fields:
            try:
                _parse_plain_state_field(
                    declaration,
                    allowed_pod_types={item.name for item in self.pod_state_types},
                    array_constants=self.array_constants,
                    island_scalar_types=self.island_scalar_types,
                    island_struct_types=self.island_struct_types,
                )
            except CppComError as exc:
                problems.append(str(exc))
        if problems:
            raise CppComError(
                f"{self.name}: deterministic COM scaffold preflight failed:\n  - "
                + "\n  - ".join(problems)
            )

    def to_json(self) -> str:
        data = asdict(self)
        for key in ("source", "header"):
            data[key] = str(data[key])
        return json.dumps(data, indent=2, sort_keys=True)


_CLASS_HEAD = re.compile(r"\b(?:class|struct)\s+(?P<name>[A-Za-z_]\w*)\s*(?P<bases>:[^{]+)?\{")
_BEGIN_MAP = re.compile(r"\bBEGIN_COM_MAP\s*\(\s*(\w+)\s*\)")
_END_MAP = re.compile(r"\bEND_COM_MAP\s*\(\s*\)")
_MAP_ENTRY = re.compile(r"\b(COM_INTERFACE_ENTRY(?:2)?(?:_IID)?)\s*\(([^)]*)\)")
_UNSUPPORTED_MAP = re.compile(
    r"\b(COM_INTERFACE_ENTRY_(?:CHAIN|FUNC|FUNC_BLIND|AGGREGATE|BREAK|NOINTERFACE))\s*\("
)
# A method DECLARATION, consumed whole. Two prefixes must be inside the match or their
# leftovers merge with the next real field into a bogus one: the `static` storage specifier,
# and a POINTER return type (`SApp *AddInternal(...)` used to match only from `AddInternal`,
# stranding `SApp *`). COpenWithList's private section has both, and produced the field
# `SApp * static static static static` — which would have been a silent struct-layout change
# had it happened to parse.
_METHOD_DECL = re.compile(
    r"(?:static\s+|inline\s+|explicit\s+)*"
    r"~?\s*"
    # The return type may be scope-qualified (`COpenWithList::SApp *GetCurrentApp()`);
    # matching only from `SApp` stranded `COpenWithList::` onto the next member.
    r"(?:STDMETHOD(?:IMP)?|virtual\s+[^;()]+"
    r"|[A-Za-z_]\w*(?:\s*::\s*[A-Za-z_]\w*)*(?:\s+(?:WINAPI|CALLBACK|APIENTRY|PASCAL"
    r"|STDMETHODCALLTYPE|NTAPI|WINAPIV|FASTCALL|CDECL|__stdcall|__cdecl|__fastcall))?)\s*"
    r"(?:[*&]\s*)?"
    r"(?:\([^;{}]*\)|[A-Za-z_]\w*\s*\([^;{}]*\))\s*(?:const\s*)?(?:override\s*)?;"
)

# Tokens that can only be parser debris once a statement is claimed to be a data member.
# A field carrying one of these is a mis-parse, and a mis-parsed field silently changes the
# object layout — so it HARD FAILS instead of being dropped or guessed at.
_FIELD_DEBRIS = re.compile(r"\b(?:static|typedef|virtual|inline|template|explicit|friend)\b")


def _balanced_block(text: str, open_brace: int) -> tuple[str, int]:
    depth = 0
    for i in range(open_brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[open_brace + 1 : i], i
    raise CppComError("unbalanced C++ class/interface block")


def _find_class_text(header: Path, class_name: str) -> str:
    text = header.read_text(encoding="utf-8", errors="replace")
    for match in _CLASS_HEAD.finditer(text):
        if match.group("name") == class_name:
            body, _ = _balanced_block(text, match.end() - 1)
            return body
    raise CppComError(f"{class_name}: class definition not found in {header}")


def _local_concrete_base(
    source: Path, header: Path, base: str
) -> tuple[Path, str, list[str]] | None:
    """Header + body + bases for a deterministically flattenable concrete base.

    Same-header bases are the common case (CCopyMoveToMenu), but shell32 also uses
    small concrete SDK helpers such as CObjectWithSiteBase from shellutils.h.  A
    non-template base with one unambiguous definition is equally provable; ATL
    templates remain outside this path and fail loud.
    """
    if "<" in base:
        return None
    try:
        base_header = header if base in _class_names(header) else find_class_header(source, base)
        return base_header, _find_class_text(base_header, base), _class_bases(base_header, base)
    except CppComError:
        return None


def _class_names(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8", errors="replace")
    return {match.group("name") for match in _CLASS_HEAD.finditer(text)}


def _state_field_name(declaration: str) -> str:
    """The declared identifier of one raw state-field declaration string."""
    text = " ".join(declaration.split())
    text = re.sub(r"\s*=\s*[^=]+$", "", text)
    match = re.search(r"([A-Za-z_]\w*)\s*(?:\[[^\]]*\])?\s*$", text)
    return match.group(1) if match else text


def _class_bases(header: Path, class_name: str) -> list[str]:
    text = header.read_text(encoding="utf-8", errors="replace")
    for match in _CLASS_HEAD.finditer(text):
        if match.group("name") != class_name:
            continue
        raw = (match.group("bases") or "").lstrip(":").strip()
        raw = re.sub(r"/\*.*?\*/|//[^\n]*", " ", raw, flags=re.S)
        return _split_args(raw) if raw else []
    raise CppComError(f"{class_name}: class definition not found in {header}")


def _header_files(root: Path, pattern: str = "*.h") -> list[Path]:
    """Retry directory enumeration if concurrent tooling removes a scratch directory.

    Materialize the entire walk before returning: a partial corpus must never
    escape. Missing roots and persistent directory churn still fail.
    """
    for attempt in range(3):
        root.stat()
        try:
            return list(root.rglob(pattern))
        except FileNotFoundError:
            if attempt == 2:
                raise
    raise AssertionError("unreachable")


def find_class_header(source: Path, class_name: str) -> Path:
    """Find the source/header containing the actual class definition.

    shell32 has both classes defined directly in their ``.cpp`` and forward
    declarations spread across sibling headers. A bare ``class Name`` grep
    confuses those declarations with definitions and makes reruns ambiguous.
    """

    def defines(path: Path) -> bool:
        text = path.read_text(encoding="utf-8", errors="replace")
        return any(match.group("name") == class_name for match in _CLASS_HEAD.finditer(text))

    if defines(source):
        return source

    hits = [header for header in sorted(source.parent.glob("*.h")) if defines(header)]
    if not hits:
        shell32_root = next((p for p in source.parents if p.name == "shell32"), None)
        if shell32_root is not None and shell32_root != source.parent:
            hits = [header for header in sorted(_header_files(shell32_root)) if defines(header)]
    if not hits:
        repo_root = next(
            (parent for parent in source.parents if (parent / "sdk" / "include").is_dir()),
            None,
        )
        if repo_root is not None:
            include_root = repo_root / "sdk" / "include"
            hits = [header for header in sorted(_header_files(include_root)) if defines(header)]
            if not hits:
                atl_root = repo_root / "sdk" / "lib" / "atl"
                if atl_root.is_dir():
                    hits = [header for header in sorted(_header_files(atl_root)) if defines(header)]
    if not hits:
        raise CppComError(f"{class_name}: no defining C++ source/header found from {source}")
    if len(hits) > 1:
        raise CppComError(
            f"{class_name}: ambiguous defining header: " + ", ".join(str(p) for p in hits)
        )
    return hits[0]


def _parse_com_map(body: str, class_name: str) -> tuple[list[ComMapEntry], list[str]]:
    begin = _BEGIN_MAP.search(body)
    if not begin or begin.group(1) != class_name:
        return [], []
    end = _END_MAP.search(body, begin.end())
    if not end:
        raise CppComError(f"{class_name}: BEGIN_COM_MAP has no END_COM_MAP")
    map_text = body[begin.end() : end.start()]
    map_text = re.sub(r"/\*.*?\*/|//[^\n]*", " ", map_text, flags=re.S)
    unsupported = [
        f"unsupported ATL map macro {m.group(1)}" for m in _UNSUPPORTED_MAP.finditer(map_text)
    ]
    entries: list[ComMapEntry] = []
    for match in _MAP_ENTRY.finditer(map_text):
        macro, args = match.group(1), _split_args(match.group(2))
        if macro == "COM_INTERFACE_ENTRY" and len(args) == 1:
            entries.append(ComMapEntry(f"IID_{args[0]}", args[0], macro))
        elif macro == "COM_INTERFACE_ENTRY_IID" and len(args) == 2:
            entries.append(ComMapEntry(args[0], args[1], macro))
        elif macro == "COM_INTERFACE_ENTRY2" and len(args) == 2:
            entries.append(ComMapEntry(f"IID_{args[0]}", args[0], macro))
        elif macro == "COM_INTERFACE_ENTRY2_IID" and len(args) == 3:
            entries.append(ComMapEntry(args[0], args[1], macro))
        else:
            unsupported.append(f"unrecognized {macro} argument shape: {match.group(2).strip()}")
    return entries, unsupported


def _state_fields(body: str) -> list[str]:
    """Conservatively list class data declarations outside the COM-map/method surface."""
    # Comments are not syntax.  Leaving them in previously made trailing prose such
    # as ``// CControlPanelFolder does it for us`` look like an object field.
    stripped = re.sub(r"/\*.*?\*/|//[^\n]*", " ", body, flags=re.S)
    # A PREPROCESSOR DIRECTIVE ENDS AT THE NEWLINE, NOT AT A SEMICOLON. The split below is on
    # `;`, so a directive with no semicolon of its own merges with the declaration that FOLLOWS
    # it into one statement, which then starts with `#` and is skipped -- silently taking a real
    # data member with it. CMenuFocusManager.h does exactly that:
    #
    #     #define MAX_RECURSE 20
    #     StackEntry m_bandStack[MAX_RECURSE];
    #
    # and `m_bandStack` vanished from the generated struct while its siblings `m_bandCount` and
    # `m_menuDepth` stayed -- a struct whose layout silently disagrees with the C++. This is the
    # same swallow-the-next-declaration shape the `static` and `~dtor` comments below describe;
    # it is the third one, so directives are removed LINE-WISE here rather than skipped later.
    #
    # Only `#define`/`#undef`, which never guard code. Conditionals are left to the existing
    # `#`-statement skip: dropping an `#if` line would expose whatever it guards as live fields.
    stripped = re.sub(r"^\s*#\s*include[^\n]*$", "", stripped, flags=re.M)
    stripped = re.sub(
        r"^[ \t]*#[ \t]*(?:define|undef)[^\n]*(?:\\\n[^\n]*)*$", "", stripped, flags=re.M
    )
    stripped = re.sub(r"BEGIN_COM_MAP\b.*?END_COM_MAP\s*\(\s*\)", "", stripped, flags=re.S)
    # ...and the MESSAGE map, for the SAME reason and by the same swallow. This is the third
    # instance of the shape described above: `END_MSG_MAP()` carries parentheses, the split
    # below is on `;`, and two of CChangeNotifyServer's MESSAGE_HANDLER lines end in a stray
    # `;` -- so the final fragment is `END_MSG_MAP() private: UINT m_nNextRegID`, which looks
    # like a method declaration and is skipped, taking a real data member with it. The struct
    # then silently disagrees with the C++ about its own layout, which is exactly what
    # `m_bandStack` did.
    stripped = re.sub(r"BEGIN_MSG_MAP\b.*?END_MSG_MAP\s*\(\s*\)", "", stripped, flags=re.S)

    # Remove in-class method definitions before splitting declarations at ';'.
    # A body can itself contain semicolons, and treating those fragments as class
    # declarations produces both false positives and (worse) architecture-dependent
    # layout decisions.  We only consume a balanced block when its declaration
    # prefix contains a parameter list, so nested type declarations remain visible.
    inline_method = re.compile(
        r"(?:^|[;}])\s*(?:template\s*<[^>{}]*>\s*)?[^;{}]*\([^;{}]*\)"
        r"\s*(?:const\s*)?(?:noexcept\s*)?(?:override\s*)?\{",
        re.M,
    )
    while match := inline_method.search(stripped):
        open_brace = match.end() - 1
        _, close_brace = _balanced_block(stripped, open_brace)
        stripped = stripped[: match.start()] + ";" + stripped[close_brace + 1 :]

    # Nested POD declarations define a type; their members are not additional
    # fields of the outer COM object.  Keep the actual ``volume_info volume;``
    # declaration while removing the nested type block before semicolon
    # splitting.  The type block is independently captured by
    # ``_discover_pod_state_types`` from the unmodified class text.
    # `typedef` must be part of the match, not left dangling in front of the next
    # declaration — `typedef struct _LANGANDCODEPAGE {...} LANGANDCODEPAGE, *LP...;`
    # otherwise leaves a bare `typedef` that glues onto the field below it.
    nested_type = re.compile(r"\b(?:typedef\s+)?(?:struct|union|enum)\s+[A-Za-z_]\w*\s*\{")
    while match := nested_type.search(stripped):
        open_brace = stripped.find("{", match.start())
        _, close_brace = _balanced_block(stripped, open_brace)
        semicolon = stripped.find(";", close_brace)
        if semicolon < 0:
            break
        stripped = stripped[: match.start()] + stripped[semicolon + 1 :]

    stripped = _METHOD_DECL.sub("", stripped)
    stripped = re.sub(r"\b(?:public|private|protected)\s*:\s*", "", stripped)
    stripped = re.sub(r"\bDECLARE_[A-Z0-9_]+\s*(?:\([^)]*\))?", "", stripped)
    fields = []
    for statement in stripped.split(";"):
        statement = " ".join(statement.split())
        if not statement or "(" in statement or ")" in statement or statement == "override":
            continue
        if statement.startswith("#") or " #" in statement:
            continue
        # A static member is not part of the instance layout, so it is dropped — but drop it
        # only when it is a CLEAN static declaration. `static INT_PTR CALLBACK DialogProc();`
        # once stranded `static INT_PTR`, which glued onto the following member and made the
        # whole thing start with `static ` — so the drop silently swallowed a real field.
        # Debris after the storage class means the strip is wrong, not that the member is.
        if statement.startswith("static "):
            if _FIELD_DEBRIS.search(statement[len("static ") :]):
                raise CppComError(
                    f"class layout parse merged debris into a static declaration: "
                    f"{statement!r}. Dropping it would silently delete the data member "
                    "trailing it — fix the method-declaration stripping."
                )
            continue
        # A destructor's leading `~` used to make the whole statement be DROPPED. When the
        # `~` stands alone that is right, but `~COpenWithList();` is followed by the first
        # data member, and once the declaration was consumed the two joined into
        # `~ SApp *m_pApp` — so the class SILENTLY LOST its first field, for every class
        # that declares a destructor. `_METHOD_DECL` now eats the `~` with the declaration;
        # strip any survivor rather than discarding the member behind it.
        if statement.startswith("~"):
            statement = statement.lstrip("~").strip()
            if not statement:
                continue
        # Access labels and macro debris contain no plausible declarator.
        if re.search(r"\b[A-Za-z_]\w*\s*(?:\[[^]]+\])?$", statement):
            # C permits ``UINT first, second``. There is one base type and two
            # independent scalar fields; expand that proven shape before the
            # single-declarator layout parser. Pointer stars, initializers,
            # bitfields, and per-declarator arrays remain loud.
            multiple = re.fullmatch(
                r"(?P<type>[A-Za-z_]\w*(?:\s+[A-Za-z_]\w*)*)\s+"
                r"(?P<names>[A-Za-z_]\w*(?:\s*,\s*[A-Za-z_]\w*)+)",
                statement,
            )
            if multiple is not None:
                fields.extend(
                    f"{multiple.group('type')} {name.strip()}"
                    for name in multiple.group("names").split(",")
                )
            else:
                fields.append(statement)
    for statement in fields:
        debris = _FIELD_DEBRIS.search(statement)
        if debris is not None:
            raise CppComError(
                f"class layout parse produced a field carrying `{debris.group(0)}`: "
                f"{statement!r}. That is method/typedef debris merged into a data member, "
                "and a mis-parsed member silently changes the object layout — fix the "
                "declaration stripping rather than shipping a guessed struct."
            )
    return fields


_POD_LEAF_TYPES = frozenset(
    {
        "CLSID",
        "FILETIME",
        "GUID",
        "IID",
        "LARGE_INTEGER",
        "STGMEDIUM",
        "ULARGE_INTEGER",
        # Not a Windows type: `crate::atlwin::AtlWindowState`, the Rust stand-in for what
        # a C++ CWindowImpl class inherits. It is a proven local layout for the same
        # reason the others are -- we define it -- and the CWindowImpl branch above
        # synthesises a field of it.
        "AtlWindowState",
        # Likewise `crate::atlwin::AtlContainedWindow` -- the Rust stand-in for ATL's
        # `CContainedWindow` (`CContainedWindowT<CWindow>`), an embedded window that
        # SUBCLASSES an existing HWND and routes its messages back into the owner's
        # message map under an alt-map id. `CMenuToolbarBase::m_pager` is the shape.
        "AtlContainedWindow",
    }
)

# Rust spellings bindgen emits for an integer primitive on the right of a type alias.
_ISLAND_INTEGER_PRIMITIVES = frozenset(
    {
        "::core::ffi::c_char",
        "::core::ffi::c_int",
        "::core::ffi::c_long",
        "::core::ffi::c_schar",
        "::core::ffi::c_short",
        "::core::ffi::c_uchar",
        "::core::ffi::c_uint",
        "::core::ffi::c_ulong",
        "::core::ffi::c_ushort",
        "i8",
        "i16",
        "i32",
        "i64",
        "u8",
        "u16",
        "u32",
        "u64",
    }
)
_ISLAND_TYPE_ALIAS = re.compile(r"^pub type (?P<name>\w+) = (?P<rhs>[^;]+);", re.M)
_ISLAND_TYPE_REEXPORT = re.compile(r"^pub use self::(?P<target>\w+) as (?P<name>\w+);", re.M)
# `pub use ntdll_sys::nt::*;` — the substrate crate an island resolves its base types through.
_ISLAND_SYS_GLOB = re.compile(r"^pub use (?P<crate>\w+)_sys::\w+::\*;", re.M)
# Alias chains are short in practice (SHCONTF -> DWORD -> c_ulong); the bound is a cycle guard,
# not a tuning knob. A malformed island that aliases in a loop must terminate, not hang.
_ISLAND_ALIAS_MAX_HOPS = 8


def _sys_companion(island: Path) -> Path | None:
    """The `-sys` bindgen file an island resolves its base types through, same arch.

    An island stops at the names bindgen emitted for THIS module; everything below that
    (``DWORD``, ``UINT``, ...) arrives through the ``pub use <x>_sys::<mod>::*`` glob at
    its head. Reading that file is what lets a chain like ``SHCONTF = DWORD`` reach a
    primitive. The arch comes from the island's OWN filename, never from a default: pairing
    ``island_arm64.rs`` with ``bindgen_i386.rs`` would silently answer the width question
    with the wrong target, which is the one thing this whole module exists to prevent.
    """
    arch = re.fullmatch(r"island_(\w+)\.rs", island.name)
    if arch is None:
        return None
    try:
        head = island.read_text(encoding="utf-8", errors="replace")[:4096]
    except OSError:
        return None
    glob = _ISLAND_SYS_GLOB.search(head)
    if glob is None:
        return None
    # <workspace>/<crate>-rust/src/island_<arch>.rs -> <workspace>/<crate>-sys/src/bindgen_<arch>.rs
    workspace = island.parent.parent.parent
    companion = (
        workspace
        / f"{glob.group('crate').replace('_', '-')}-sys"
        / "src"
        / f"bindgen_{arch.group(1)}.rs"
    )
    return companion if companion.is_file() else None


def _resolve_alias_chain(name: str, aliases: dict[str, str]) -> str | None:
    """Follow ``name`` through ``aliases`` to a primitive, or None.

    Transitive because a single hop is not how bindgen spells the interesting cases:
    ``SHCONTF`` is ``DWORD`` and only ``DWORD`` is ``c_ulong``, so a one-hop resolver
    proves neither and the caller falls back to rejecting a perfectly ordinary DWORD
    state field.
    """
    seen: set[str] = set()
    current = name
    for _ in range(_ISLAND_ALIAS_MAX_HOPS):
        if current in _ISLAND_INTEGER_PRIMITIVES:
            return current
        if current in seen:
            return None
        seen.add(current)
        nxt = aliases.get(current)
        if nxt is None:
            return None
        current = nxt
    return None


_ISLAND_ALIAS_CACHE: dict[tuple[str, ...], frozenset[str]] = {}


def island_integer_aliases(islands: Sequence[Path]) -> frozenset[str]:
    """Type names every per-arch bindgen island proves to be a plain integer alias.

    A C ``typedef enum { ... } NAME;`` has no fixed underlying type, so its width is
    an ABI fact rather than a source fact and reading the header cannot settle it.
    bindgen renders each island against the real per-arch target, so the islands DO
    settle it -- and a name is only accepted when EVERY arch resolves it to the SAME
    primitive. Trusting one arch would bake that arch's width into the scaffold,
    which is exactly how a cast inferred from i386 has shipped a wrong width before.

    Names that survive are used as-is in the generated Rust: they resolve through the
    prelude's ``pub use crate::island::*`` to the island's own alias, so the scaffold
    never re-declares them and there is no second source of truth for the width.
    """
    key = tuple(str(path) for path in islands)
    cached = _ISLAND_ALIAS_CACHE.get(key)
    if cached is not None:
        return cached
    resolved: list[dict[str, str]] = []
    for path in islands:
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return frozenset()
        aliases = {
            m.group("name"): " ".join(m.group("rhs").split())
            for m in _ISLAND_TYPE_ALIAS.finditer(text)
        }
        # Base types below the island's own surface live in the `-sys` crate it globs. The
        # island's aliases WIN on a name collision: it is the module-specific authority, and
        # the -sys file is only consulted for what the island never mentions.
        companion = _sys_companion(path)
        if companion is not None:
            try:
                sys_text = companion.read_text(encoding="utf-8", errors="replace")
            except OSError:
                sys_text = ""
            for m in _ISLAND_TYPE_ALIAS.finditer(sys_text):
                aliases.setdefault(m.group("name"), " ".join(m.group("rhs").split()))
        names = {m.group("name"): m.group("target") for m in _ISLAND_TYPE_REEXPORT.finditer(text)}
        # A re-export chains onto an alias (`pub use self::__WIDL_..._00000025 as
        # FOLDERVIEWMODE;` over `pub type __WIDL_..._00000025 = c_int;`). Direct
        # aliases count too, so seed with them and let re-exports resolve through.
        arch: dict[str, str] = {}
        for name in aliases:
            primitive = _resolve_alias_chain(name, aliases)
            if primitive is not None:
                arch[name] = primitive
        for name, target in names.items():
            primitive = _resolve_alias_chain(target, aliases)
            if primitive is not None:
                arch[name] = primitive
        resolved.append(arch)
    if not resolved:
        return frozenset()
    shared = set(resolved[0])
    for arch in resolved[1:]:
        # `name in resolved[0]` is load-bearing, not defensive: an arch can prove a name the
        # FIRST one never mentions at all (arm64's -sys carries _tag_ARM64INTR_BARRIER_TYPE),
        # and indexing resolved[0] blindly raises KeyError on it. Intersecting is the whole
        # point — a name absent from any arch is not proven — so filtering here is exact.
        shared &= {name for name in arch if name in resolved[0] and arch[name] == resolved[0][name]}
    _ISLAND_ALIAS_CACHE[key] = frozenset(shared)
    return _ISLAND_ALIAS_CACHE[key]


_ISLAND_STRUCT = re.compile(r"^pub struct (?P<name>\w+)\b", re.M)
# `pub use com_sys::{A,B,C};` and `pub use ntdll_sys::nt::*;` — every -sys crate an island
# resolves names through, however it spells the import.
_ISLAND_SYS_REF = re.compile(r"^pub use (?P<crate>\w+)_sys::", re.M)
_ISLAND_STRUCT_CACHE: dict[tuple[str, ...], frozenset[str]] = {}

_ISLAND_VTBL_BLOCK = re.compile(r"^pub struct (?P<name>\w+)Vtbl \{(?P<body>.*?)^\}", re.M | re.S)
_ISLAND_VTBL_SLOT = re.compile(r"^    pub (?P<slot>\w+):", re.M)
_ISLAND_LPVTBL_CACHE: dict[tuple[str, ...], dict[str, tuple[str, ...]]] = {}


def island_lpvtbl_interfaces(islands: Sequence[Path]) -> dict[str, tuple[str, ...]]:
    """Interfaces the island already provides in the C ``lpVtbl`` idiom, slots in order.

    Only that idiom qualifies. A header read in C++ mode makes bindgen emit an opaque
    ``<I>__bindgen_vtable`` with no named slots -- shell32's island is entirely that shape --
    and such a type cannot stand in for a rendered vtable struct, so shell32 keeps rendering
    its own. recyclebin's headers are read in C mode and produce the real thing.

    Returning the slot NAMES, not just the interface names, is the point. The Windows A/W
    macros expand inside the interface definition, so the island's slot for
    ``STDMETHOD(DeleteFile)`` is ``DeleteFileW`` -- and a vtable instance must use the
    island's spelling, not the header's textual one.

    Intersected across arches for the same reason as :func:`island_integer_aliases`: a slot
    list only one arch proves is not proven.
    """
    key = tuple(str(path) for path in islands)
    if key in _ISLAND_LPVTBL_CACHE:
        return _ISLAND_LPVTBL_CACHE[key]
    per_arch: list[dict[str, tuple[str, ...]]] = []
    for island in islands:
        if not island.is_file():
            continue
        text = island.read_text(encoding="utf-8", errors="replace")
        # The companion object struct is what proves the C idiom: `pub struct I { pub lpVtbl }`.
        with_lpvtbl = {
            match.group("name")
            for match in re.finditer(r"^pub struct (?P<name>\w+) \{\n    pub lpVtbl:", text, re.M)
        }
        found: dict[str, tuple[str, ...]] = {}
        for block in _ISLAND_VTBL_BLOCK.finditer(text):
            name = block.group("name")
            if name not in with_lpvtbl:
                continue
            slots = tuple(_ISLAND_VTBL_SLOT.findall(block.group("body")))
            if slots:
                found[name] = slots
        per_arch.append(found)
    if not per_arch:
        result: dict[str, tuple[str, ...]] = {}
    else:
        shared = set(per_arch[0])
        for other in per_arch[1:]:
            shared &= {name for name, slots in other.items() if per_arch[0].get(name) == slots}
        result = {name: per_arch[0][name] for name in sorted(shared)}
    _ISLAND_LPVTBL_CACHE[key] = result
    return result


def island_struct_types(islands: Sequence[Path]) -> frozenset[str]:
    """Struct names every per-arch bindgen island proves, through its whole -sys closure.

    This is a **destructor** proof, not a layout one. bindgen only ever renders C
    declarations, so a name it emits as a struct came from a C header and is therefore
    trivially destructible — exactly the property an ATL container element must have for
    `atl.rs`'s `Drop` (a `drop_in_place` per element) to match the C++.

    A C++ class with a real destructor is never in this set, so `CSimpleArray<CStringW>`
    stays a loud refusal instead of becoming a silent leak of every element.

    Aliases are followed, because the C spelling is almost never the struct's own name:
    `FORMATETC` is `pub type FORMATETC = tagFORMATETC;` over `pub struct tagFORMATETC`.
    Multiple -sys crates are consulted, not just the one `_sys_companion` picks, since
    an island imports its ordinary base types from one (`ntdll_sys`) and its COM types
    from another (`com_sys`) — reading only the first would refuse every OLE struct.

    Intersected across arches for the same reason as :func:`island_integer_aliases`: a
    name only one arch proves is not proven.
    """
    key = tuple(str(path) for path in islands)
    cached = _ISLAND_STRUCT_CACHE.get(key)
    if cached is not None:
        return cached
    per_arch: list[set[str]] = []
    for path in islands:
        arch = re.fullmatch(r"island_(\w+)\.rs", path.name)
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return frozenset()
        sources = [text]
        if arch is not None:
            workspace = path.parent.parent.parent
            for match in _ISLAND_SYS_REF.finditer(text[:8192]):
                companion = (
                    workspace
                    / f"{match.group('crate').replace('_', '-')}-sys"
                    / "src"
                    / f"bindgen_{arch.group(1)}.rs"
                )
                if companion.is_file():
                    sources.append(companion.read_text(encoding="utf-8", errors="replace"))
        structs: set[str] = set()
        aliases: dict[str, str] = {}
        for source in sources:
            structs |= {m.group("name") for m in _ISLAND_STRUCT.finditer(source)}
            for m in _ISLAND_TYPE_ALIAS.finditer(source):
                # `pub type FORMATETC = tagFORMATETC;` — strip any crate qualifier so
                # `com_sys::_COAUTHIDENTITY` matches the struct name it names.
                aliases.setdefault(m.group("name"), m.group("rhs").strip().split("::")[-1])
        proven = set(structs)
        for name in aliases:
            current, hops = name, 0
            while current in aliases and hops < _ISLAND_ALIAS_MAX_HOPS:
                current = aliases[current]
                hops += 1
            if current in structs:
                proven.add(name)
        per_arch.append(proven)
    if not per_arch:
        return frozenset()
    shared = set(per_arch[0])
    for arch_names in per_arch[1:]:
        shared &= arch_names
    _ISLAND_STRUCT_CACHE[key] = frozenset(shared)
    return _ISLAND_STRUCT_CACHE[key]


_ISLAND_STRUCT_BODY = re.compile(r"^pub struct (?P<name>\w+) \{(?P<body>.*?)^\}", re.M | re.S)
_ISLAND_STRUCT_FIELD = re.compile(r"^\s*pub (?P<field>\w+): (?P<type>[^,\n]+),\s*$", re.M)
_ISLAND_FIELD_CACHE: dict[tuple[str, ...], dict[str, dict[str, str]]] = {}

# bindgen spells C's primitives through core::ffi; overload binding compares C spellings.
# Only these exact primitives are translated back. Anything else -- a pointer, an array,
# a nested struct -- is left out entirely rather than guessed at, because an argument the
# binder cannot type is SKIPPED (it scores nothing) while an argument it types WRONGLY
# picks an overload. Silence is safe here; a bad guess is not.
_ISLAND_FFI_PRIMITIVES = {
    "::core::ffi::c_int": "int",
    "::core::ffi::c_uint": "unsigned int",
    "::core::ffi::c_long": "long",
    "::core::ffi::c_ulong": "unsigned long",
    "::core::ffi::c_short": "short",
    "::core::ffi::c_ushort": "unsigned short",
    "::core::ffi::c_char": "char",
    "::core::ffi::c_schar": "signed char",
    "::core::ffi::c_uchar": "unsigned char",
    "::core::ffi::c_longlong": "long long",
    "::core::ffi::c_ulonglong": "unsigned long long",
}


def island_struct_field_types(islands: Sequence[Path]) -> dict[str, dict[str, str]]:
    """Field C types every per-arch bindgen island agrees on, keyed by struct name.

    Exists so the overload binder can type a MEMBER ACCESS argument. C++ resolves
    `SelectItem(hittest.iItem, ...)` by the static type of `hittest.iItem`; the binder
    could previously type only bare identifiers and `this->x`, so a member access
    contributed no evidence and two same-arity candidates stayed tied forever. That is
    the shape CDefView_OnContextMenu hit against SelectItem(PCUITEMID_CHILD, UINT) vs
    SelectItem(int, DWORD) -- a tie the ARGUMENT settles and nothing else can.

    The islands are the right authority rather than a hand-written table: they are
    generated from THIS worktree's headers at THIS commit, so a struct that changes
    shape changes this answer with it, and the fleet-isolation READ rule already
    guarantees they are not some sibling lane's.

    Intersected across arches for the same reason as :func:`island_integer_aliases`,
    and it is not hypothetical here: a field is only usable as evidence when its width
    is a source fact rather than an ABI one. A field the arches DISAGREE on is dropped,
    which turns a width-dependent overload back into the loud ambiguity it really is.

    Aliases are followed, because C code names the alias: `LVHITTESTINFO hittest;` over
    `pub struct tagLVHITTESTINFO`.
    """
    key = tuple(str(path) for path in islands)
    cached = _ISLAND_FIELD_CACHE.get(key)
    if cached is not None:
        return cached
    per_arch: list[dict[str, dict[str, str]]] = []
    for path in islands:
        arch = re.fullmatch(r"island_(\w+)\.rs", path.name)
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return {}
        sources = [text]
        if arch is not None:
            workspace = path.parent.parent.parent
            for match in _ISLAND_SYS_REF.finditer(text[:8192]):
                companion = (
                    workspace
                    / f"{match.group('crate').replace('_', '-')}-sys"
                    / "src"
                    / f"bindgen_{arch.group(1)}.rs"
                )
                if companion.is_file():
                    sources.append(companion.read_text(encoding="utf-8", errors="replace"))
        fields: dict[str, dict[str, str]] = {}
        aliases: dict[str, str] = {}
        for source in sources:
            for struct in _ISLAND_STRUCT_BODY.finditer(source):
                members = {
                    member.group("field"): _ISLAND_FFI_PRIMITIVES.get(
                        " ".join(member.group("type").split()),
                        " ".join(member.group("type").split()),
                    )
                    for member in _ISLAND_STRUCT_FIELD.finditer(struct.group("body"))
                    # A field whose type is not a plain name is left untyped on purpose
                    # (see _ISLAND_FFI_PRIMITIVES): pointers and arrays would need the
                    # binder to reason about decay, which is exactly the guessing this
                    # whole path refuses to do.
                    if re.fullmatch(
                        r"[A-Za-z_]\w*|::core::ffi::c_\w+",
                        " ".join(member.group("type").split()),
                    )
                }
                fields.setdefault(struct.group("name"), members)
            for m in _ISLAND_TYPE_ALIAS.finditer(source):
                aliases.setdefault(m.group("name"), m.group("rhs").strip().split("::")[-1])
        for name in aliases:
            current, hops = name, 0
            while current in aliases and hops < _ISLAND_ALIAS_MAX_HOPS:
                current = aliases[current]
                hops += 1
            if current in fields:
                fields.setdefault(name, fields[current])
        per_arch.append(fields)
    if not per_arch:
        return {}
    shared: dict[str, dict[str, str]] = {}
    for name, members in per_arch[0].items():
        agreed = {
            member: c_type
            for member, c_type in members.items()
            if all(other.get(name, {}).get(member) == c_type for other in per_arch[1:])
        }
        if agreed:
            shared[name] = agreed
    _ISLAND_FIELD_CACHE[key] = shared
    return shared


_RUST_KEYWORDS = frozenset(
    {
        "as",
        "break",
        "const",
        "continue",
        "crate",
        "else",
        "enum",
        "extern",
        "false",
        "final",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "match",
        "mod",
        "move",
        "mut",
        "pub",
        "ref",
        "return",
        "self",
        "Self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "type",
        "unsafe",
        "use",
        "where",
        "while",
    }
)


def _parse_plain_state_field(
    declaration: str,
    *,
    allowed_pod_types: set[str] | frozenset[str] = frozenset(),
    array_constants: dict[str, int] | None = None,
    island_scalar_types: set[str] | frozenset[str] = frozenset(),
    island_struct_types: set[str] | frozenset[str] = frozenset(),
) -> PlainStateField:
    """Parse the deliberately small C-layout state subset.

    A Rust-owned COM allocation does not contain ATL base-class state, so plain C
    scalar/pointer fields can be represented directly and zero-initialized before
    the translated constructor runs. C++ objects, references, arrays, bitfields,
    initializers, and multi-declarator statements retain hidden lifetime/layout
    semantics and must remain a loud preflight failure.
    """
    text = " ".join(declaration.split())
    # `struct IconLocation defaultIcon` is C's elaborated-type-specifier: the keyword names
    # no extra semantics, it is the same type as `IconLocation defaultIcon`, and C++ accepts
    # both. Only the bare spelling was recognised, so a class declaring its members the C way
    # failed with "state field type is not proven C scalar/pointer state" -- which reads as a
    # claim about the TYPE when the truth was that nobody had parsed the declaration.
    # CExtractIcon declares all four of its state fields this way.
    text = _ELABORATED_TYPE_RE.sub("", text)
    # A C++11 zero-valued default member initializer is exactly the scaffold's
    # zero-init (HeapAlloc(HEAP_ZERO_MEMORY) + zeroed ctor write); strip it.
    # Any non-zero default stays inside the declaration and fails loud below.
    zero_default = re.fullmatch(r"(?P<decl>.+?)\s*=\s*(?:NULL|nullptr|0|FALSE|false)", text)
    if zero_default is not None:
        text = zero_default.group("decl")
    com_ptr = re.fullmatch(r"CComPtr\s*<\s*([A-Za-z_]\w*)\s*>\s+([A-Za-z_]\w*)", text)
    if com_ptr is not None:
        interface, name = com_ptr.groups()
        return PlainStateField(
            declaration=text,
            c_type=f"{interface} *",
            name=name,
            com_ptr_interface=interface,
        )
    heap_ptr = re.fullmatch(
        r"CComHeapPtr\s*<\s*(?P<pointee>[A-Za-z_]\w*)\s*>\s+(?P<name>[A-Za-z_]\w*)"
        r"(?:\s*\[\s*(?P<count>[A-Za-z_]\w*|\d+)\s*\])?",
        text,
    )
    if heap_ptr is not None:
        # A fixed ARRAY of owned heap pointers is still deterministic: C++ destroys each
        # element, so the generated Release frees each element. CRecycleBinFolderViewCB's
        # `CComHeapPtr<ITEMIDLIST> m_pidls[2]` is the shape.
        count_text = heap_ptr.group("count")
        array_len = None
        if count_text is not None:
            if count_text.isdigit():
                array_len = int(count_text)
            else:
                constants = array_constants or {}
                if count_text not in constants:
                    raise CppComError(
                        f"state field array bound is not a proven numeric macro: {declaration}"
                    )
                array_len = constants[count_text]
        return PlainStateField(
            declaration=text,
            c_type=f"{heap_ptr.group('pointee')} *",
            name=heap_ptr.group("name"),
            owned_heap_ptr=True,
            array_len=array_len,
        )
    cstring = re.fullmatch(r"CStringW\s+([A-Za-z_]\w*)", text)
    if cstring is not None:
        return PlainStateField(
            declaration=text,
            c_type="CStringW",
            name=cstring.group(1),
            owned_cstring=True,
        )
    # `CComBSTR m_name` — ATL's owning BSTR. Matched on its own rather than through the
    # `Tmpl<Elem>` branch below because it takes no type parameter; it still keeps its TYPE
    # so atl.rs's `Drop` reproduces `~CComBSTR`'s SysFreeString. Lowering it to a bare
    # `BSTR` instead would compile and silently leak every string the object ever held.
    # ATL's `CContainedWindow` is a TYPEDEF (`CContainedWindowT<CWindow>`), so it reaches
    # the parser as a plain `Type name` and would otherwise be rejected as unproven state.
    # Named here rather than left to the generic path because the Rust side is a
    # DIFFERENT layout by design, exactly as `AtlWindowState` is: ATL's version carries a
    # machine-code thunk and a current-message pointer that `crate::atlwin` does not
    # reproduce.
    contained = re.fullmatch(r"CContainedWindow\s+([A-Za-z_]\w*)", text)
    if contained is not None:
        return PlainStateField(
            declaration=text,
            c_type="AtlContainedWindow",
            name=contained.group(1),
        )
    ccombstr = re.fullmatch(r"CComBSTR\s+([A-Za-z_]\w*)", text)
    if ccombstr is not None:
        return PlainStateField(
            declaration=text,
            c_type="CComBSTR",
            name=ccombstr.group(1),
            raii_container="CComBSTR",
        )
    container = _RAII_CONTAINER_FIELD_RE.fullmatch(text)
    if container is not None and container.group("template") in _RAII_CONTAINER_LOWERINGS:
        # The element type has to be one the scaffold can already spell, or the container
        # would be generic over a type the module never defines. Two families qualify, and
        # they are safe for DIFFERENT reasons:
        #   - a scalar or an island struct is trivially destructible, so the container's
        #     per-element `drop_in_place` is a no-op and matches C++ exactly;
        #   - a source-discovered POD may hold ATL members, and is safe because the POD is
        #     rendered with `owned_rust_type` — its RAII members keep their owning Rust
        #     types, so Rust's field-drop order IS the C++ implicit destructor.
        # Rendering that POD with raw pointers instead would leak every element with
        # nothing red anywhere; `CAtlList<DynamicShellEntry>` (a `CComPtr<IContextMenu>`
        # per entry) is the shape that makes the difference visible.
        element = container.group("element")
        is_pointer_element = bool(container.group("ptr"))
        known = (
            set(allowed_pod_types)
            | set(island_scalar_types)
            | set(island_struct_types)
            | set(_POD_LEAF_TYPES)
            | set(_RUST_SCALARS)
        )
        # A container of RAW POINTERS is the third safe family, and it is safer than either
        # of the two above rather than a relaxation of them. `CSimpleArray<CWatchItem*>`
        # stores pointers: ATL's per-element `DestructItem` on a pointer is a no-op, and so
        # is Rust's `drop_in_place::<*mut T>`, so the container's `Drop` reproduces the C++
        # one no matter what the POINTEE looks like. The element's layout is simply not the
        # container's business — it only has to be NAMEABLE for `CSimpleArray<*mut T>` to
        # spell, which a source-discovered POD is.
        if element not in known:
            hint = (
                f"`{element}` is not a type this module defines or proves"
                if is_pointer_element
                else f"`{element}` is neither a source-discovered POD nor a proven scalar"
            )
            raise CppComError(
                f"container element type is not a proven layout: {declaration} ({hint})"
            )
        rust_element = f"*mut {element}" if is_pointer_element else element
        c_element = f"{element}*" if is_pointer_element else element
        return PlainStateField(
            declaration=text,
            c_type=f"{container.group('template')}<{c_element}>",
            name=container.group("name"),
            raii_container=f"{container.group('template')}<{rust_element}>",
        )
    ptr_array = re.fullmatch(
        r"(?P<type>[A-Za-z_]\w*(?:\s+[A-Za-z_]\w*)*)\s*\*\s*"
        r"(?P<name>[A-Za-z_]\w*)\s*\[\s*(?P<count>[A-Za-z_]\w*|\d+)\s*\]",
        text,
    )
    if ptr_array is not None:
        # Fixed array of raw pointers, e.g. `IShellFolder *m_cache[N]`. A pointer is
        # a valid C scalar whatever the pointee, so the layout is `[*mut T; N]`,
        # zero-initialized to nulls. Any per-element ownership (AddRef/Release) lives
        # in the class's explicit, translated ctor/dtor — the scaffold imposes no
        # auto-release here (that is reserved for CComPtr `com_ptr_interface` fields).
        count_text = ptr_array.group("count")
        if count_text.isdigit():
            count = int(count_text)
        else:
            constants = array_constants or {}
            if count_text not in constants:
                raise CppComError(
                    f"state field array bound is not a proven numeric macro: {declaration}"
                )
            count = constants[count_text]
        return PlainStateField(
            declaration=text,
            c_type=f"{ptr_array.group('type')} *",
            name=ptr_array.group("name"),
            array_len=count,
        )
    fixed_array = re.fullmatch(
        r"(?P<type>[A-Za-z_]\w*(?:\s+[A-Za-z_]\w*)*)\s+"
        r"(?P<name>[A-Za-z_]\w*)\s*\[\s*(?P<count>[A-Za-z_]\w*|\d+)\s*\]",
        text,
    )
    if fixed_array is not None:
        c_type = fixed_array.group("type")
        base = " ".join(re.sub(r"\b(?:const|volatile)\b", "", c_type).split())
        scalar_types = set(_RUST_SCALARS) | _WINDOWS_SCALAR_TYPEDEFS
        pointer_typedef = base in _KNOWN_POINTER_TYPEDEFS or bool(
            re.fullmatch(r"(?:L?P(?:C)?|H)[A-Z][A-Z0-9_]*", base)
        )
        if (
            base not in scalar_types
            and not pointer_typedef
            and base not in allowed_pod_types
            and base not in island_scalar_types
        ):
            raise CppComError(f"state field array element type is not proven scalar: {declaration}")
        count_text = fixed_array.group("count")
        if count_text.isdigit():
            count = int(count_text)
        else:
            constants = array_constants or {}
            if count_text not in constants:
                raise CppComError(
                    f"state field array bound is not a proven numeric macro: {declaration}"
                )
            count = constants[count_text]
        return PlainStateField(
            declaration=text,
            c_type=c_type,
            name=fixed_array.group("name"),
            array_len=count,
        )
    if any(token in text for token in ("<", ">", "&", "[", "]", "{", "}", "=", ",", ":")):
        template = re.match(r"([A-Za-z_]\w*)\s*<", text)
        if template is not None:
            # Name the mechanism. An unlisted C++ container is not a mystery, it is a
            # missing Rust lowering, and the fix is bounded: implement it in atl.rs and
            # add it to _RAII_CONTAINER_LOWERINGS. Saying so here is what stops the next
            # reader from concluding the TU is untranslatable.
            registered = template.group(1) in _RAII_CONTAINER_LOWERINGS
            why = (
                # Do NOT send the reader to the registry when the entry is already there --
                # `CSimpleArray` was registered AND implemented in atl.rs, and the old
                # message still said to go add it, which cost a round of looking for a
                # missing thing that existed. If it is registered, the template matched
                # nothing here, so the SHAPE is what this parser cannot spell.
                f"`{template.group(1)}` has a Rust lowering, but this declaration's shape is "
                f"not one the state-field parser can spell"
                if registered
                else f"no Rust lowering for `{template.group(1)}`; implement it in "
                f"shell32-rust/src/atl.rs and list it in _RAII_CONTAINER_LOWERINGS"
            )
            raise CppComError(
                f"state field needs an explicit C++ layout/lifetime mapping: {declaration} ({why})"
            )
        raise CppComError(
            f"state field needs an explicit C++ layout/lifetime mapping: {declaration}"
        )
    match = re.fullmatch(r"(?P<type>.+?)(?P<name>[A-Za-z_]\w*)", text)
    if match is None:
        raise CppComError(f"cannot parse object state declaration: {declaration}")
    c_type = match.group("type").strip()
    name = match.group("name")
    if not c_type or "(" in c_type or ")" in c_type:
        raise CppComError(f"cannot parse object state declaration: {declaration}")
    base = re.sub(r"\b(?:const|volatile)\b", "", c_type).replace("*", "")
    base = " ".join(base.split())
    scalar_types = set(_RUST_SCALARS) | _WINDOWS_SCALAR_TYPEDEFS
    pointer_typedef = base in _KNOWN_POINTER_TYPEDEFS or bool(
        re.fullmatch(r"(?:L?P(?:C)?|H)[A-Z][A-Z0-9_]*", base)
    )
    if (
        "*" not in c_type
        and base not in scalar_types
        and base not in allowed_pod_types
        and base not in _POD_LEAF_TYPES
        and base not in island_scalar_types
        # A struct the ISLANDS agree on, held BY VALUE. Previously admitted only as an ATL
        # container's element type (see the field note on island_struct_types), which left a
        # plain embedded POD member rejected — RecycleBin5File's `DELETED_FILE_RECORD
        # m_deletedFile`, the on-disk INFO2 record. Admitting it is not a loosening: the island
        # carries that layout generated per-arch from the real header, `#[repr(C, packed)]` and
        # all, so there is strictly LESS to infer than for a scalar typedef, whose width has to
        # be agreed across arches first. A type the islands do NOT define still fails loudly.
        and base not in island_struct_types
        and not pointer_typedef
    ):
        raise CppComError(f"state field type is not proven C scalar/pointer state: {declaration}")
    return PlainStateField(declaration=text, c_type=c_type, name=name)


def _pod_definition(text: str, name: str) -> tuple[str, bool] | None:
    patterns = (
        re.compile(
            rf"typedef\s+struct(?:\s+[A-Za-z_]\w*)?\s*\{{(?P<body>[^{{}}]*)\}}\s*"
            rf"(?P<aliases>[^;]*\b{re.escape(name)}\b[^;]*)\s*;",
            re.S,
        ),
        re.compile(rf"\bstruct\s+{re.escape(name)}\s*\{{(?P<body>[^{{}}]*)\}}\s*;", re.S),
        # `typedef struct TAG { … } ALIAS, *PALIAS;` asked for by its TAG. The first pattern
        # finds this record by one of its ALIASES and the second wants a plain
        # `struct NAME { … };` with nothing between `}` and `;`, so a tag-named typedef
        # matched neither — and `source_pod_names` collects exactly those tags, so every such
        # record was discovered and then immediately failed to define.
        re.compile(
            rf"typedef\s+struct\s+{re.escape(name)}\s*\{{(?P<body>[^{{}}]*)\}}\s*[^;]*;",
            re.S,
        ),
    )
    for pattern in patterns:
        match = pattern.search(text)
        if match is None:
            continue
        prefix = text[: match.start()]
        packed = prefix.rfind("pshpack1.h") > prefix.rfind("poppack.h")
        return match.group("body"), packed
    return None


def _plain_member_class_definition(text: str, name: str) -> tuple[str, bool] | None:
    """Return field-only layout for a non-polymorphic embedded C++ class.

    Constructors, destructors, and ordinary methods do not affect object layout.
    This deliberately accepts only an unbased class with no virtual methods; its
    fields still pass the ordinary scalar/POD parser before the type is admitted.
    """
    match = next((item for item in _CLASS_HEAD.finditer(text) if item.group("name") == name), None)
    if match is None:
        return None
    raw_bases = (match.group("bases") or "").lstrip(":").strip()
    if raw_bases:
        return None
    body, _ = _balanced_block(text, match.end() - 1)
    if re.search(r"\bvirtual\b", body):
        return None
    fields = _state_fields(body)
    if not fields:
        return None
    return "; ".join(fields) + ";", False


# `struct|union|enum|class NAME` -> `NAME`. `\s+` after the keyword is required, so a member
# actually called `structFoo` is untouched.
_ELABORATED_TYPE_RE = re.compile(r"\b(?:struct|union|enum|class)\s+")


def _class_head_bases(header: Path, class_name: str) -> list[str]:
    """The base-specifier list of a class, as written."""
    text = header.read_text(encoding="utf-8", errors="replace")
    for match in _CLASS_HEAD.finditer(text):
        if match.group("name") == class_name:
            raw = (match.group("bases") or "").lstrip(":").strip()
            return [
                re.sub(r"^(?:public|protected|private)\s+", "", item.strip())
                for item in _split_args(raw)
                if item.strip()
            ]
    return []


def _window_base_adds_no_state(source: Path, base_name: str) -> bool:
    """True if an ATL window wrapper contributes NO storage beyond `CWindow::m_hWnd`.

    `CWindowImpl<T, CWindow, Traits>` is admitted below with a fixed two-field layout
    (`m_hWnd` + `AtlWindowState`). `CMenuToolbarBase` derives instead from
    `CWindowImplBaseT<CToolbar<DWORD_PTR>, CControlWinTraits>` -- the very same ATL
    shape with a different wrapper in the TBase slot -- so the only question that
    separates the two is whether that wrapper adds fields of its own.

    MEASURED, not assumed. The wrapper's body is read out of the tree and must declare
    no state fields, and the same is then required of ITS bases, so storage cannot
    simply hide one level down. `CToolbar<T>` passes because it is methods only over
    the inherited `m_hWnd`, terminating at `CWindow`. A wrapper that declares even one
    field fails and stays a loud preflight error -- silently admitting it would shift
    every field of the derived class by that wrapper's size, which is not a compile
    error anywhere, just wrong bytes.
    """
    seen: set[str] = set()
    queue = [base_name]
    while queue:
        name = queue.pop()
        if name in seen:
            continue
        seen.add(name)
        # `CWindow` is the terminator: its one field IS the `m_hWnd` the caller emits.
        # `CMessageMap` is ATL's pure-virtual dispatch interface and declares no data.
        if name in {"CWindow", "CMessageMap"}:
            continue
        # Strip a template argument list -- `CToolbar<DWORD_PTR>` is declared as
        # `template<class T> class CToolbar`, so the definition is under the bare name.
        bare = re.sub(r"\s*<.*", "", name).strip()
        if not re.fullmatch(r"[A-Za-z_]\w*", bare):
            return False
        try:
            header = find_class_header(source, bare)
            body = _find_class_text(header, bare)
        except (CppComError, OSError):
            return False
        if _state_fields(body):
            return False
        queue.extend(_class_head_bases(header, bare))
    return True


def _sdk_wide_string_macros(source_path: Path, used_words: set[str]) -> dict[str, tuple[int, ...]]:
    """Referenced SDK `#define NAME L"..."` macros, as UTF-16 unit tuples.

    The TCHAR-neutral spelling is what C++ writes (`WC_LISTVIEW`), and under UNICODE it is
    the `...W` macro. The ISLAND cannot supply these: it is generated without `-DUNICODE`,
    so `WC_LISTVIEWW` comes out as `&[u8; 14] = b"SysListView32\0"` -- the ANSI bytes, under
    the wide name. Handing that to `FindWindowExW` makes it read UTF-16 out of ASCII bytes,
    which is not a type error in C and not a compile error here either; `rosctrls.rs` already
    writes `WC_LISTVIEW` out by hand for exactly this reason.

    Only a fully literal definition is admitted, and the `...W` spelling is the authority.
    """
    repo_root = next(
        (parent for parent in source_path.parents if (parent / "sdk" / "include").is_dir()), None
    )
    if repo_root is None or not used_words:
        return {}
    found: dict[str, tuple[int, ...]] = {}
    for name in sorted(used_words):
        if not re.fullmatch(r"[A-Z][A-Z0-9_]*", name):
            continue
        pattern = re.compile(
            rf'^[ \t]*#[ \t]*define[ \t]+{re.escape(name)}W?[ \t]+(L"(?:\\.|[^"\\])*")[ \t]*$',
            re.M,
        )
        values = set()
        for header in _header_files(repo_root / "sdk" / "include", "*.[hi]"):
            text = header.read_text(encoding="utf-8", errors="replace")
            if name not in text:
                continue
            for token in pattern.findall(text):
                values.add(_wide_literal_units(token))
        # Every spelling must agree, exactly as for a numeric bound.
        if len(values) == 1:
            found[name] = values.pop()
    return found


def _expand_declarators(declaration: str) -> list[str]:
    """`INT8 SortCol, SortReverse` -> two declarations, one per declarator.

    C lets one statement declare several members, and a struct that uses it was being
    DROPPED ENTIRELY: the field parser raised on the comma, and the POD renderers swallow
    that with `except CppComError: continue`, so `_FILE_TYPE_GLOBALS` (whose only sin is
    `INT8 SortCol, SortReverse;`) silently never reached the output. The failure surfaced
    far away as `cannot find type PFILE_TYPE_GLOBALS` in an unrelated struct's field.

    Commas INSIDE `<...>`, `(...)` or `[...]` are not declarator separators
    (`CSimpleArray<A, B> m_x`, `WCHAR name[A, B]`), so nesting is tracked rather than the
    string split. A declarator may carry its own `*` or `[N]`, both of which bind to the
    NAME and not the type, so each one keeps them: `T *a, b[4]` is a pointer and an array.
    """
    depth = 0
    parts: list[str] = []
    current: list[str] = []
    for ch in declaration:
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(current))
            current = []
            continue
        current.append(ch)
    parts.append("".join(current))
    if len(parts) == 1:
        return [declaration]
    head = parts[0].strip()
    # The type is everything before the first declarator's name/`*`/`[`.
    match = re.match(r"(?P<type>.*?)(?P<decl>\**\s*[A-Za-z_]\w*\s*(?:\[[^\]]*\])?)\s*$", head)
    if match is None:
        return [declaration]
    base_type = match.group("type").strip()
    if not base_type:
        return [declaration]
    out = [head]
    for extra in parts[1:]:
        extra = extra.strip()
        if not extra:
            continue
        out.append(f"{base_type} {extra}")
    return out


def _module_private_int_macros(source_path: Path, names: set[str]) -> dict[str, int]:
    """Integer macros defined in the MODULE's own headers, not in `sdk/include`.

    shell32's `RECYCLEBINMAXDRIVECOUNT ('Z' - 'A' + 1)` lives in
    `shellrecyclebin/recyclebin.h`: it is compiler-facing, arch-independent, and reachable
    from nowhere else, so the bodies that size an array by it need it emitted here. Every
    spelling must agree and the expression must evaluate, exactly as for an array bound.
    """
    if not names:
        return {}
    module_root = next(
        (
            parent
            for parent in source_path.parents
            if parent.parent is not None and parent.parent.name in {"win32", "win32ss"}
        ),
        None,
    )
    if module_root is None or not module_root.is_dir():
        return {}
    values: dict[str, int] = {}
    for name in sorted(names):
        pattern = re.compile(
            rf"^[ \t]*#[ \t]*define[ \t]+{re.escape(name)}[ \t]+(.+?)[ \t]*$", re.M
        )
        found: set[int] = set()
        for header in module_root.rglob("*.[hi]"):
            text = header.read_text(encoding="utf-8", errors="replace")
            for replacement in pattern.findall(text):
                replacement = re.sub(r"//.*$|/\*.*?\*/", "", replacement).strip()
                value = _eval_c_int_constexpr(replacement)
                if value is not None:
                    found.add(value)
        if len(found) == 1:
            values[name] = found.pop()
    return values


def _typedef_aliases(source: Path, tag: str) -> list[tuple[str, bool]]:
    """`(name, is_pointer)` for each alias a `typedef struct TAG { … } A, *PA;` declares."""
    try:
        text = source.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    # BALANCED, not `[^{}]*`: a C++ struct body routinely contains braces of its own.
    # `_FILE_TYPE_ENTRY` declares six inline methods, so a non-nesting body pattern matched
    # nothing at all and the aliases were silently not emitted.
    head = re.search(rf"typedef\s+struct\s+{re.escape(tag)}\s*\{{", text)
    if head is None:
        return []
    _body, end = _balanced_block(text, head.end() - 1)
    tail = text[end:]
    semi = tail.find(";")
    if semi < 0:
        return []
    aliases = []
    for raw in tail[1:semi].split(","):
        item = raw.strip()
        pointer = item.startswith("*")
        name = item.lstrip("*").strip()
        if re.fullmatch(r"[A-Za-z_]\w*", name):
            aliases.append((name, pointer))
    return aliases


def _local_include_closure(source: Path, header: Path) -> list[Path]:
    """The TU's own `#include "..."` closure, restricted to its sibling directories.

    A by-value member of a class the TU does NOT declare itself is the common shape
    (`CDirectoryWatcher` holds a `CDirectoryList m_dir_list`, declared in the sibling
    `CDirectoryList.h`). Reading only the .cpp and its matching .h left that class
    invisible, and the state-field parser then rejected the member as "not proven C
    scalar/pointer state" -- reading as an untranslatable TU when the real answer was
    that nobody had opened the header next door.

    Deliberately QUOTED includes only, and only files that resolve inside a directory
    the TU already occupies. System/SDK headers are the `_pod_definition` scan's job
    further down; letting this walk into `sdk/include` would pull a large, mostly
    irrelevant closure into every layout decision.
    """
    roots = {path.parent for path in (source, header)}
    seen: dict[Path, None] = {}
    queue = [path for path in dict.fromkeys((source, header)) if path.is_file()]
    while queue:
        current = queue.pop(0)
        current = current.resolve()
        if current in seen:
            continue
        seen[current] = None
        text = current.read_text(encoding="utf-8", errors="replace")
        for quoted in re.findall(r'^\s*#\s*include\s*"([^"]+)"', text, re.M):
            for root in roots | {current.parent}:
                candidate = (root / quoted).resolve()
                if candidate.is_file() and any(
                    parent in {item.resolve() for item in roots} for parent in candidate.parents
                ):
                    queue.append(candidate)
                    break
    return list(seen)


def _discover_pod_state_types(
    source: Path, header: Path, declarations: list[str]
) -> tuple[list[PodStateType], list[EnumStateType]]:
    """Find simple structs/enums reachable from class-owned state.

    Pointer-owned records matter too: translated workers dereference them even
    though they do not contribute inline bytes to the COM owner.
    """

    def named_types(declarations: Sequence[str]) -> list[str]:
        """Type names a set of member declarations refers to."""
        found = []
        for declaration in declarations:
            # An ATL container's ELEMENT type is state too, and it hides inside the template
            # argument where the plain `Type name` pattern below cannot see it. Missing it does
            # not fail loud: the container preflight then rejects the element as "not a proven
            # layout" and the whole TU reads as untranslatable, when the real answer is that
            # nobody looked. `CAtlList<DynamicShellEntry>` is the shape.
            container = _RAII_CONTAINER_FIELD_RE.fullmatch(declaration)
            if container is not None and container.group("template") in _RAII_CONTAINER_LOWERINGS:
                found.append(container.group("element"))
                continue
            declaration = _ELABORATED_TYPE_RE.sub("", declaration)
            match = re.fullmatch(
                r"\s*(?P<type>[A-Za-z_]\w*)\s*(?:\*+\s*)?[A-Za-z_]\w*\s*", declaration
            )
            if match:
                c_type = match.group("type")
                pointer_typedef = bool(re.fullmatch(r"(?:L?P(?:C)?|H)[A-Z][A-Z0-9_]*", c_type))
                if (
                    c_type not in _RUST_SCALARS
                    and c_type not in _POD_LEAF_TYPES
                    and not pointer_typedef
                ):
                    found.append(c_type)
        return found

    candidates = named_types(declarations)
    if not candidates:
        return [], []
    local_texts = [
        path.read_text(encoding="utf-8", errors="replace")
        for path in _local_include_closure(source, header)
    ]
    repo_root = next(
        (parent for parent in source.parents if (parent / "sdk" / "include").is_dir()), None
    )
    definitions: dict[str, tuple[str, bool]] = {}
    # A worklist, not a flat pass: a discovered class's own fields name further types, and
    # they are just as load-bearing. `CDirectoryList` holds `CSimpleArray<CDirectoryItem>`,
    # so stopping at depth one proves the outer class and then rejects its element.
    pending = list(dict.fromkeys(candidates))
    visited: set[str] = set()
    while pending:
        name = pending.pop(0)
        if name in visited:
            continue
        visited.add(name)
        definition = next(
            (
                found
                for text in local_texts
                if (
                    found := _pod_definition(text, name)
                    or _plain_member_class_definition(text, name)
                )
                is not None
            ),
            None,
        )
        if definition is None and repo_root is not None:
            for candidate in _header_files(repo_root / "sdk" / "include", "*.[hi]"):
                text = candidate.read_text(encoding="utf-8", errors="replace")
                if not re.search(rf"\b{re.escape(name)}\b", text):
                    continue
                definition = _pod_definition(text, name)
                if definition is not None:
                    break
        if definition is None:
            continue
        definitions[name] = definition
        body = re.sub(r"/\*.*?\*/|//[^\n]*|#\s*include[^\n]*", " ", definition[0], flags=re.S)
        pending.extend(
            named_types([" ".join(raw.split()) for raw in body.split(";") if raw.strip()])
        )

    enum_names: set[str] = set()
    for body, _ in definitions.values():
        for raw in body.split(";"):
            declaration = " ".join(raw.split())
            field = re.match(r"(?:const\s+)?([A-Za-z_]\w*)\b", declaration)
            if field:
                enum_names.add(field.group(1))
    enums: list[EnumStateType] = []
    all_local_text = "\n".join(local_texts)
    for name in sorted(enum_names):
        match = re.search(
            rf"\benum\s+{re.escape(name)}\s*\{{(?P<body>[^{{}}]*)\}}\s*;",
            all_local_text,
            re.S,
        )
        if match is None:
            continue
        value = -1
        variants: list[tuple[str, int]] = []
        valid = True
        for raw in _split_args(match.group("body")):
            item = re.fullmatch(
                r"([A-Za-z_]\w*)(?:\s*=\s*(-?(?:0[xX][0-9A-Fa-f]+|\d+)))?",
                raw.strip(),
            )
            if item is None:
                valid = False
                break
            value = int(item.group(2), 0) if item.group(2) is not None else value + 1
            variants.append((item.group(1), value))
        if valid and variants:
            enums.append(EnumStateType(name, tuple(variants)))

    specs = []
    allowed_local = set(definitions) | {item.name for item in enums}
    for name, (body, packed) in definitions.items():
        body = re.sub(r"/\*.*?\*/|//[^\n]*|#\s*include[^\n]*", " ", body, flags=re.S)
        fields = []
        valid = True
        for raw in body.split(";"):
            statement = " ".join(raw.split())
            if not statement:
                continue
            # One C statement can declare several members; each becomes its own field.
            for declaration in _expand_declarators(statement):
                try:
                    fields.append(
                        _parse_plain_state_field(
                            declaration,
                            allowed_pod_types=_POD_LEAF_TYPES | allowed_local,
                        )
                    )
                except CppComError:
                    valid = False
                    break
            if not valid:
                break
        if valid and fields:
            specs.append(PodStateType(name=name, fields=tuple(fields), packed=packed))
    return specs, enums


def _base_is_widl_interface(source: Path, base: str) -> bool:
    """True if `base` is a real COM interface with a flattened WIDL vtable.

    Used to tell a non-QI'd interface base (materialize its vtable subobject in the
    layout, e.g. IContextMenuCB) from genuine custom/stateful C++ inheritance (a
    loud failure). Restricted to interface-shaped names and confirmed against the
    generated SDK headers so a non-interface base is never silently accepted.
    """
    if not re.fullmatch(r"I[A-Z]\w*", base):
        return False
    repo_root = next(
        (parent for parent in source.parents if (parent / "sdk" / "include").is_dir()), None
    )
    if repo_root is None:
        return False
    for arch in ("amd64", "i386", "arm64"):
        try:
            discover_widl_header(repo_root, arch, interface=base, extra_roots=[source.parent])
            return True
        except CppComError:
            continue
    return _base_is_declared_interface(source, base)


def _base_is_declared_interface(source: Path, base: str) -> bool:
    """True if `base` is a COM interface declared by the DECLARE_INTERFACE_ macro.

    The WIDL probe above only sees interfaces generated from an .idl. A module that
    declares its own interfaces in a hand-written header — shellrecyclebin's IRecycleBin /
    IRecycleBin5 / IRecycleBinEnumList / IRecycleBinFile — is the same THING: the macro
    expands to a pure-virtual class in C++ and to an `lpVtbl` struct in C, and either way it
    has no data members, so materializing the vtable subobject in the layout is exactly as
    sound as for a WIDL one. Without this the class path rejected every such base as
    "custom/inherited ... needs deterministic inherited layout mapping", which pushed the
    caller toward --plain-without-a-scaffold and shipped objects whose lpVtbl was never set.

    Deliberately strict, because the whole point of the surrounding check is that a genuine
    stateful C++ base stays a LOUD failure: the declaration has to be found verbatim, and the
    search is bounded to the module's own directory tree and the SDK headers.
    """
    pattern = re.compile(rf"\bDECLARE_INTERFACE_?\s*\(\s*{re.escape(base)}\s*[,)]")
    roots = [source.parent]
    repo_root = next(
        (parent for parent in source.parents if (parent / "sdk" / "include").is_dir()), None
    )
    if repo_root is not None:
        roots.append(repo_root / "sdk" / "include")
    for root in roots:
        for header in _header_files(root):
            try:
                if pattern.search(header.read_text(encoding="utf-8", errors="replace")):
                    return True
            except OSError:
                continue
    return False


_SDK_TYPE_MACRO = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+(?P<name>\w+)[ \t]+(?P<target>\w+)[ \t]*$", re.M
)
_SDK_AW_MACRO = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+(?P<name>\w+)[ \t]+__MINGW_NAME_AW\((?P<target>\w+)\)[ \t]*$", re.M
)
_SDK_MACRO_ALIAS_CACHE: dict[tuple[str, bool | None], dict[str, str]] = {}
_MODULE_UNICODE_CACHE: dict[str, bool | None] = {}


def module_is_unicode(source: Path) -> bool | None:
    """Whether the module owning ``source`` compiles with UNICODE, per its CMakeLists.

    ``__MINGW_NAME_AW(LVITEM)`` is ``LVITEMW`` or ``LVITEMA`` depending on it, so the
    A/W half of the SDK's type names cannot be resolved without this. Returns None when
    no `set_module_type()` is found rather than assuming a half: an unresolved alias
    leaves an argument untyped, which is a loud ambiguity, while a WRONG half would
    silently bind the wrong overload.
    """
    key = str(source)
    if key in _MODULE_UNICODE_CACHE:
        return _MODULE_UNICODE_CACHE[key]
    verdict: bool | None = None
    for parent in source.parents:
        lists = parent / "CMakeLists.txt"
        if not lists.is_file():
            continue
        try:
            text = lists.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        match = re.search(r"set_module_type\s*\((?P<args>[^)]*)\)", text)
        if match is not None:
            verdict = "UNICODE" in match.group("args").split()
            break
    _MODULE_UNICODE_CACHE[key] = verdict
    return verdict


def sdk_type_macro_aliases(repo_root: Path, unicode: bool | None = None) -> dict[str, str]:
    """Object-like ``#define OLD NEW`` type aliases the SDK headers declare.

    bindgen renders declarations, never the preprocessor, so an island can NEVER prove
    one of these -- and Win32 is full of them: commctrl.h alone carries the whole legacy
    ``#define LV_HITTESTINFO LVHITTESTINFO`` family. C code uses the old spelling freely,
    so without this a struct the islands DO prove is unreachable under the name the
    source actually writes.

    Deliberately narrow. Only a bare identifier-to-identifier define is taken, and a name
    defined two different ways anywhere in the tree is DROPPED rather than picked between,
    because these feed overload binding where a wrong answer silently selects the wrong
    method. A dropped alias just leaves the argument untyped, which is the pre-existing
    loud ambiguity.
    """
    key = (str(repo_root), unicode)
    cached = _SDK_MACRO_ALIAS_CACHE.get(key)
    if cached is not None:
        return cached
    header_roots = [repo_root / "sdk/include"] + [
        repo_root / f"build-{arch}" / "sdk" / "include" / "psdk"
        for arch in ("i386", "amd64", "arm64")
    ]
    seen: dict[str, set[str]] = defaultdict(set)
    for root in header_roots:
        if not root.is_dir():
            continue
        for header in sorted(_header_files(root)):
            try:
                text = header.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            for match in _SDK_TYPE_MACRO.finditer(text):
                seen[match.group("name")].add(match.group("target"))
            if unicode is None:
                continue
            # `#define LVITEM __MINGW_NAME_AW(LVITEM)` — the A/W half is a build fact,
            # not a source one, so it is resolved only when the module settled it.
            for match in _SDK_AW_MACRO.finditer(text):
                suffix = "W" if unicode else "A"
                seen[match.group("name")].add(f"{match.group('target')}{suffix}")
    direct = {name: next(iter(targets)) for name, targets in seen.items() if len(targets) == 1}
    # `#define LV_ITEM LVITEM` sits on top of `#define LVITEM __MINGW_NAME_AW(LVITEM)`,
    # so the legacy spelling only reaches a real struct by following the chain.
    aliases: dict[str, str] = {}
    for name in direct:
        current, hops = name, 0
        while current in direct and hops < _ISLAND_ALIAS_MAX_HOPS:
            current = direct[current]
            hops += 1
        aliases[name] = current
    _SDK_MACRO_ALIAS_CACHE[key] = aliases
    return aliases


_ISLAND_POINTER_ALIAS = re.compile(
    r"^pub type (?P<name>\w+) = \*(?:mut|const) (?P<target>[\w:]+);", re.M
)
_ISLAND_POINTER_CACHE: dict[tuple[str, ...], dict[str, str]] = {}


def island_pointer_aliases(islands: Sequence[Path]) -> dict[str, str]:
    """Pointer typedefs the islands prove, as ``NAME -> "<pointee> *"``.

    Win32 names most pointers with a typedef that hides the star (`LPNMLISTVIEW`), so
    `params->iItem` cannot be typed from the spelling alone — and refusing to type it
    is what leaves an overload tied. Rendering the star back makes the ordinary
    pointer/member rules apply with no special case anywhere else.

    Intersected across arches on the same principle as the rest of the island readers.
    """
    key = tuple(str(path) for path in islands)
    cached = _ISLAND_POINTER_CACHE.get(key)
    if cached is not None:
        return cached
    per_arch: list[dict[str, str]] = []
    for path in islands:
        arch = re.fullmatch(r"island_(\w+)\.rs", path.name)
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return {}
        sources = [text]
        if arch is not None:
            workspace = path.parent.parent.parent
            for match in _ISLAND_SYS_REF.finditer(text[:8192]):
                companion = (
                    workspace
                    / f"{match.group('crate').replace('_', '-')}-sys"
                    / "src"
                    / f"bindgen_{arch.group(1)}.rs"
                )
                if companion.is_file():
                    sources.append(companion.read_text(encoding="utf-8", errors="replace"))
        found: dict[str, str] = {}
        for source_text in sources:
            for match in _ISLAND_POINTER_ALIAS.finditer(source_text):
                found.setdefault(match.group("name"), f"{match.group('target').split('::')[-1]} *")
        per_arch.append(found)
    if not per_arch:
        return {}
    shared = {
        name: target
        for name, target in per_arch[0].items()
        if all(other.get(name) == target for other in per_arch[1:])
    }
    _ISLAND_POINTER_CACHE[key] = shared
    return shared


def _source_sdk_type_aliases(source: Path) -> dict[str, str]:
    """The SDK macro type aliases visible to ``source``, at its module's A/W half."""
    repo_root = next(
        (parent for parent in source.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is None:
        return {}
    return sdk_type_macro_aliases(repo_root, module_is_unicode(source))


def _field_types_with_macro_aliases(
    source: Path, island_files: Sequence[Path]
) -> dict[str, dict[str, str]]:
    """Island-proven struct fields, reachable under the SDK's macro spellings too."""
    fields = dict(island_struct_field_types(island_files))
    repo_root = next(
        (parent for parent in source.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is None:
        return fields
    for name, target in sdk_type_macro_aliases(repo_root, module_is_unicode(source)).items():
        if target in fields:
            fields.setdefault(name, fields[target])
    return fields


def analyze_class(
    source: Path,
    class_name: str,
    *,
    object_map_source: Path | None = None,
    activation_source: Path | None = None,
    method_names: list[str] | None = None,
    island_files: Sequence[Path] = (),
) -> CppComClass:
    source = source.resolve()
    source_text = source.read_text(encoding="utf-8", errors="replace")
    header = find_class_header(source, class_name)
    body = _find_class_text(header, class_name)
    state_fields = _state_fields(body)
    entries, unsupported = _parse_com_map(body, class_name)
    bases = _class_bases(header, class_name)
    interfaces = {entry.interface for entry in entries}
    final_construct = bool(re.search(r"\bFinalConstruct\s*\(", body))
    if re.search(r"\bFinalRelease\s*\(", body):
        unsupported.append("custom ATL lifecycle method FinalRelease needs deterministic wiring")
    layout_bases: list[str] = []
    local_base_classes: list[str] = []
    base_worklist = list(bases)
    while base_worklist:
        raw_base = base_worklist.pop(0)
        base = re.sub(r"\b(?:public|private|protected|virtual)\b", "", raw_base)
        base = " ".join(base.split())
        if (
            base.startswith("CComCoClass<")
            or base.startswith("CComObjectRootEx<")
            or base in interfaces
        ):
            continue
        # ATL's IDispatchImpl<T, ...> contributes T's already-mapped interface
        # subobject plus one CComPtr<ITypeInfo> state field and five ordinary
        # method bodies. The template arguments are compile-time ABI facts, so
        # flatten that exact known template rather than treating it as opaque
        # inherited state. Other ATL templates remain loud failures.
        # The inner type is NOT required to be `I`-prefixed. shldisp.idl's automation
        # interfaces are `Folder`, `Folder2`, `FolderItem`, `FolderItems`, `FolderItemVerb`,
        # `FolderItemVerbs` -- no `I` anywhere -- and WIDL generates the same flattened vtable
        # for them as for `IShellDispatch4` in that same header. Demanding the prefix rejected
        # five real interfaces and reported "custom/inherited base class ... needs deterministic
        # inherited layout mapping", which reads as a MISSING FEATURE rather than a bad guess,
        # and held CFolder/CFolderItems/CFolderItemVerbs behind a gap that did not exist.
        # Nothing is loosened: the `&IID_\1` BACKREFERENCE still requires the IID to be named
        # after the interface, and the `in interfaces` test below still requires it to be one
        # this class actually declares in its COM map. A name shape was never the evidence.
        dispatch = re.fullmatch(
            r"IDispatchImpl\s*<\s*([A-Za-z_]\w*)\s*,\s*&IID_\1(?:\s*,.*)?\s*>",
            base,
        )
        if dispatch is not None and dispatch.group(1) in interfaces:
            template_name = "IDispatchImpl"
            template_header = find_class_header(source, template_name)
            template_body = _find_class_text(template_header, template_name)
            template_fields = _state_fields(template_body)
            state_fields = template_fields + state_fields
            if template_name not in local_base_classes:
                local_base_classes.append(template_name)
            continue
        # CWindowImpl's C++ thunk/message-map object layout is not carried into
        # Rust. Rust-owned shellmenu classes use a direct Win32 WNDPROC with the
        # owner pointer passed through CreateWindowEx; the only persistent base
        # state visible to class bodies is CWindow::m_hWnd. Admit only the exact
        # self/CWindow/traits form used by these classes. Window registration and
        # message dispatch remain deterministic scaffold/substrate work, never a
        # model-owned ABI guess.
        window_impl = re.fullmatch(
            rf"CWindowImpl\s*<\s*{re.escape(class_name)}\s*,\s*CWindow\s*,\s*"
            r"([A-Za-z_]\w*)\s*>",
            base,
        )
        # `CWindowImplBaseT<TBase, Traits>` is what `CWindowImpl` derives from, and a
        # class may name it directly: `CMenuToolbarBase : CWindowImplBaseT<
        # CToolbar<DWORD_PTR>, CControlWinTraits >`. Same ATL layout, same Rust model --
        # the derived-class parameter is absent because it only ever fed CRTP dispatch,
        # which `crate::atlwin` does through the window's own `AtlWindow` impl instead.
        #
        # Admitted only once the wrapper in the TBase slot is PROVEN stateless, so this
        # widens the spelling, not the layout claim.
        if window_impl is None:
            impl_base = re.fullmatch(
                r"CWindowImplBaseT\s*<\s*(?P<wrapper>.+?)\s*,\s*[A-Za-z_]\w*\s*>",
                base,
            )
            if impl_base is not None and _window_base_adds_no_state(
                source, impl_base.group("wrapper")
            ):
                window_impl = impl_base
        if window_impl is not None:
            # BOTH fields, not just m_hWnd. `AtlWindowState` is what C++ inherits from
            # CWindowImplRoot/CWindowImplBaseT -- the displaced superclass procedure, the
            # window state flags and the dispatch depth -- and `crate::atlwin` needs it to
            # exist on every ATL window class. CDefView carried it as a HAND EDIT inside a
            # GENERATED file, so regenerating CDefView would have silently dropped it and
            # taken the use-after-free guard in `atl_window_proc` with it. Emitting it here
            # makes that class regenerable and every later one correct by default.
            state_fields = ["HWND m_hWnd", "AtlWindowState m_atl", *state_fields]
            continue
        # A real COM interface base that the class implements but does not QI-expose
        # (absent from the COM map) is materialized in the layout, not QueryInterface.
        if base not in layout_bases and _base_is_widl_interface(source, base):
            layout_bases.append(base)
            continue
        # Concrete same-header base (CCopyMoveToMenu): flatten it into this
        # translation — its state precedes ours, its methods become ours (renamed at
        # scan-extend), and its OWN bases join the worklist for the same treatment.
        local = _local_concrete_base(source, header, base)
        if local is not None:
            if base in local_base_classes:
                continue
            _base_header, base_body, base_bases = local
            local_base_classes.append(base)
            base_fields = _state_fields(base_body)
            derived_names = {_state_field_name(decl) for decl in state_fields}
            for decl in base_fields:
                if _state_field_name(decl) in derived_names:
                    # Base methods touch the base copy, derived methods the derived
                    # copy — two same-named objects. Not mechanically flattenable;
                    # resolve the C++ (usually dead copy-paste dupes) first.
                    unsupported.append(
                        f"derived class shadows inherited state field `{decl}` of "
                        f"{base} — resolve the C++ shadowing before translation"
                    )
            state_fields = base_fields + state_fields
            base_worklist.extend(base_bases)
            final_construct = final_construct or bool(
                re.search(r"\bFinalConstruct\s*\(", base_body)
            )
            if re.search(r"\bFinalRelease\s*\(", base_body):
                unsupported.append(
                    "custom ATL lifecycle method FinalRelease needs deterministic wiring"
                )
            continue
        # Anything else (ATL templates, out-of-header bases) is genuine
        # custom/stateful inheritance and stays a loud failure.
        unsupported.append(
            f"custom/inherited base class {base} needs deterministic inherited layout mapping"
        )
    pod_state_types, enum_state_types = _discover_pod_state_types(source, header, state_fields)
    array_constants = _numeric_array_constants(source, state_fields)
    resource = re.search(r"\bDECLARE_REGISTRY_RESOURCEID\s*\(\s*([^)\s]+)\s*\)", body)
    not_agg = bool(re.search(r"\bDECLARE_(?:CENTRAL_INSTANCE_)?NOT_AGGREGATABLE\s*\(", body))
    atl_refcounted = any(
        b.replace("public ", "").strip().startswith("CComObjectRootEx<") for b in bases
    )
    clsid = None
    # `ShellObjectCreatorInit<T, T1, ...>` — shellutils.h overloads the template on the
    # INIT-ARGUMENT types as well as the class, and a caller that spells them explicitly
    # (`ShellObjectCreatorInit<CDesktopBrowser, IShellDesktopTray*>(Tray, riid, ppv)`) put
    # the class name in a template list that did not end after it. Keying on `<T>` alone
    # read that as no internal activation at all, so a class constructed one line below its
    # own declaration reported "internal COM-object construction is not mapped".
    #
    # The trailing arguments are deliberately NOT captured: they are the Initialize
    # parameter types, and the arity check below already proves them against the real
    # `Initialize` signature. `[^<>]*` refuses a nested template rather than mis-splitting
    # one, which fails loudly and correctly.
    activation_re = re.compile(
        rf"\bShellObjectCreator(?P<init>Init)?\s*<\s*{re.escape(class_name)}\s*"
        rf"(?:,[^<>]*)?>\s*\("
    )
    masked_source = _masked_cpp(source_text)
    activation_hits = list(activation_re.finditer(masked_source))
    activation_modes = {bool(hit.group("init")) for hit in activation_hits}
    if len(activation_modes) > 1:
        unsupported.append(
            "mixed ShellObjectCreator/ShellObjectCreatorInit activation needs an explicit mapping"
        )
    # ATL's own creator is the SECOND internal-activation shape, and it is the one a
    # class with no CLSID at all uses: `CFileSysBindData::_CreatorClass::CreateInstance(
    # NULL, IID_PPV_ARG(IFileSystemBindData, &p))`. Its signature is ATL's
    # `CComCreator<...>::CreateInstance(void *pv, REFIID riid, LPVOID *ppv)`, which is
    # exactly the `(outer, riid, ppv)` factory the internal path already emits — so this
    # is a DETECTION gap, not an emission one, and widening it needs no new codegen.
    #
    # It stays behind `--internal` rather than being inferred, for the same reason
    # `--plain` is asserted: "this class is activated only from inside the DLL" is a claim
    # about the whole tree, and a file that merely looks self-contained is not proof.
    # Before flipping such a TU, check it is a link LEAF (`llvm-nm --defined-only` on its
    # object, intersected with every other object's undefined set) — for a class with no
    # CLSID that intersection IS the activation surface.
    creator_class_re = re.compile(
        rf"\b{re.escape(class_name)}\s*::\s*_CreatorClass\s*::\s*CreateInstance\s*\("
    )
    creator_class_hits = list(creator_class_re.finditer(masked_source))
    # ATL's raw `new CComObject<T>()` — the THIRD internal-activation shape, and the most
    # literal one: no factory at all, the caller allocates and then drives `Init`/`AddRef`
    # itself. `CFolderItem::Verbs` does exactly that with CFolderItemVerbs.
    #
    # It is still "this class has no CLSID and is constructed inside the DLL", which is the
    # only question this flag answers, so it joins the other two rather than needing its own
    # concept. What differs is the CALLER's shape, not the class's: a `ShellObjectCreatorInit`
    # caller is one expression while this one is a statement sequence, and that difference
    # lives at the call site, not here.
    # Two spellings, both ATL's CComObject: the raw `new CComObject<T>()` expression and the
    # static factory `CComObject<T>::CreateInstance(&p)`. They differ only in who allocates —
    # BOTH hand the caller an object at refcount 0 and leave it to AddRef, which CStartMenu.cpp
    # states outright next to its call ("CreateInstance returns object with 0 ref count").
    com_object_new_re = re.compile(
        rf"\bnew\s+CComObject\s*<\s*{re.escape(class_name)}\s*>\s*\("
        rf"|\bCComObject\s*<\s*{re.escape(class_name)}\s*>\s*::\s*CreateInstance\s*\("
    )
    # A class is often constructed by a DIFFERENT TU than the one that defines it —
    # `CFolderItem::Verbs` (CFolderItems.cpp) does the `new CComObject<CFolderItemVerbs>()`
    # for a class defined in CFolderItemVerbs.cpp. Scanning only the class's own file then
    # reports "internal COM-object construction is not mapped" for a class whose activation
    # is perfectly ordinary and simply lives next door.
    #
    # NAMED, not searched: the caller is supplied with `--activation-source` rather than
    # inferred by sweeping the directory, for the same reason `object_map_source` is named.
    # It contributes to DETECTION only — the initializer-argument extraction below stays on
    # the class's own file, so a foreign file can never silently redefine the ctor shape.
    activation_scan = masked_source
    if activation_source is not None and activation_source.is_file():
        activation_scan += "\n" + _masked_cpp(
            activation_source.read_text(encoding="utf-8", errors="replace")
        )
    com_object_new_hits = list(com_object_new_re.finditer(activation_scan))

    # The FOURTH shape, and the only one that never spells the class in the call:
    # shellutils.h also overloads `ShellObjectCreator(CComPtr<T> &objref)`, deducing T
    # from the argument. `CRecycleBinFolderViewCB_CreateInstance_RustBridge` uses it, so
    # every regex above -- all of which key on `<ClassName>` -- saw nothing, and a class
    # constructed by name two lines from its own declaration reported "internal
    # COM-object construction is not mapped".
    #
    # Detection ONLY, like the two shapes above it: this overload takes no init args
    # (it hands back a bare object at refcount 1 and the caller drives everything else),
    # so it deliberately stays out of `activation_hits` and cannot affect the
    # Init-vs-plain mode or the initializer-arity proof.
    #
    # The variable must be declared `CComPtr<ClassName>` in the SAME scanned text, so
    # this cannot match a creator call for some other class that happens to be nearby.
    deduced_creator_vars = set(
        re.findall(
            rf"\bCComPtr\s*<\s*{re.escape(class_name)}\s*>\s*([A-Za-z_]\w*)\s*[;,=]",
            activation_scan,
        )
    )
    deduced_creator_hits = [
        name
        for name in deduced_creator_vars
        if re.search(rf"\bShellObjectCreator\s*\(\s*{re.escape(name)}\s*\)", activation_scan)
    ]

    internal_shell_object_creator = (
        bool(activation_hits)
        or bool(creator_class_hits)
        or bool(com_object_new_hits)
        or bool(deduced_creator_hits)
        or bool(activation_re.search(activation_scan))
        or bool(creator_class_re.search(activation_scan))
    )
    internal_initializer_params = None
    if activation_hits and activation_hits[0].group("init"):
        activation_arg_lists = []
        for activation_hit in activation_hits:
            opening = masked_source.find("(", activation_hit.start())
            closing = _matching_paren(masked_source, opening)
            activation_arg_lists.append(_split_args(source_text[opening + 1 : closing]))
        initialize_hits = list(
            re.finditer(
                r"\bHRESULT\s+(?:(?:WINAPI|STDMETHODCALLTYPE|CALLBACK)\s+)?"
                r"Initialize\s*\((?P<params>[^()]*)\)",
                body,
            )
        )
        if len(initialize_hits) != 1:
            unsupported.append(
                "ShellObjectCreatorInit requires exactly one source-proven HRESULT Initialize"
            )
        else:
            params = tuple(
                _parse_param(raw, index)
                for index, raw in enumerate(_split_args(initialize_hits[0].group("params")))
                if raw.strip() and raw.strip() != "void"
            )

            def effective_creator_arity(args: list[str]) -> int:
                # A ppv macro is one C++ argument but several ABI ones, and the
                # family members differ in HOW many — so the count has to come
                # from the macro's own expansion, not from a flat +1.
                tail = (
                    re.fullmatch(rf"\s*({_PPV_ARG_MACRO})\s*\([^()]+\)\s*", args[-1], re.DOTALL)
                    if args
                    else None
                )
                extra = 1 + _PPV_ARG_MIDDLES[tail.group(1)].count(",") if tail else 0
                return len(args) + extra

            if any(
                effective_creator_arity(args) != len(params) + 2 for args in activation_arg_lists
            ):
                unsupported.append(
                    "ShellObjectCreatorInit argument count does not match Initialize + riid/ppv"
                )
            else:
                internal_initializer_params = params

    stgmedium_lifecycle_fields: list[str] = []
    stgmedium_fields = [
        match.group(1)
        for declaration in state_fields
        if (match := re.fullmatch(r"\s*STGMEDIUM\s+([A-Za-z_]\w*)\s*", declaration))
    ]
    if stgmedium_fields:
        from .scanner import scan_file

        class_workers = [
            worker for worker in scan_file(source).functions if worker.cpp_class == class_name
        ]
        constructors = [worker for worker in class_workers if worker.is_ctor]
        destructors = [worker for worker in class_workers if worker.is_dtor]
        for field_name in stgmedium_fields:
            ctor_proven = any(
                re.search(
                    rf"\b{re.escape(field_name)}\.tymed\s*=\s*TYMED_NULL\b",
                    worker.body_text,
                )
                for worker in constructors
            )
            dtor_proven = any(
                "DestroyCIDA" in worker.body_text
                and re.search(rf"\b{re.escape(field_name)}\b", worker.body_text)
                for worker in destructors
            )
            if ctor_proven and dtor_proven:
                stgmedium_lifecycle_fields.append(field_name)
    # The primary constructor's parameters, for `_new`. Read from the scanner rather than
    # re-parsed here so the names and types match the worker signature the model was given.
    ctor_params: tuple[tuple[str, str], ...] = ()
    try:
        from .scanner import scan_file as _scan_file

        _ctors = [
            worker
            for worker in _scan_file(source).functions
            if worker.cpp_class == class_name and worker.is_ctor
        ]
        if _ctors:
            ctor_params = tuple(_ctors[0].params)
    except (OSError, ValueError):
        ctor_params = ()

    activation_sources = [source]
    if header != source:
        activation_sources.append(header)
    if object_map_source and object_map_source.is_file():
        activation_sources.append(object_map_source.resolve())
    for activation_source in dict.fromkeys(activation_sources):
        object_map = activation_source.read_text(encoding="utf-8", errors="replace")
        hit = re.search(
            rf"\bOBJECT_ENTRY(?:_AUTO)?\s*\(\s*([^,]+?)\s*,\s*"
            rf"{re.escape(class_name)}(?:RustEntry)?\s*\)",
            object_map,
        )
        if hit:
            clsid = hit.group(1).strip()
            break
    return CppComClass(
        name=class_name,
        source=source,
        header=header,
        com_map=entries,
        registry_resource=resource.group(1) if resource else None,
        not_aggregatable=not_agg,
        object_map_clsid=clsid,
        internal_shell_object_creator=internal_shell_object_creator,
        atl_refcounted=atl_refcounted,
        internal_initializer_params=internal_initializer_params,
        stgmedium_lifecycle_fields=tuple(stgmedium_lifecycle_fields),
        final_construct=final_construct,
        method_names=method_names or [],
        base_classes=bases,
        state_fields=state_fields,
        pod_state_types=pod_state_types,
        enum_state_types=enum_state_types,
        array_constants=array_constants,
        ctor_params=ctor_params,
        virtual_methods=tuple(
            match.group("name")
            for match in re.finditer(r"\bvirtual\b[^;{()]*?(?P<name>~?[A-Za-z_]\w*)\s*\(", body)
        ),
        unsupported=unsupported,
        layout_bases=layout_bases,
        local_base_classes=local_base_classes,
        island_scalar_types=island_integer_aliases(island_files),
        island_struct_types=island_struct_types(island_files),
        island_struct_field_types=_field_types_with_macro_aliases(source, island_files),
        sdk_type_aliases={
            **island_pointer_aliases(island_files),
            **_source_sdk_type_aliases(source),
        },
    )


_VTBL_BLOCK = re.compile(
    r"typedef\s+struct\s+(?P<name>[A-Za-z_]\w*Vtbl)\s*\{(?P<body>.*?)\}\s*(?P=name)\s*;",
    re.S,
)
_VTBL_METHOD = re.compile(
    r"(?P<ret>[A-Za-z_]\w*(?:\s*\*)*)\s*"
    r"\(\s*STDMETHODCALLTYPE\s*\*(?P<name>[A-Za-z_]\w*)\s*\)\s*"
    r"\((?P<params>.*?)\)\s*;",
    re.S,
)
_DECLARE_METHOD = re.compile(
    r"STDMETHOD(?:(?:_\s*\(\s*(?P<ret>[^,]+?)\s*,\s*(?P<name2>\w+)\s*\))"
    r"|(?:\s*\(\s*(?P<name1>\w+)\s*\)))\s*"
    # PURE is OPTIONAL. It expands to nothing in C mode, so a header that omits it is a
    # typo no compiler can see -- and when this pattern required it, the non-greedy params
    # ran straight through the unmatched declaration to the next `) PURE;` and swallowed
    # the following method whole. recyclebin_v5.h's `STDMETHOD(EmptyRecycleBin)(THIS);`
    # ate EnumObjects that way, leaving a 10-slot vtable for an 11-method interface with
    # every slot after it shifted up by one. _assert_no_dropped_slots is the backstop.
    r"\((?P<params>.*?)\)\s*(?:PURE\s*)?;",
    re.S,
)
_SAL_ANNOTATION = re.compile(r"\b_[A-Za-z][A-Za-z0-9_]*_(?:\s*\([^)]*\))?\s*")
# ntdef.h defines these to nothing, so they are annotations exactly like the `_In_` family
# above -- but they are not underscore-flanked, so _SAL_ANNOTATION cannot see them. Left in
# place they become part of the parsed type: `IN REFIID` and `*mut *mut OUT void`.
_CLASSIC_SAL = re.compile(r"\b(?:IN|OUT|INOUT|OPTIONAL)\b\s*")
_STDMETHOD_TOKEN = re.compile(r"\bSTDMETHOD_?\s*\(")


def _assert_no_dropped_slots(interface: str, header: Path, body: str, parsed: int) -> None:
    """A vtable is positional, so a dropped slot is not a partial result -- it renumbers
    every slot after it. That can never degrade to a warning."""
    declared = len(_STDMETHOD_TOKEN.findall(body))
    if declared != parsed:
        raise CppComError(
            f"{interface}: {header.name} declares {declared} vtable method(s) but "
            f"{parsed} parsed. A vtable is positional -- a missed declaration shifts "
            f"every slot after it, so calls dispatch to the wrong method. Check for a "
            f"declaration this parser does not recognize."
        )


def _vtbl_declared_slots(body: str) -> int:
    """Count vtable SLOT declarations in a WIDL ``InterfaceVtbl`` body.

    Counting every ``STDMETHODCALLTYPE`` token over-counts, because a slot's PARAMETER can be a
    function pointer carrying the same calling convention. oleidl.h's IViewObject is the case:

        HRESULT (STDMETHODCALLTYPE *Draw)(
            ...
            BOOL (STDMETHODCALLTYPE *pfnContinue)(ULONG_PTR dwContinue),   <-- a parameter
            ULONG_PTR dwContinue);

    That reads as 10 tokens for an interface with 9 slots (QueryInterface, AddRef, Release,
    Draw, GetColorSet, Freeze, Unfreeze, SetAdvise, GetAdvise — the documented layout), so the
    cross-check fired on a CORRECT parse and refused the whole translation. A false positive
    here is not harmless: this guard is the thing standing between us and a silently shifted
    vtable, so it has to be trustworthy enough that nobody is tempted to loosen it.

    Depth separates the two cleanly. A slot's token sits inside the declarator parens, at depth
    1 of the struct body; a parameter's sits inside the slot's parameter list and then its own
    declarator, i.e. deeper. Counting only depth 1 keeps a genuinely missed slot mismatching.
    """
    depth = 0
    slots = 0
    for match in re.finditer(r"[()]|\bSTDMETHODCALLTYPE\b", body):
        token = match.group(0)
        if token == "(":
            depth += 1
        elif token == ")":
            depth -= 1
        elif depth == 1:
            slots += 1
    return slots


def _parse_param(text: str, index: int) -> tuple[str, str]:
    text = " ".join(text.replace("CONST", "const").split())
    text = _CLASSIC_SAL.sub("", text).strip()
    if text in ("", "void"):
        return "", ""
    array = re.match(
        r"(?P<type>.*?)(?P<name>[A-Za-z_]\w*)\s*\[[^]]*\]$",
        text,
    )
    if array:
        # Function-parameter arrays are adjusted to pointers by C/C++ before
        # ABI lowering. The bound, when present, is not part of the vtable type.
        return f"{array.group('type').strip()} *", array.group("name")
    match = re.match(r"(?P<type>.*?)(?P<name>[A-Za-z_]\w*)$", text)
    if not match:
        return text, f"arg{index}"
    c_type = match.group("type").strip()
    name = match.group("name")
    if not c_type:
        # Legacy DECLARE_INTERFACE_ surfaces commonly omit parameter names
        # (`THIS_ DWORD`, `THIS_ REFIID,PVOID*`). A lone token is the type,
        # not a parameter named DWORD with an empty type.
        return text, f"arg{index}"
    while name.startswith("*"):
        c_type += " *"
        name = name[1:]
    return c_type, name


def parse_widl_interface(header: Path, interface: str) -> InterfaceSpec:
    """Read an exact flattened interface vtable from a compiler-facing SDK header.

    See :func:`_vtbl_declared_slots` for why the slot cross-check counts by paren depth.

    WIDL's generated ``InterfaceVtbl`` struct is preferred. A small number of
    legacy shell interfaces use ``DECLARE_INTERFACE_`` instead; that C macro
    surface explicitly lists every inherited slot, so it is equally suitable
    deterministic input once SAL/``THIS_`` spelling is normalized.
    """
    text = header.read_text(encoding="utf-8", errors="replace")
    wanted = f"{interface}Vtbl"
    block = next((m for m in _VTBL_BLOCK.finditer(text) if m.group("name") == wanted), None)
    methods = []
    if block is not None:
        for match in _VTBL_METHOD.finditer(block.group("body")):
            params = []
            for i, raw in enumerate(_split_args(match.group("params"))):
                c_type, name = _parse_param(raw, i)
                if c_type:
                    params.append((c_type, name))
            methods.append(
                InterfaceMethod(
                    name=match.group("name"),
                    return_type=" ".join(match.group("ret").split()),
                    params=tuple(params),
                )
            )
        declared = _vtbl_declared_slots(block.group("body"))
        if declared != len(methods):
            raise CppComError(
                f"{interface}: {header.name}'s {wanted} declares {declared} slot(s) but "
                f"{len(methods)} parsed. A vtable is positional -- a missed declaration "
                f"shifts every slot after it."
            )
    else:
        legacy_text = re.sub(r"/\*.*?\*/|//[^\n]*", " ", text, flags=re.S)
        legacy = re.search(
            rf"DECLARE_INTERFACE_\s*\(\s*{re.escape(interface)}\s*,[^)]*\)\s*"
            r"\{(?P<body>.*?)\}\s*;",
            legacy_text,
            flags=re.S,
        )
        if legacy is None:
            raise CppComError(
                f"{interface}: neither {wanted} nor DECLARE_INTERFACE_ definition found in "
                f"SDK header {header}"
            )
        body = legacy.group("body")
        for match in _DECLARE_METHOD.finditer(body):
            raw_params = match.group("params").strip()
            has_this = raw_params == "THIS" or raw_params.startswith("THIS_")
            raw_params = re.sub(r"^THIS_?\s*", "", raw_params)
            raw_params = _SAL_ANNOTATION.sub("", raw_params)
            params = [(f"{interface} *", "This")] if has_this else []
            for i, raw in enumerate(_split_args(raw_params), len(params)):
                c_type, name = _parse_param(raw, i)
                if c_type:
                    params.append((c_type, name))
            methods.append(
                InterfaceMethod(
                    name=match.group("name1") or match.group("name2"),
                    return_type=" ".join((match.group("ret") or "HRESULT").split()),
                    params=tuple(params),
                )
            )
        _assert_no_dropped_slots(interface, header, body, len(methods))
    if not methods:
        raise CppComError(f"{interface}: no vtable methods parsed from {header}")
    # The C++ half of WIDL's generated header is also compiler-facing authority
    # for interface inheritance.  Preserve it so a COM map that exposes both a
    # derived interface and one of its bases uses the single physical base
    # subobject that C++ actually lays out (for example IContextMenu2 and
    # IContextMenu).
    inheritance = re.search(
        rf"\b{re.escape(interface)}\s*:\s*public\s+([A-Za-z_]\w*)",
        text,
    )
    base = inheritance.group(1) if inheritance is not None else None
    # Keep generated output byte-identical across worktrees/build roots.
    return InterfaceSpec(interface, tuple(methods), header.name, base)


# Windows SDK integer typedefs admitted as proven scalar state, on top of _RUST_SCALARS.
#
# ONE copy on purpose. This lived inline in both arms of _parse_plain_state_field — the fixed
# array arm and the plain arm — and the two had already drifted: `INT8` was in the plain set and
# missing from the array set, so `INT8 Direction` was proven state while `INT8 Direction[2]` was
# not. Nothing reports a divergence between two hand-maintained copies of the same table, so the
# fix is to stop having two.
#
# Two kinds of name are in here and they are trusted for different reasons. A name that STATES
# its width (INT8/UINT16/...) is self-proving — there is nothing left to establish, which is why
# completing that family is not the same guess as admitting an unmeasured typedef. The rest
# (LONG, INT_PTR, SIZE_T, ...) are data-model-dependent and are here because this codebase is
# Windows; a name that is neither still fails loudly and wants an island proof, not an entry.
_WINDOWS_SCALAR_TYPEDEFS = frozenset(
    {
        "BOOL",
        "BOOLEAN",
        "bool",  # C++ bool: 1-byte on all targets (Itanium/MSVC), == Rust bool
        "BYTE",
        "CHAR",
        "DWORD",
        "DWORD64",
        # `DWORD_PTR` == `ULONG_PTR` (basetsd.h). Its absence next to the four other
        # `*_PTR` aliases already here was an omission, not a policy: it is exactly as
        # pointer-width-dependent as `ULONG_PTR`, and no more inferred.
        "DWORD_PTR",
        "HRESULT",
        "INT",
        "INT8",
        "INT16",
        "INT32",
        "INT64",
        "INT_PTR",
        "LONG",
        "LONG_PTR",
        "LONGLONG",
        "SHORT",
        "SIZE_T",
        "UCHAR",
        "UINT",
        "UINT8",
        "UINT16",
        "UINT32",
        "UINT64",
        "UINT_PTR",
        "ULONG",
        "ULONG_PTR",
        "ULONGLONG",
        "USHORT",
        "WCHAR",
        "WORD",
    }
)


_CPP_COM_DEBUG_MACROS = {"ASSERT", "ERR", "FIXME", "MESSAGE", "TRACE", "WARN", "pdump"}
_CPP_COM_LITERAL_WORDS = {"FALSE", "NULL", "TRUE"}
_CPP_COM_TYPE_WORDS = {
    "BOOL",
    "BYTE",
    "DWORD",
    "HRESULT",
    "INT",
    "LONG",
    "LSTATUS",
    "SIZE_T",
    "UINT",
    "ULONG",
    "ULONG_PTR",
    "WCHAR",
    "WORD",
}
_INTEGER_LITERAL = re.compile(
    r"(?<![\w.])(?P<number>0[xX][0-9A-Fa-f]+|0[oO][0-7]+|\d+)"
    r"(?:[uUlL]+|[iu](?:8|16|32|64|128|size))?(?![\w.])"
)


def _wide_literal_units(token: str) -> tuple[int, ...]:
    try:
        value = ast.literal_eval(token[1:])
    except (SyntaxError, ValueError) as exc:
        raise CppComError(f"unsupported C wide string literal {token}: {exc}") from exc
    if not isinstance(value, str):
        raise CppComError(f"unsupported C wide string literal {token}")
    encoded = value.encode("utf-16-le", "surrogatepass")
    units = tuple(encoded[i] | (encoded[i + 1] << 8) for i in range(0, len(encoded), 2))
    return (*units, 0)


def _wide_literal_name(token: str) -> str:
    units = _wide_literal_units(token)
    payload = b"".join(unit.to_bytes(2, "little") for unit in units)
    return "CPP_COM_WSTR_" + hashlib.sha256(payload).hexdigest()[:12].upper()


def _wide_char_unit(token: str) -> int:
    try:
        value = ast.literal_eval(token[1:])
    except (SyntaxError, ValueError) as exc:
        raise CppComError(f"unsupported C wide character literal {token}: {exc}") from exc
    if not isinstance(value, str) or len(value) != 1:
        raise CppComError(f"unsupported C wide character literal {token}")
    encoded = value.encode("utf-16-le", "surrogatepass")
    if len(encoded) != 2:
        raise CppComError(f"C wide character literal is not one UTF-16 code unit: {token}")
    return encoded[0] | (encoded[1] << 8)


def _wide_literal_tokens(text: str) -> list[tuple[int, int, str]]:
    """(start, end, token) for each wide literal, skipping narrow ones and comments.

    KNOWN GAP: C adjacent-literal concatenation is not folded. `REGSTR_PATH_EXPLORER
    L"\\\\IconUnderline"` (general.cpp:104) lowers the `L"..."` half to a static and
    leaves the macro beside it, which is not Rust syntax — it fails loudly at compile
    rather than silently, but it does need the macro expanded and both halves folded
    into one static before the pair can be lowered.
    """
    out = []
    for start, end in _cpp_literal_spans(text):
        token = text[start:end]
        if token.startswith(('L"', "L'")):
            out.append((start, end, token))
    return out


def _rewrite_wide_literals(text: str) -> str:
    pieces, prev = [], 0
    for start, end, token in _wide_literal_tokens(text):
        if token.startswith('L"'):
            replacement = f"{_wide_literal_name(token)}.as_ptr()"
        else:
            replacement = f"0x{_wide_char_unit(token):04X}u16"
        pieces.append(text[prev:start])
        pieces.append(replacement)
        prev = end
    pieces.append(text[prev:])
    return "".join(pieces)


def rewrite_wide_literals(text: str) -> str:
    """Public entry point for the deterministic UTF-16 literal lowering.

    `L"x"` / `L'x'` are C++ syntax with NO Rust equivalent — Rust has no UTF-16
    literal at all — so a model asked to translate one has nothing correct to
    emit and invents something (`u"x"`, `"x".encode_utf16()`, a `&str` cast).
    Lowering it here is deterministic, costs nothing, and is the same code path
    the COM translator has always used; only the entry point is new.
    """
    return _rewrite_wide_literals(text)


# The `ppv` header macros (sdk/include/reactos/shellutils.h). Each expands ONE
# apparent C++ argument into several ABI arguments, so neither is a symbol and
# neither is one Rust argument. They differ only by what sits between the IID and
# the `void**`: `IID_NULL_PPV_ARG` is for `GetUIObjectOf`, whose reserved
# `prgfInOut` parameter is always passed NULL.
#
# `IID_NULL_PPV_ARG` does not CONTAIN the string `IID_PPV_ARG`, so every rule
# keyed on `\bIID_PPV_ARG` was structurally incapable of matching it — it reached
# a module as an unexpanded macro call plus a phantom `static IID_NULL_PPV_ARG`.
# Naming the family once is what keeps a new member from repeating that.
_PPV_ARG_MIDDLES = {"IID_PPV_ARG": "", "IID_NULL_PPV_ARG": "core::ptr::null_mut(), "}
_PPV_ARG_MACRO = r"IID_(?:NULL_)?PPV_ARG"


def normalize_cpp_idioms(source: str, rust: str, *, residual: bool = True) -> str:
    """C++-generic model-output repairs — no class, no vtable, no `this` required.

    `IID_PPV_ARG(T, &out)` is a *header macro* that expands one apparent C++
    argument into two ABI arguments (`REFIID, void**`). It is not a callable
    symbol, so emitting it verbatim never compiles. Split out of
    :func:`normalize_worker_semantics` so the free-function path
    (`translate --free-functions`) gets it too: that path has no COM scaffold,
    but the C++ it translates uses the same macros.
    """
    rust = re.sub(
        r"let\s+([A-Za-z_]\w*)\s*=\s*IID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*;\s*"
        r"let\s+([A-Za-z_]\w*)\s*=\s*&mut\s+(\(\*this\)\.[A-Za-z_]\w*|"
        r"[A-Za-z_]\w*)\s*\)\s*;",
        r"let \1 = core::ptr::addr_of!(IID_\2); "
        r"let \3 = core::ptr::addr_of_mut!(\4).cast();",
        rust,
    )
    iid_pair = re.search(
        r"let\s*\(\s*([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*\)\s*=\s*"
        r"IID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*,\s*&mut\s+([A-Za-z_]\w*)\s*\)\s*;",
        rust,
    )
    if iid_pair is not None:
        iid_name, output_name, interface, output = iid_pair.groups()
        rust = rust[: iid_pair.start()] + rust[iid_pair.end() :]
        rust = re.sub(
            rf"\b{re.escape(iid_name)}\s*,\s*{re.escape(output_name)}\b",
            f"core::ptr::addr_of!(IID_{interface}), core::ptr::addr_of_mut!({output}).cast()",
            rust,
            count=1,
        )
    rust = re.sub(
        r"\bIID_PPV_ARG\s*\(\s*(?:&IID_)?([A-Za-z_]\w*)\s*,\s*"
        r"&mut\s+([A-Za-z_]\w*)(?:\s+as\s+[^,)]+(?:<[^>]+>)?)?\s*,?\s*\)",
        r"core::ptr::addr_of!(IID_\1), core::ptr::addr_of_mut!(\2).cast()",
        rust,
    )
    source_iid_args = re.findall(
        rf"\b({_PPV_ARG_MACRO})\s*\(\s*([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*\)", source
    )
    for macro, interface, output in source_iid_args:
        rust = re.sub(
            rf"\b{macro}\s*\(\s*{re.escape(output)}\s*\)",
            f"core::ptr::addr_of!(IID_{interface}), {_PPV_ARG_MIDDLES[macro]}{output}.cast()",
            rust,
        )
    # A model that splits the macro into the right NUMBER of arguments can still lose
    # the interface, putting the MACRO's own name where the IID belongs: measured on
    # CDefaultContextMenu::DoCreateLink as `&IID_PPV_ARG as *const _ as REFIID` for
    # `IID_PPV_ARG(IDropTarget, &pDT)`. The ppv slot was right; only the riid was a
    # name that exists in no library. The source names the interface, so recover it —
    # but only when the body names exactly one, since two would be a guess.
    bare_macro = re.compile(rf"\b({_PPV_ARG_MACRO})\b(?!\s*\()")
    if bare_macro.search(rust):
        interfaces = {interface for _, interface, _ in source_iid_args} or set(
            re.findall(rf"\b{_PPV_ARG_MACRO}\s*\(\s*([A-Za-z_]\w*)", source)
        )
        if len(interfaces) == 1:
            rust = bare_macro.sub(f"IID_{interfaces.pop()}", rust)
    # The COM path has LATER rules that still need to see an intact IID_PPV_ARG —
    # notably alias resolution (`let ppv = &mut (*this).inner;` … `(…, ppv)`), which
    # recovers the real destination. So the catch-all must run after those, and
    # normalize_worker_semantics applies it at its own end.
    return _expand_residual_iid_ppv_arg(rust) if residual else rust


def _expand_residual_iid_ppv_arg(rust: str) -> str:
    """Catch-all for any `IID_[NULL_]PPV_ARG(T, expr)` the shape-specific rules missed.

    Those rules each pin one spelling of the second argument (`&mut x`, a bare
    identifier, a `let` pair). A model that writes anything else — measured:
    `IID_PPV_ARG(IShellFolder, spFolder as *mut *mut IShellFolder)` — slipped
    through all of them and reached the module as a call to a macro, which is not
    a symbol and cannot link. Splitting on the top-level comma handles every
    spelling instead of enumerating them, so the failure mode is bounded.

    Being spelling-agnostic on the second argument is also why this is the one
    place the NULL variant has to be understood: it makes the whole family work
    without duplicating each shape-specific rule above.
    """
    out = []
    i = 0
    while True:
        match = re.search(rf"\b({_PPV_ARG_MACRO})\s*\(", rust[i:])
        if match is None:
            out.append(rust[i:])
            return "".join(out)
        middle = _PPV_ARG_MIDDLES[match.group(1)]
        start = i + match.start()
        cursor = i + match.end()  # just past the '('
        depth, comma = 1, None
        while cursor < len(rust) and depth:
            char = rust[cursor]
            if char in "([":
                depth += 1
            elif char in ")]":
                depth -= 1
            elif char == "," and depth == 1 and comma is None:
                comma = cursor
            cursor += 1
        if depth or comma is None:  # unbalanced or one-argument — leave it alone
            out.append(rust[i:cursor])
            i = cursor
            continue
        interface = rust[i + match.end() : comma].strip()
        # A trailing comma inside the macro is legal C and models do emit it. Left in,
        # the paren-wrap below turns `(expr,)` into a one-TUPLE, which compiles as far as
        # `.cast()` and then fails with "no method named `cast` found for tuple" —
        # measured on shell32 folders.cpp.
        value = rust[comma + 1 : cursor - 1].strip().rstrip(",").strip()
        if not re.fullmatch(r"[A-Za-z_]\w*", interface):
            out.append(rust[i:cursor])
            i = cursor
            continue
        interface = interface.removeprefix("&IID_")
        if value.startswith("&mut "):
            # Already delimited by the macro's own parens — wrapping again would
            # differ cosmetically from the shape-specific rules above for no gain.
            value = f"core::ptr::addr_of_mut!({value[5:].strip()})"
        elif value.startswith("&"):
            value = f"core::ptr::addr_of!({value[1:].strip()})"
        else:
            value = f"({value})"
        out.append(rust[i:start])
        out.append(f"core::ptr::addr_of!(IID_{interface}), {middle}{value}.cast()")
        i = cursor


def render_wide_literal_statics(raw_deps: list[FunctionDep], plan: list[FunctionDep]) -> str:
    """Render only the UTF-16 literals referenced by the selected helper closure."""
    selected = set(
        re.findall(r"\bCPP_COM_WSTR_[0-9A-F]{12}\b", "\n".join(d.body_text for d in plan))
    )
    by_name: dict[str, tuple[int, ...]] = {}
    for dep in raw_deps:
        for _start, _end, token in _wide_literal_tokens(dep.body_text):
            if not token.startswith('L"'):
                continue
            name = _wide_literal_name(token)
            if name in selected:
                by_name[name] = _wide_literal_units(token)
    missing = selected - set(by_name)
    if missing:
        raise CppComError("lost deterministic wide literal source: " + ", ".join(sorted(missing)))
    lines = ["// Deterministic UTF-16 literals from the selected C++ helper closure."]
    for name, units in sorted(by_name.items()):
        values = ", ".join(f"0x{unit:04X}" for unit in units)
        lines.append(f"pub static {name}: [WCHAR; {len(units)}] = [{values}];")
    return "\n".join(lines) if by_name else ""


def _integer_values(text: str) -> Counter[int]:
    values: Counter[int] = Counter()
    for match in _INTEGER_LITERAL.finditer(text):
        token = match.group("number")
        if token.lower().startswith("0x"):
            base = 16
        elif token.lower().startswith("0o") or (
            len(token) > 1 and token.startswith("0") and not re.search(r"[89]", token)
        ):
            base = 8
        else:
            base = 10
        values[int(token[2:] if token.lower().startswith("0o") else token, base)] += 1
    return values


# `IsEqualIID(a, b)` in a body is rewritten to `cpp_com_iid_equal(a, b)` on EVERY C++ path,
# so every path that can carry a body owes the definition. It used to live only in the ATL
# branch, past the plain-helper early return, which left a hand-rolled COM class calling a
# function nothing defined.
_CPP_COM_IID_EQUAL = "\n".join(
    [
        "#[inline]",
        "unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {",
        "    !left.is_null() && !right.is_null()",
        "        && (*left).Data1 == (*right).Data1",
        "        && (*left).Data2 == (*right).Data2",
        "        && (*left).Data3 == (*right).Data3",
        "        && (*left).Data4 == (*right).Data4",
        "}",
    ]
)


def _adopt_assembled_shaped_worker(
    rust: str, expected_name: str, expected_params: list[str]
) -> str:
    """Rename an ASSEMBLED-shaped emission back onto the worker contract.

    The model is contracted to write `X_body` with no ABI, but on COM boilerplate
    (`AddRef`, `QueryInterface`, `ctor`) it routinely writes the deterministic
    wrapper's own shape instead: `#[no_mangle] pub unsafe extern "C" fn X(...)`.
    That is RECALL, not reasoning -- the famous body arrives from memory in the
    shape it is usually written in -- so it is stable across samples and re-paying
    for the same function just buys the same wrapper again.

    Adopt it only when the emission is PROVABLY the same function under a
    different name: no `X_body` anywhere, exactly one definition of the base name,
    a parameter list already naming the generated slots in order, and a body that
    does not call `X_body`. That last guard is the one that matters -- a model
    that emitted the wrapper's *forwarder* body would become `X_body` calling
    `X_body`, i.e. silent infinite self-recursion. Everything a rename cannot
    prove stays rejected; `audit_worker_body` still grades the body afterwards,
    so this repairs shape only, never semantics.
    """
    suffix = "_body"
    if not expected_name.endswith(suffix):
        return rust
    base = expected_name[: -len(suffix)]
    if re.search(rf"\bfn\s+{re.escape(expected_name)}\s*\(", rust):
        return rust
    definitions = list(re.finditer(rf"\bfn\s+{re.escape(base)}\s*\(", rust))
    if len(definitions) != 1:
        return rust
    match = definitions[0]
    try:
        opening = rust.find("(", match.start())
        closing = _matching_paren(rust, opening)
        brace = rust.find("{", closing + 1)
        if brace < 0 or not re.fullmatch(r"\s*(?:->\s*[^{;]+)?\s*", rust[closing + 1 : brace]):
            return rust
        actual_params = [
            item_match.group(1)
            for item in _split_args(
                re.sub(r"/\*.*?\*/|//[^\n]*", " ", rust[opening + 1 : closing], flags=re.S)
            )
            if (item_match := re.match(r"\s*(?:mut\s+)?([A-Za-z_]\w*)\s*:", item))
        ]
        if actual_params != expected_params:
            return rust
        _, body_end = _balanced_block(_masked_cpp(rust), brace)
    except CppComError:
        # A `.partial` is a truncated stream by construction, so unbalanced input
        # is expected here rather than exceptional. Leave it unrepaired and let
        # the audit reject it; raising would kill the whole run over one body.
        return rust
    if re.search(rf"\b{re.escape(expected_name)}\s*\(", rust[brace : body_end + 1]):
        return rust
    prefix = re.search(
        r"(?:#\[\s*(?:unsafe\s*\(\s*)?no_mangle\s*\)?\s*\]\s*)?(?:pub\s+)?(?:unsafe\s+)?"
        r'(?:extern\s+"(?:C|system)"\s+)?$',
        rust[: match.start()],
    )
    assert prefix is not None  # every group is optional, so the empty match always succeeds
    return rust[: prefix.start()] + f"pub unsafe fn {expected_name}" + rust[opening:]


def _conform_worker_param_names(rust: str, expected_name: str, expected_params: list[str]) -> str:
    """Rename a worker's parameters to the deterministic contract's, in place.

    The splice below already replaces every type/return spelling with the contract's -- but
    only once the parameter NAMES line up, and a model that snake_cases them never gets
    there. `IntDeleteRecursive` came back as `(full_name: *const u16) -> i32` against a
    contract of `(FullName: LPCWSTR) -> BOOL`: the same ABI, renamed and de-aliased. The
    equality test could not see past the names, so a perfectly good body was rejected -- and
    a rejected body is never banked, so that one was re-bought on EVERY run, forever.

    ARITY is the line, and it stays hard. A missing or extra parameter is a real defect about
    what the function receives, not a spelling difference.
    """
    match = re.search(rf"\b(?:pub\s+)?(?:unsafe\s+)?fn\s+{re.escape(expected_name)}\s*\(", rust)
    if match is None:
        return rust
    opening = rust.find("(", match.start())
    closing = _matching_paren(rust, opening)
    brace = rust.find("{", closing + 1)
    if brace < 0:
        return rust
    items = _split_args(re.sub(r"/\*.*?\*/|//[^\n]*", " ", rust[opening + 1 : closing], flags=re.S))
    names = [
        found.group(1)
        for item in items
        if (found := re.match(r"\s*(?:mut\s+)?([A-Za-z_]\w*)\s*:", item))
    ]
    if len(names) != len(items) or len(names) != len(expected_params) or names == expected_params:
        return rust
    renames = [(was, now) for was, now in zip(names, expected_params, strict=True) if was != now]
    tail = rust[opening:]
    # If the contract's name already occurs, something else in this body answers to it and a
    # blind rename would capture that instead. Leave it for the audit to reject.
    if any(re.search(rf"\b{re.escape(now)}\b", tail) for _, now in renames):
        return rust
    for was, now in renames:
        tail = re.sub(rf"\b{re.escape(was)}\b", now, tail)
    return rust[:opening] + tail


def normalize_worker_body(rust: str, expected_signature: str | None = None) -> str:
    """Normalize only syntax the deterministic shell32 substrate already owns."""
    names = "|".join(sorted(_CPP_COM_DEBUG_MACROS))
    rust = re.sub(r"^\s*```(?:rust)?\s*|\s*```\s*$", "", rust, flags=re.I)
    # Cheap models sometimes turn a C macro into a variadic extern declaration.
    # The crate supplies macro_rules definitions, so the declaration is both wrong
    # and unnecessary. Accept only a block containing that one declaration.
    rust = re.sub(
        rf'\s*extern\s+"(?:C|system)"\s*\{{\s*fn\s+(?:{names})\s*\([^;]*;\s*\}}\s*',
        "\n    ",
        rust,
        flags=re.S,
    )
    rust = re.sub(rf"\b({names})\s*\(", r"\1!(", rust)
    # Body workers never own FFI declarations. Keep their calls, but remove
    # model-invented local declaration blocks so the deterministic module-level
    # substrate remains the single ABI authority. The audit still rejects any
    # declaration shape this narrow matcher cannot prove is declarations-only.
    rust = re.sub(
        r'\s*extern\s+"(?:C|system)"\s*\{(?:\s*(?:pub\s+)?(?:fn\s+[^;]+|'
        r"static\s+(?:mut\s+)?[A-Za-z_]\w*\s*:\s*[^;]+);)+\s*\}\s*",
        "\n    ",
        rust,
        flags=re.S,
    )
    rust = re.sub(r"\bcrate::(?:types|shellcom)::", "", rust)
    rust = re.sub(r"\bcrate::driver_ddk::([A-Z][A-Z0-9_]*)\b", r"\1", rust)
    # HRESULT is a C/Rust scalar alias, not a tuple struct constructor.
    rust = re.sub(r"\bHRESULT\s*\(\s*([A-Z][A-Z0-9_]*)\s*\)", r"\1", rust)
    # A model may name a constant array bound with `let`, which parses but does
    # not compile in a type-level array length. Promote only bindings whose RHS
    # is provably integer/operator/cast syntax and which are actually consumed
    # as an array bound. Runtime-derived bindings remain untouched.
    constant_binding = re.compile(r"\blet\s+([A-Za-z_]\w*)\s*:\s*usize\s*=\s*([^;]+);")
    for match in reversed(list(constant_binding.finditer(rust))):
        name, expression = match.groups()
        tail = rust[match.end() :]
        lowercase_words = re.findall(r"\b[a-z_][A-Za-z0-9_]*\b", expression)
        constant_only = all(word in {"as", "usize"} for word in lowercase_words)
        if constant_only and re.search(rf"\[[^;\]]*;\s*{re.escape(name)}\s*\]", tail):
            replacement = f"const {name}: usize = {expression};"
            rust = rust[: match.start()] + replacement + rust[match.end() :]
    if expected_signature is not None:
        # Rust warns about unused parameters, so cheap models often rename an
        # ABI-owned `param` to `_param`. The deterministic wrapper fixes the
        # worker signature; restore those exact names in signature and body.
        expected_opening = expected_signature.find("(")
        expected_closing = _matching_paren(expected_signature, expected_opening)
        expected_params = [
            match.group(1)
            for item in _split_args(expected_signature[expected_opening + 1 : expected_closing])
            if (match := re.match(r"\s*(?:mut\s+)?([A-Za-z_]\w*)\s*:", item))
        ]
        for name in expected_params:
            rust = re.sub(rf"\b_{re.escape(name)}\b", name, rust)
        expected_name_match = re.search(r"\bfn\s+([A-Za-z_]\w*)\s*\(", expected_signature)
        if expected_name_match is None:
            raise CppComError(f"invalid deterministic worker signature: {expected_signature}")
        expected_name = expected_name_match.group(1)
        rust = _adopt_assembled_shaped_worker(rust, expected_name, expected_params)
        rust = _conform_worker_param_names(rust, expected_name, expected_params)
        actual = re.search(
            rf"\b(?:pub\s+)?(?:unsafe\s+)?fn\s+{re.escape(expected_name)}\s*\(",
            rust,
        )
        if actual is not None:
            opening = rust.find("(", actual.start())
            closing = _matching_paren(rust, opening)
            brace = rust.find("{", closing + 1)
            actual_param_text = re.sub(
                r"/\*.*?\*/|//[^\n]*", " ", rust[opening + 1 : closing], flags=re.S
            )
            actual_items = _split_args(actual_param_text)
            actual_params = [
                match.group(1)
                for item in actual_items
                if (match := re.match(r"\s*(?:mut\s+)?([A-Za-z_]\w*)\s*:", item))
            ]
            mutable_params = {
                match.group(1)
                for item in actual_items
                if (match := re.match(r"\s*mut\s+([A-Za-z_]\w*)\s*:", item))
            }
            # The model owns no part of the worker surface. If its parameter
            # list still names the generated slots in order, replace all of
            # its type/return/qualifier spellings with the exact authority.
            if brace >= 0 and actual_params == expected_params:
                authoritative = expected_signature
                for name in mutable_params:
                    authoritative = re.sub(
                        rf"(?<=[(,])\s*{re.escape(name)}\s*:",
                        lambda match, param=name: match.group(0).replace(param, f"mut {param}"),
                        authoritative,
                        count=1,
                    )
                rust = rust[: actual.start()] + authoritative + " " + rust[brace:]
        # The model is contracted for exactly one body worker. Deterministically
        # discard crate attributes, imports, shadow types, and trailing helpers
        # outside that function before any response can enter the cache/module.
        exact = re.search(rf"\bpub\s+unsafe\s+fn\s+{re.escape(expected_name)}\s*\(", rust)
        if exact is not None:
            opening = rust.find("(", exact.start())
            closing = _matching_paren(rust, opening)
            brace = rust.find("{", closing + 1)
            if brace >= 0:
                _, body_end = _balanced_block(_masked_cpp(rust), brace)
                rust = rust[exact.start() : body_end + 1]
    return rust


# Optional receiver chain (`(*p).`, `fi.`, `(*this).m_pApp.`) in front of a bare `hIcon`.
# Group 1 is the chain including its trailing dot, or None when the name stands alone.
_HICON_TRUTHY = re.compile(r"((?:\(\*\w+\)|\w+)(?:\.\w+)*\.)?\bhIcon\s*!=\s*0\b")


def normalize_worker_semantics(source: str, rust: str) -> str:
    """Lower compiler-known C/ATL conveniences after a cached/model worker returns.

    These rewrites depend only on syntax present in the original worker.  They are
    deliberately downstream of the response cache: improving a deterministic
    lowering must never require buying the same model response again.
    """
    rust = strip_debug_statements(rust, {"pdump"})
    rust = re.sub(r"\bcrate::driver_ddk::([A-Z][A-Z0-9_]*)\b", r"\1", rust)
    rust = re.sub(
        r"&mut\s+(\(\*[A-Za-z_]\w*\)\.u\.[A-Za-z_]\w*)\s+as\s+\*mut\s+"
        r"([A-Za-z_]\w*)",
        r"core::ptr::addr_of_mut!(\1).cast::<\2>()",
        rust,
    )
    self_guards = set()
    for owner, name in re.findall(
        r"\bCComPtr\s*<\s*([A-Za-z_]\w*)\s*>\s+([A-Za-z_]\w*)\s*=\s*this\s*;",
        source,
    ):
        self_guards.add(name)
        if re.search(
            rf"let\s+mut\s+{re.escape(name)}\s*:\s*\*mut\s+IUnknown\s*=\s*this\.cast\(\)",
            rust,
        ):
            continue
        model_guard = re.compile(
            r"(?s)\s*let\s+[A-Za-z_]\w*\s*=\s*this\s+as\s+\*mut\s+IUnknown\s*;"
            r".*?let\s+_[A-Za-z_]\w*\s*=\s*[A-Za-z_]\w*\s*\([^;]+\)\s*;"
        )
        replacement = (
            f"\n    let mut {name}: *mut IUnknown = this.cast();\n"
            f"    ((*(*{name}).lpVtbl).AddRef)({name});\n"
            f"    let _{name}_cpp_com_guard = CppComPtrGuard::new("
            f"core::ptr::addr_of_mut!({name}));"
        )
        rust, replaced = model_guard.subn(replacement, rust, count=1)
        if replaced != 1:
            raise CppComError(f"CComPtr self guard {owner} {name}: model guard not recognized")
    rust = re.sub(
        r"\*([A-Za-z_]\w*(?:pidl|Pidl)[A-Za-z_]*)\s*=\s*0\s*;",
        r"*\1 = core::ptr::null_mut();",
        rust,
    )
    rust = re.sub(r"\bIsEqualIID\s*\(", "cpp_com_iid_equal(", rust)
    if "SHCreateWorkerWindowW" in source and "WorkerWndProc" in source:
        rust = re.sub(
            r"([A-Za-z_]\w*WorkerWndProc)\s+as\s+unsafe\s+extern\s+"
            r'"system"\s+fn\s*\(\.\.\.\)\s*->\s*LRESULT',
            r"Some(\1)",
            rust,
        )
        rust = rust.replace("Some(WorkerWndProc)", "Some(CUserNotification_WorkerWndProc)")
        rust = rust.replace("return 0 as isize;", "return 0 as LRESULT;")
        rust = rust.replace(
            "(*this).m_hWorkerWnd = 0 as isize;",
            "(*this).m_hWorkerWnd = core::ptr::null_mut();",
        )
        rust = re.sub(
            r"(wParam\s*==\s*ID_[A-Za-z0-9_]+)\s+as\s+usize",
            r"\1 as WPARAM",
            rust,
        )
        rust = rust.replace(
            "KillTimer(hWnd, wParam as isize)",
            "KillTimer(hWnd, wParam as UINT_PTR)",
        )
    rust = re.sub(
        r"\(\*this\)\.(m_[A-Za-z0-9_]+)\s+as\s+\*const\s+u16",
        r"(*this).\1.as_ptr()",
        rust,
    )
    rust = re.sub(
        r"\(\*this\)\.(m_h[A-Za-z0-9_]+)\s*!=\s*0\b",
        r"!(*this).\1.is_null()",
        rust,
    )
    # `hIcon != 0` is a pointer truthiness test, but the `!` belongs in front of the WHOLE
    # place expression, not in front of the field. Rewriting only the bare name turned the
    # model's perfectly good `(*p).hIcon != 0` into `(*p).!hIcon.is_null()` — a parse error,
    # so it failed loud, but it is the pass corrupting correct input rather than the model
    # slipping. Capture the receiver chain and re-emit it INSIDE the negation. `\b` still
    # guards the `m_hIcon` case, whose own rule runs just above (no boundary between `_`
    # and `h`, so a bare-name pattern cannot match inside it).
    rust = _HICON_TRUTHY.sub(lambda m: f"!{m.group(1) or ''}hIcon.is_null()", rust)
    if "CUserNotification" in source:
        rust = rust.replace("Some(WorkerWndProc)", "Some(CUserNotification_WorkerWndProc)")
    rust = re.sub(
        r"(SetTimer\([^;]*?),\s*(?:::)?core::ptr::null_mut\(\)\s*\)",
        r"\1, None)",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"&nid\s+as\s+\*const\s+NOTIFYICONDATAW",
        r"&mut nid",
        rust,
    )
    rust = re.sub(
        r"\bSHELL32_CompareDetails\s*\(\s*this\s*,",
        "SHELL32_CompareDetails(core::ptr::addr_of_mut!((*this).IShellFolder2_iface),",
        rust,
    )
    rust = re.sub(
        r"(pshf:\s*)this\b",
        r"\1core::ptr::addr_of_mut!((*this).IShellFolder_iface)",
        rust,
    )
    rust = re.sub(
        r"(\*pcsFlags\s*=\s*[A-Za-z_]\w*\[[^]]+\]\.colstate)(\s*;)",
        r"\1 as SHCOLSTATEF\2",
        rust,
    )
    rust = re.sub(
        r"((?:\(\*[A-Za-z_]\w*\)|[A-Za-z_]\w*)\.(?:fmt|cxChar)\s*=\s*"
        r"[A-Za-z_]\w*\[[^]]+\]\.(?:fmt|cxChar))(\s*;)",
        r"\1 as INT\2",
        rust,
    )
    rust = re.sub(
        r"cpp_com_iid_equal\(([^()]*(?:\([^()]*\)[^()]*)*)\)\s*!=\s*0",
        r"cpp_com_iid_equal(\1)",
        rust,
    )
    # A worker may use an SDK type named by the C source, but it may not shadow
    # that deterministic module-level authority with a guessed local layout.
    # Remove only repr(C) declarations whose names occur in the source and are
    # not themselves declared by that source worker. Source-local PODs remain
    # intact and are layout-checked by ``audit_worker_body``.
    source_local_types = _source_local_pod_fields(source)
    shadow_heads = list(
        re.finditer(
            r"#\s*\[\s*repr\s*\(\s*C\s*\)\s*\]\s*struct\s+([A-Za-z_]\w*)\s*\{",
            rust,
        )
    )
    for shadow in reversed(shadow_heads):
        type_name = shadow.group(1)
        if type_name in source_local_types or not re.search(rf"\b{re.escape(type_name)}\b", source):
            continue
        open_brace = rust.find("{", shadow.start())
        try:
            _, close_brace = _balanced_block(rust, open_brace)
        except CppComError:
            continue
        rust = rust[: shadow.start()] + rust[close_brace + 1 :]

    # Rust reserves ``final`` for future use, while C/C++ permits it as an
    # ordinary local name. Raw identifiers preserve that source spelling.
    if re.search(r"\bfinal\s*(?:=|;|\[)", _masked_cpp(source)):
        masked_rust = _masked_cpp(rust)
        matches = list(re.finditer(r"(?<![#\w])final\b", masked_rust))
        for match in reversed(matches):
            rust = rust[: match.start()] + "r#final" + rust[match.end() :]

    # no_std shell modules never own a `std` dependency.
    rust = rust.replace("std::", "core::")
    sfv_create = re.search(
        r"\bSFV_CREATE\s+([A-Za-z_]\w*)\s*=\s*\{\s*sizeof\s*\(\s*SFV_CREATE\s*\)\s*,\s*"
        r"([A-Za-z_]\w*)\s*\}\s*;",
        source,
    )
    if sfv_create is not None:
        local_name, folder_name = sfv_create.groups()
        # The SDK declaration is the ABI authority. A model-local shadow type is
        # never accepted, even if its guessed prefix happens to compile.
        rust = re.sub(
            r"(?s)(?:#\[repr\(C\)\]\s*)?struct\s+SFV_CREATE\s*\{[^{}]*\}\s*",
            "",
            rust,
        )
        initializer = (
            f"let {local_name} = SFV_CREATE {{\n"
            "        cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,\n"
            f"        pshf: {folder_name},\n"
            "        psvOuter: core::ptr::null_mut(),\n"
            "        psfvcb: core::ptr::null_mut(),\n"
            "    };"
        )
        rust, replacements = re.subn(
            rf"(?s)let\s+(?:mut\s+)?{re.escape(local_name)}(?:\s*:[^=]+)?\s*=\s*"
            r"SFV_CREATE\s*\{.*?\}\s*;",
            initializer,
            rust,
            count=1,
        )
        if replacements != 1:
            rust, replacements = re.subn(
                rf"(?s)let\s+(?:mut\s+)?{re.escape(local_name)}\s*:\s*SFV_CREATE\s*=\s*"
                rf"core::mem::zeroed\(\)\s*;\s*{re.escape(local_name)}\.cbSize\s*=.*?;\s*"
                rf"{re.escape(local_name)}\.pshf\s*=.*?;",
                initializer,
                rust,
                count=1,
            )
        if replacements != 1:
            raise CppComError(f"SFV_CREATE local {local_name}: worker initializer not found")
    # CComPtr locals have scope-exit semantics on every C++ return path.  Keep
    # the model's raw pointer spelling (which makes vtable calls simple), but
    # attach a deterministic guard to its slot.  This closes failure exits the
    # model may overlook and makes ownership independent of control-flow style.
    com_ptr_locals = re.findall(
        r"\bCComPtr\s*<\s*([A-Za-z_]\w*)\s*>\s+([A-Za-z_]\w*)\s*[;=]",
        source,
    )
    com_drop = re.search(r"\bstruct\s+ComDrop\s*<[^>]+>\s*\{", rust)
    if com_drop is not None:
        first_open = rust.find("{", com_drop.start())
        _inner, first_end = _balanced_block(rust, first_open)
        drop_impl = re.search(
            r"impl\s*<[^>]+>\s+Drop\s+for\s+ComDrop\s*<[^>]+>\s*\{",
            rust[first_end + 1 :],
        )
        if drop_impl is None:
            raise CppComError("model ComDrop helper has no matching Drop implementation")
        impl_start = first_end + 1 + drop_impl.start()
        impl_open = rust.find("{", impl_start)
        _inner, impl_end = _balanced_block(rust, impl_open)
        rust = rust[: com_drop.start()] + rust[impl_end + 1 :]
    for interface, name in com_ptr_locals:
        if name in self_guards:
            continue
        if len(re.findall(rf"\b{re.escape(name)}\b", source)) == 1:
            # An unused ATL local has no observable construction/destruction;
            # omitting it is the exact deterministic lowering. If the model
            # emitted an immutable raw-null placeholder, remove that placeholder
            # and its defensive Release debris. A mutable slot may still be an
            # out-parameter owner and follows the ordinary guard path below.
            immutable = re.compile(
                rf"^[ \t]*let\s+{re.escape(name)}\s*:\s*\*mut\s+"
                rf"{re.escape(interface)}\s*=\s*core::ptr::null_mut\(\)\s*;\s*",
                flags=re.M,
            )
            if immutable.search(rust):
                rust = immutable.sub("", rust)
                rust = re.sub(
                    rf"\s*if\s+!{re.escape(name)}\.is_null\(\)\s*\{{\s*"
                    rf"\(\(\*\(\*{re.escape(name)}\)\.lpVtbl\)\.Release\)"
                    rf"\({re.escape(name)}\);\s*\}}",
                    "",
                    rust,
                )
                continue
            if not re.search(rf"\b{re.escape(name)}\b", rust):
                continue
        # The model sometimes reproduces ATL's wrapper syntax even though the
        # worker contract exposes CComPtr storage as a scaffold-owned raw slot.
        # Lower only the exact null/default construction shape; Attach/Detach,
        # non-null construction, or any other ownership API remains a failure.
        rust = re.sub(
            rf"let\s+mut\s+{re.escape(name)}\s*:\s*CComPtr\s*<\s*"
            rf"{re.escape(interface)}\s*>\s*=\s*CComPtr::new\(\)\s*;",
            f"let mut {name}: *mut {interface} = core::ptr::null_mut();",
            rust,
            count=1,
        )
        rust = re.sub(
            rf"let\s+mut\s+{re.escape(name)}\s*=\s*ComDrop\s*::\s*<\s*"
            rf"{re.escape(interface)}\s*>\s*\{{\s*ptr:\s*core::ptr::null_mut\(\)\s*,?\s*\}}\s*;",
            f"let mut {name}: *mut {interface} = core::ptr::null_mut();",
            rust,
            count=1,
        )
        # Same construction, spelled with a turbofish and no annotation. The annotated form
        # above was the only one lowered, so `let mut pcdb2 = CComPtr::<ICommDlgBrowser2>::new()`
        # fell through and the declaration audit reported zero raw-pointer slots -- the last
        # CDefView worker, rejected for a spelling. Still only the null/default shape.
        rust = re.sub(
            rf"let\s+mut\s+{re.escape(name)}\s*=\s*CComPtr\s*::\s*<\s*"
            rf"{re.escape(interface)}\s*>\s*::\s*new\(\)\s*;",
            f"let mut {name}: *mut {interface} = core::ptr::null_mut();",
            rust,
            count=1,
        )
        rust = re.sub(rf"\b{re.escape(name)}\.ptr\b", name, rust)
        # `.as_ptr()` is a READ of the slot, like `.ptr` and `.0` beside it -- not an ownership
        # API, so lowering it does not weaken the Attach/Detach refusal.
        rust = re.sub(rf"\b{re.escape(name)}\.as_ptr\(\)", name, rust)
        copied_value = re.search(
            rf"\bCComPtr\s*<\s*{re.escape(interface)}\s*>\s+{re.escape(name)}\s*=\s*"
            r"(?P<this>this->)?(?P<value>[A-Za-z_]\w*)\s*;",
            source,
        )
        if copied_value is not None:
            value = copied_value.group("value")
            member_value = f"(*this).{value}"
            rust_value = (
                member_value
                if copied_value.group("this")
                or re.search(
                    rf"let\s+(?:mut\s+)?{re.escape(name)}(?:\s*:[^=]+)?\s*=\s*"
                    rf"{re.escape(member_value)}\s*;",
                    rust,
                )
                else value
            )
            rust = re.sub(
                rf"let\s+(?:mut\s+)?{re.escape(name)}(?:\s*:\s*\*mut\s+"
                rf"{re.escape(interface)})?\s*=\s*{re.escape(rust_value)}\s*;",
                f"let mut {name}: *mut {interface} = {rust_value};",
                rust,
                count=1,
            )
            # ATL's copy constructor owns one AddRef.  Normalize a model-spelled
            # inherited-IUnknown cast to the concrete vtable signature, or add
            # the call deterministically when the model omitted it.
            add_ref = re.compile(
                rf"\(\(\*\(\*{re.escape(name)}\)\.lpVtbl\)\.AddRef\)"
                rf"\({re.escape(name)}(?:\s+as\s+\*mut\s+IUnknown)?\)"
            )
            add_refs = list(add_ref.finditer(rust))
            if len(add_refs) > 1:
                raise CppComError(f"CComPtr local {name}: multiple copy AddRef calls")
            rust = add_ref.sub(f"((*(*{name}).lpVtbl).AddRef)({name})", rust, count=1)
        rust = re.sub(rf"\b{re.escape(name)}\.0\b", name, rust)
        # The slot is a RAW POINTER now, so ATL's operators have no methods to resolve to.
        # Each lowers to the thing the operator meant, and the distinction matters:
        #   `operator T*()`  -> the pointer VALUE            (`p.as_ptr()`      -> `p`)
        #   `operator&()`    -> the ADDRESS OF THE SLOT      (`p.as_mut_ptr()`  -> `&raw mut p`)
        #   `Attach(v)`      -> adopt without AddRef         (`p.Attach(v)`     -> `p = v`)
        # Getting the first two the wrong way round is silent at the call site and wrong at
        # runtime: an out-parameter handed the value leaves the slot null and leaks whatever
        # the callee allocated. `Release`/`Detach` are deliberately NOT rewritten — the guard
        # owns the release, and a `Detach` needs a temporary that this cannot express.
        rust = re.sub(rf"\b{re.escape(name)}\.as_ptr\(\)", name, rust)
        rust = re.sub(
            rf"\b{re.escape(name)}\.as_mut_ptr\(\)",
            f"core::ptr::addr_of_mut!({name})",
            rust,
        )
        rust = re.sub(rf"\b{re.escape(name)}\.Attach\(", f"{name} = (", rust)
        declaration = re.compile(
            rf"(?P<indent>^[ \t]*)let\s+mut\s+{re.escape(name)}\s*:\s*"
            rf"\*mut\s+{re.escape(interface)}\s*=\s*(?:core::ptr::null_mut\(\)|"
            rf"\(\*this\)\.[A-Za-z_]\w*|[A-Za-z_]\w*)\s*;",
            re.M,
        )
        matches = list(declaration.finditer(rust))
        if len(matches) != 1:
            raise CppComError(
                f"CComPtr local {name}: expected one null raw-pointer declaration in worker, "
                f"found {len(matches)}"
            )
        match = matches[0]
        guard_name = f"_{name}_cpp_com_guard"
        if not re.search(rf"\blet\s+{re.escape(guard_name)}\s*=", rust):
            guard = (
                match.group(0)
                + "\n"
                + match.group("indent")
                + (
                    f"if !{name}.is_null() {{ ((*(*{name}).lpVtbl).AddRef)({name}); }}\n"
                    + match.group("indent")
                    if copied_value is not None and not add_refs
                    else ""
                )
                + f"let {guard_name} = CppComPtrGuard::new("
                + f"core::ptr::addr_of_mut!({name}));"
            )
            rust = rust[: match.start()] + guard + rust[match.end() :]

        # DeepSeek commonly emits explicit CComPtr-destructor releases on some
        # branches.  Null the watched slot after those releases so the guard is
        # idempotent while retaining the exact point at which C++ destroyed it.
        release = re.compile(
            rf"(?P<indent>^[ \t]*)\(\(\*\(\*{re.escape(name)}\)\.lpVtbl\)\.Release\)"
            rf"\({re.escape(name)}(?:\s+as\s+\*mut\s+IUnknown)?\);"
            rf"(?!\s*{re.escape(name)}\s*=\s*core::ptr::null_mut\(\)\s*;)",
            re.M,
        )
        rust = release.sub(
            lambda item, local_name=name: (
                item.group(0)
                + "\n"
                + item.group("indent")
                + f"{local_name} = core::ptr::null_mut();"
            ),
            rust,
        )
    # CComPtr object state is likewise represented by a raw pointer in the
    # generated owner. `.0` is a model artifact from an imagined Rust wrapper.
    rust = re.sub(r"(\(\*this\)\.[A-Za-z_]\w*)\.0\b", r"\1", rust)
    rust = normalize_cpp_idioms(source, rust, residual=False)
    # C++ sibling calls have an implicit receiver. The prepared source names
    # every such worker explicitly, so receiver insertion is syntax-authoritative.
    owner = re.search(r"\b([A-Za-z_]\w*)::[~A-Za-z_]\w*\s*\(", source)
    if owner is None:
        # Inline ATL methods lose the qualified `Class::Method` spelling in
        # the prepared source, but the generated worker signature still owns
        # the receiver type.  Recover that deterministic owner instead.
        owner = re.search(
            r"\bfn\s+[A-Za-z_]\w*_body\s*\(\s*this\s*:\s*\*mut\s+([A-Za-z_]\w*)",
            rust,
        )
    if owner is not None:
        class_name = owner.group(1)
        for nested in set(re.findall(rf"\b{re.escape(class_name)}::([A-Za-z_]\w*)\b", source)):
            rust = re.sub(
                rf"\b(?:{re.escape(class_name)}_|{re.escape(class_name)}::)"
                rf"{re.escape(nested)}\b",
                nested,
                rust,
            )
    sibling_calls = (
        set(re.findall(rf"\b({re.escape(owner.group(1))}_[A-Za-z_]\w*_body)\s*\(", source))
        if owner is not None
        else set()
    )
    sibling_arities: dict[str, set[int]] = {}
    masked_source = _masked_cpp(source)
    for _start, end, callee, arity in _named_call_spans(source, sibling_calls):
        opening = masked_source.find("(", end)
        tail = masked_source[opening + 1 :].lstrip()
        source_has_this = tail.startswith("this") and not re.match(r"this[A-Za-z0-9_]", tail)
        sibling_arities.setdefault(callee, set()).add(arity - 1 if source_has_this else arity)
    insertions = []
    masked_rust = _masked_cpp(rust)
    for _start, end, callee, arity in _named_call_spans(rust, sibling_calls):
        if arity not in sibling_arities.get(callee, set()):
            continue
        opening = masked_rust.find("(", end)
        tail = masked_rust[opening + 1 :].lstrip()
        if tail.startswith("this") and not re.match(r"this[A-Za-z0-9_]", tail):
            continue
        insertions.append((opening + 1, "this" + (", " if arity else "")))
    for position, insertion in reversed(insertions):
        rust = rust[:position] + insertion + rust[position:]
    # Header-only predicates and HRESULT helpers are deterministic expressions.
    rust = re.sub(r"\bFAILED_UNEXPECTEDLY\s*\(([^()]+)\)", r"((\1) < 0)", rust)
    rust = re.sub(r"\bSCODE_CODE\s*\(([^()]+)\)", r"(((\1) as u32) & 0xffff)", rust)
    rust = re.sub(
        r"\bIsEqualGUID\s*\(\s*([^,]+),\s*&([A-Za-z_]\w*)\s+as\s+\*const\s+GUID\s*\)\s*!=\s*0",
        r"cpp_com_iid_equal(\1, core::ptr::addr_of!(\2))",
        rust,
    )
    rust = re.sub(
        r"\bIsEqualGUID\s*\(\s*([^,]+),\s*((?:BHID|CLSID|IID)_[A-Za-z_]\w*)\s*\)"
        r"(?:\s*!=\s*0)?",
        r"cpp_com_iid_equal(\1, core::ptr::addr_of!(\2))",
        rust,
    )
    rust = re.sub(r"if\s+_ILIsDesktop\s*\((.*?)\)\s*\{", r"if _ILIsDesktop(\1) != 0 {", rust)
    rust = re.sub(
        r"if\s+!_ILIsSpecialFolder\s*\((.*?)\)\s*\{",
        r"if _ILIsSpecialFolder(\1) == 0 {",
        rust,
    )
    rust = re.sub(r"(?m)^\s*ATLASSERT\s*\([^;]*\)\s*;\s*", "", rust)
    rust = re.sub(
        r"\bSHStrDupW\s*\(\s*INVALID_FILETITLE_CHARACTERSW\s*,",
        "SHStrDupW(INVALID_FILETITLE_CHARACTERSW.as_ptr(),",
        rust,
    )
    iid_alias = re.search(
        r"let\s+([A-Za-z_]\w*)\s*=\s*&mut\s+\(\*this\)\.([A-Za-z_]\w*)\s*;",
        rust,
    )
    if iid_alias is not None:
        alias, field = iid_alias.groups()
        iid_use = re.compile(
            rf"\b({_PPV_ARG_MACRO})\s*\(\s*([A-Za-z_]\w*)\s*,\s*{re.escape(alias)}\s*\)"
        )
        matched_use = iid_use.search(rust)
        if matched_use is not None:
            macro, interface = matched_use.groups()
            replacement = (
                f"core::ptr::addr_of!(IID_{interface}), "
                f"{_PPV_ARG_MIDDLES[macro]}"
                f"core::ptr::addr_of_mut!((*this).{field}).cast()"
            )
            rust = rust[: iid_alias.start()] + rust[iid_alias.end() :]
            rust = iid_use.sub(replacement, rust, count=1)
    rust = re.sub(
        r"let\s*\(\s*([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*\)\s*=\s*"
        r"([^,;]+),\s*([^;]+);",
        r"let \1 = \3; let \2 = \4;",
        rust,
    )
    rust = re.sub(
        r"(\.CompareIDs\)\([^,]+,)\s*([A-Za-z_]\w*)\s*,",
        r"\1 \2 as LPARAM,",
        rust,
    )
    rust = re.sub(r"\bGET_SHGDN_RELATION\s*\(([^()]+)\)", r"((\1 as DWORD) & 0xff)", rust)
    rust = re.sub(r"\bGET_SHGDN_FOR\s*\(([^()]+)\)", r"((\1 as DWORD) & 0xff00)", rust)
    rust = re.sub(r"\(\*this\)\.m_pisfInner\.as_ptr\(\)", "(*this).m_pisfInner", rust)
    rust = re.sub(r"\(\*this\)\.m_pisfInner\.p\b", "(*this).m_pisfInner", rust)
    rust = re.sub(
        r"\bIID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*,\s*"
        r"&mut\s+\(\*this\)\.([A-Za-z_]\w*)\s*\)",
        r"core::ptr::addr_of!(IID_\1), core::ptr::addr_of_mut!((*this).\2).cast()",
        rust,
    )
    iid_binding = re.search(
        r"let\s+([A-Za-z_]\w*)\s*=\s*IID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*,\s*"
        r"&mut\s+\(\*this\)\.([A-Za-z_]\w*)\s*\)\s*;",
        rust,
    )
    if iid_binding is not None:
        binding, interface, field = iid_binding.groups()
        expansion = (
            f"core::ptr::addr_of!(IID_{interface}), core::ptr::addr_of_mut!((*this).{field}).cast()"
        )
        rust = rust[: iid_binding.start()] + rust[iid_binding.end() :]
        rust = re.sub(rf"\b{re.escape(binding)}\s*,", expansion + ",", rust, count=1)
    transformed_binding = re.search(
        r"let\s+([A-Za-z_]\w*)\s*=\s*"
        r"(core::ptr::addr_of!\(IID_[A-Za-z_]\w*\))\s*,\s*"
        r"(core::ptr::addr_of_mut!\(\(\*this\)\.[A-Za-z_]\w*\)\.cast\(\))\s*;",
        rust,
    )
    if transformed_binding is not None:
        binding, iid, output = transformed_binding.groups()
        rust = rust[: transformed_binding.start()] + rust[transformed_binding.end() :]
        rust = re.sub(rf"\b{re.escape(binding)}\s*,", f"{iid}, {output},", rust, count=1)
    rust = re.sub(
        r"\bSHFree\s*\(\s*(\(\*this\)\.[A-Za-z_]\w*)\s*\)",
        r"SHFree((\1) as LPVOID)",
        rust,
    )
    # Win32 error APIs use LONG in HRESULT_FROM_WIN32, while most producers spell
    # the value DWORD.  The C macro performs this scalar conversion implicitly.
    rust = re.sub(
        r"\bHRESULT_FROM_WIN32\s*\(\s*([A-Za-z_]\w*)\s*\)", r"HRESULT_FROM_WIN32(\1 as LONG)", rust
    )
    # void-pointer arguments accept any object pointer in C.
    rust = re.sub(
        r"\bSetClipboard_body\s*\(([^,]+),\s*([A-Za-z_]\w*)\s*,\s*([^\)]+)\)",
        r"SetClipboard_body(\1, \2 as *const core::ffi::c_void, \3)",
        rust,
    )
    # ERROR_SUCCESS is canonically LONG in the shared substrate; DWORD-returning
    # helpers receive the same zero bit pattern through C's usual conversion.
    rust = re.sub(r"\{\s*ERROR_SUCCESS\s*\}\s*else", "{ ERROR_SUCCESS as DWORD } else", rust)
    # CStringW(MAKEINTRESOURCEW(id)) invokes ATL's resource-string constructor.
    # Cheap models commonly mistake the integer resource token for a string pointer.
    resource = re.search(
        r"CStringW\s+([A-Za-z_]\w*)\s*\(\s*MAKEINTRESOURCEW\s*\(\s*([A-Za-z_]\w*)\s*\)\s*\)",
        source,
    )
    if resource is not None:
        source_name, resource_id = resource.groups()
        binding = re.search(
            rf"let\s+([A-Za-z_]\w*)\s*:\s*\*const\s+u16\s*=\s*{re.escape(resource_id)}[^;]*;",
            rust,
        )
        if binding is not None:
            rust_name = binding.group(1)
            rust = (
                rust[: binding.start()]
                + f"let mut {rust_name} = CStringW::from_resource({resource_id} as UINT);"
                + rust[binding.end() :]
            )
            rust = re.sub(
                rf"\b{re.escape(rust_name)}\s+as\s+\*mut\s+u16", f"{rust_name}.as_mut_ptr()", rust
            )
        else:
            rust = re.sub(
                rf"let\s+(?:mut\s+)?{re.escape(source_name)}\s*=\s*CStringW::new\([^;]*\);",
                f"let mut {source_name} = CStringW::from_resource({resource_id} as UINT);",
                rust,
                count=1,
            )
        rust = re.sub(
            rf"\bSHStrDupW\s*\(\s*{re.escape(source_name)}\s*,",
            f"SHStrDupW({source_name}.as_ptr(),",
            rust,
        )
    # Fixed C arrays retain their extent in generated Rust object state.
    rust = re.sub(r"\b_countof\s*\(([^()]+|\(\*this\)\.[A-Za-z_]\w*)\)", r"\1.len()", rust)
    rust = re.sub(r"&mut\s+([A-Za-z_]\w*)\s+as\s+\*mut\s+_\.len\(\)", r"\1.len()", rust)
    # Finish arbitrary member/parenthesized forms with a balanced transform;
    # regex-only lowering cannot see `_countof((*p).field)` correctly.
    while True:
        masked = _masked_cpp(rust)
        calls = list(re.finditer(r"\b_countof!?\s*\(", masked))
        if not calls:
            break
        call = calls[-1]
        opening = masked.find("(", call.start())
        closing = _matching_paren(masked, opening)
        expression = rust[opening + 1 : closing].strip()
        if expression.startswith("&"):
            expression = expression[1:].strip()
        replacement = expression if expression.endswith(".len()") else f"({expression}).len()"
        rust = rust[: call.start()] + replacement + rust[closing + 1 :]
    # A model sometimes heapifies a fixed C stack buffer. Restore the exact
    # source-derived extent; shell32 workers are no_std and this also preserves
    # the original allocation/failure behavior.
    for fixed in re.finditer(
        r"\b(?P<type>WCHAR|CHAR|BYTE)\s+(?P<name>[A-Za-z_]\w*)\s*"
        r"\[\s*(?P<count>[A-Za-z_]\w*|\d+)\s*\]",
        source,
    ):
        c_type, name, count = fixed.group("type", "name", "count")
        rust = re.sub(
            rf"(?:const\s+[A-Za-z_]\w*\s*:\s*usize\s*=\s*{re.escape(count)}(?:\s+as\s+usize)?\s*;\s*)?"
            rf"let\s+mut\s+{re.escape(name)}\s*:\s*Vec<[^>]+>\s*=\s*"
            rf"vec!\[\s*0(?:u\d+)?\s*;\s*[A-Za-z_]\w*\s*\]\s*;",
            f"let mut {name}: [{c_type}; {count} as usize] = [0; {count} as usize];",
            rust,
        )
    # C field spellings can be Rust keywords. Raw identifiers preserve layout
    # and keep source-derived member access mechanical.
    rust = re.sub(r"\.type\b", ".r#type", rust)
    rust = re.sub(r"(\b[A-Za-z_]\w*_body\(this,)\s+", r"\1 ", rust)
    rust = re.sub(
        r"let\s+([A-Za-z_]\w*)\s*:\s*BOOL\s*=\s*if\s+([^{};]+)\s*"
        r"\{\s*1\s*\}\s*else\s*\{\s*0\s*\}\s*;",
        r"let \1: BOOL = (\2) as BOOL;",
        rust,
    )
    rust = re.sub(r"\bCShellLink::volume_info\b", "volume_info", rust)
    rust = re.sub(
        r"\bHeapFree\s*\(\s*GetProcessHeap\(\)\s*,\s*0\s*,\s*"
        r"(\(\*this\)\.[A-Za-z_]\w*)\s*,?\s*\)",
        r"HeapFree(GetProcessHeap(), 0, \1.cast())",
        rust,
    )
    rust = re.sub(
        r"\bStringCchCopyW\s*\(\s*(\(\*this\)\.[A-Za-z_]\w*)\s*,",
        r"StringCchCopyW(\1.as_mut_ptr(),",
        rust,
    )
    rust = re.sub(
        r"\bSHEmptyRecycleBinW\s*\(\s*([^,]+),\s*(\(\*this\)\.[A-Za-z_]\w*)\s*,",
        r"SHEmptyRecycleBinW(\1, \2.as_ptr(),",
        rust,
    )
    # COM callback syntax is fixed by the generated interface vtable.  Some
    # models add one dereference around the interface before selecting lpVtbl
    # and omit the call-parentheses required for a Rust function-pointer field.
    rust = re.sub(
        r"\(\(\*\(\*([A-Za-z_]\w*)\)\)\.lpVtbl\)\.([A-Za-z_]\w*)\(",
        r"((*(*\1).lpVtbl).\2)(",
        rust,
    )
    if re.search(r"\*pdwlSpaceUsed\s*=\s*bin\.i64Size", source):
        rust = re.sub(
            r"\bbin\.i64Size\b(?!\s+as\s+DWORDLONG|\s*=)",
            "bin.i64Size as DWORDLONG",
            rust,
        )
    if re.search(r"\bSIZE_T\s+[A-Za-z_]\w*\s*=\s*lstrlenW", source):
        rust = re.sub(
            r"let\s+cch\s*=\s*lstrlenW\(([^)]+)\)\s*\+\s*1\s*;",
            r"let cch: SIZE_T = lstrlenW(\1) as SIZE_T + 1;",
            rust,
        )
    if re.search(r"CStringW\s*&\s*[A-Za-z_]\w*", source):
        rust = re.sub(r"(?m)^(\s*)paths\s*\+=", r"\1*paths +=", rust)
    # File-scope C arrays decay to pointers at call sites.
    rust = re.sub(
        r"SHELL_MapContextMenuVerbToCmdId\s*\(([^,]+),\s*g_VerbMap\s*\)",
        r"SHELL_MapContextMenuVerbToCmdId(\1, core::ptr::addr_of!(g_VerbMap).cast())",
        rust,
    )
    rust = re.sub(
        r"&g_VerbMap\s+as\s+\*const\s+u8\s+as\s+\*const\s+core::ffi::c_void",
        "core::ptr::addr_of!(g_VerbMap).cast()",
        rust,
    )
    rust = re.sub(r"\bcmd_id\s*==\s*(IDC_[A-Za-z_]\w*)", r"cmd_id == \1 as INT", rust)
    # C null/unsigned initializer semantics are source-authoritative; cheap
    # models often spell them as integer operations that Rust rejects.
    for field in re.findall(r"\b(m_[A-Za-z_]\w*)\s*\(\s*NULL\s*\)", source):
        rust = re.sub(
            rf"(\(\*this\)\.{re.escape(field)}\s*=\s*)0\s*;",
            r"\1core::ptr::null_mut();",
            rust,
        )
    # `m_X(-1)` in a C++ initializer list means ALL-ONES, and what that is spelled as in Rust
    # depends on a signedness this function cannot see: `source` is `dep.body_text`, the
    # function body, while the field's declaration lives in the class HEADER.
    #
    # This used to rewrite to `UINT::MAX`, which is right only when the field is unsigned and
    # is a hard type error when it is not — `CFolderItems::CFolderItems() : m_Count(-1)` over
    # `long m_Count` came out as `(*this).m_Count = UINT::MAX;` and could never compile.
    # The cast keeps ONE spelling correct for both: `-1 as u32` is all-ones, `-1 as i32` is
    # -1, and `as _` takes the field's own type from the assignment. It also keeps the `-1`
    # in the text, which `UINT::MAX` did not — the fact-seal audit counts the C's literals
    # and a rewrite that deletes one is a rewrite that can fail the replay.
    for field in re.findall(r"\b(m_[A-Za-z_]\w*)\s*\(\s*-1\s*\)", source):
        rust = re.sub(
            rf"(\(\*this\)\.{re.escape(field)}\s*=\s*)(?:-1|UINT::MAX)\s*;",
            r"\g<1>-1 as _;",
            rust,
        )
    for field in re.findall(r"\bDestroyIcon\s*\(\s*(m_[A-Za-z_]\w*)\s*\)", source):
        rust = re.sub(
            rf"\(\*this\)\.{re.escape(field)}\s*!=\s*0",
            f"!(*this).{field}.is_null()",
            rust,
        )
    bool_fields = set(re.findall(r"\bBOOL\s+([A-Za-z_]\w*)\s*;", source))
    for field in bool_fields:
        rust = re.sub(rf"\b{re.escape(field)}\s*:\s*false\b", f"{field}: FALSE", rust)
        rust = re.sub(rf"\b{re.escape(field)}\s*:\s*true\b", f"{field}: TRUE", rust)
    rust = re.sub(
        r"(StringCbPrintfW\(\s*[^,]+,\s*core::mem::size_of_val\([^)]*\))\s+as\s+u32",
        r"\1 as usize",
        rust,
    )
    rust = re.sub(r"\bStringCbPrintfW\s*\(", "StringCbPrintfW!(", rust)
    for alias, target in _CPP_SUPPORT_GUID_ALIASES.items():
        rust = re.sub(rf"\b{re.escape(alias)}\b", target, rust)
    owner = re.search(r"\b([A-Za-z_]\w*)::[~A-Za-z_]\w*\s*\(", source)
    if owner is not None:
        class_name = owner.group(1)
        explicit_receivers: dict[str, set[str]] = defaultdict(set)
        for match in re.finditer(r"\b([A-Za-z_]\w*)\s*(?:->|\.)\s*([A-Za-z_]\w*)\s*\(", source):
            explicit_receivers[match.group(2)].add(match.group(1))

        def insert_receiver(match: re.Match) -> str:
            target = match.group("target")
            first = match.group("first") or ""
            if re.search(r"\bfn\s*$", rust[max(0, match.start() - 8) : match.start()]):
                return match.group(0)
            method = target.removeprefix(f"{class_name}_").removesuffix("_body")
            if first == "this" or first in explicit_receivers.get(method, set()):
                return match.group(0)
            return f"{target}(this, " + first

        rust = re.sub(
            rf"\b(?P<target>{re.escape(class_name)}_[A-Za-z_]\w*_body)\s*\(\s*"
            r"(?P<first>[A-Za-z_]\w*)?",
            insert_receiver,
            rust,
        )

    # `delete this` -> the scaffold's own `X_delete`. The model cannot get this right and should
    # never be asked to: C++ hides `operator new` and the vtable store in the constructor
    # PROLOGUE, so the allocation strategy appears in no source it reads. Left to itself it
    # invented five different deallocators for shellrecyclebin's five identical `delete this;` --
    # CRT `free` and `Box::from_raw` over memory that came from `HeapAlloc(GetProcessHeap(), ...)`,
    # plus a placeholder symbol. Those failed to compile HERE only because `Box` is undeclared in
    # a no_std crate without `alloc`; wherever such a name is in scope the file builds clean and
    # frees across heaps, corrupting at Release time arbitrarily far from the allocation.
    #
    # The class comes from the WORKER SIGNATURE, not from `owner` above. `owner` scrapes a
    # `Class::Method(` token out of the body text, and a body that contains no `::` at all --
    # `RecycleBin5::Release`, three lines long -- yields nothing, which silently skipped the
    # lowering for exactly one of the five. `this: *mut <Class>` is in every worker by
    # construction. (The receiver-insertion rewrite above still depends on `owner` and has the
    # same blind spot; left alone here deliberately rather than changed unverified.)
    receiver = re.search(r"\bfn\s+\w+\s*\(\s*this\s*:\s*\*mut\s+([A-Za-z_]\w*)", rust)
    if receiver is not None and re.search(r"\bdelete\s+this\s*;", source):
        owning_class = receiver.group(1)
        deallocators = "|".join(
            [
                r"(?:alloc::boxed::)?Box::from_raw\s*\(\s*this\s*\)",
                r"__rustconv_\w*\s*\(\s*this\s*\)",
                r"free\s*\(\s*this\b[^;]*?\)",
                r"SENDTO_ITEM_delete\s*\(\s*this\s*\)",
            ]
        )
        # `X_delete` runs the destructor itself, so an emitted `X_dtor_body(this);` right before
        # the free has to go with it — keeping both runs every member destructor twice.
        rust = re.sub(
            rf"(?:{re.escape(owning_class)}_dtor_body\s*\(\s*this\s*\)\s*;\s*)?"
            rf"(?:let\s+_\s*=\s*)?(?:drop\s*\(\s*)?(?:{deallocators})\s*\)?\s*;",
            f"{owning_class}_delete(this);",
            rust,
        )

    # The same rule one step out: a FREE function that builds one of these objects.
    #
    #     RecycleBin5File *pThis = new RecycleBin5File();
    #     ...
    #     delete pThis;
    #
    # `--cpp-com --class` never sees this function (it is file-scope, so it belongs to the
    # `--free-functions` half), and that half has no scaffold in front of it -- so the model
    # is again inventing an allocator and a deallocator for a strategy written nowhere it can
    # read. It duly produced `RecycleBin5File()` (calling the STRUCT) and `free(pThis)` for a
    # `HeapAlloc(GetProcessHeap(), ...)` object.
    #
    # The C++ names the static type at the `new` site, which is exactly what `delete v` needs
    # and the one thing the Rust cannot recover: by the time it is `let pThis = ...`, the
    # class name has been erased into whatever the model chose to call the constructor.
    owned_locals = {
        var: cls
        for cls, var in re.findall(
            r"\b([A-Za-z_]\w*)\s*\*\s*([A-Za-z_]\w*)\s*=\s*new\s+\1\b", source
        )
    }
    for var, cls in owned_locals.items():
        # `new X()` with no scaffold in scope became a call to the STRUCT `X()`. The paired
        # allocator is the scaffold's `X_new`, which also performs the vptr store -- so
        # getting this wrong is not merely an unresolved name, it is an object whose lpVtbl
        # stays the HEAP_ZERO_MEMORY null. Unconditional: a factory with no failure path
        # (RecycleBinGeneric_Constructor) news without ever deleting, and gating this on the
        # presence of a `delete` skipped it entirely.
        rust = re.sub(
            rf"(?<![\w:]){re.escape(cls)}\s*\(\s*\)(?!\s*\{{)",
            f"{cls}_new()",
            rust,
        )
        if not re.search(rf"\bdelete\s+{re.escape(var)}\s*;", source):
            continue
        deallocators = [
            rf"(?:alloc::boxed::)?Box::from_raw\s*\(\s*{re.escape(var)}\s*\)",
            rf"__rustconv_\w*\s*\(\s*{re.escape(var)}\s*\)",
            rf"free\s*\(\s*{re.escape(var)}\b[^;]*?\)",
            rf"SENDTO_ITEM_delete\s*\(\s*{re.escape(var)}\s*\)",
        ]
        # The nastiest spelling of all, and the only one that COMPILES: `X_Delete` — capital
        # D — is the class's own COM `Delete` METHOD, which for a recycle-bin file deletes the
        # FILE FROM DISK. `delete pThis` came back as `RecycleBin5File_Delete(pThis)`, one
        # letter away from the deallocator and a completely different operation, resolving to
        # a real symbol with a matching signature. Only claimed when the C++ body never calls
        # `->Delete()` itself, so a body that legitimately does both is left alone.
        if "->Delete(" not in source:
            deallocators.append(rf"{re.escape(cls)}_Delete\s*\(\s*{re.escape(var)}\s*\)")
        deallocators = "|".join(deallocators)
        rust = re.sub(
            rf"(?:{re.escape(cls)}_dtor_body\s*\(\s*{re.escape(var)}\s*\)\s*;\s*)?"
            rf"(?:let\s+_\s*=\s*)?(?:drop\s*\(\s*)?(?:{deallocators})\s*\)?\s*;",
            f"{cls}_delete({var});",
            rust,
        )

    rust = re.sub(
        r"let\s+(?P<name>[A-Za-z_]\w*)\s*=\s*"
        r"(?P<iid>core::ptr::addr_of!\(IID_[A-Za-z_]\w*\))\s*,\s*"
        r"core::ptr::addr_of_mut!\([^;]+?\)\.cast\(\)\s*;",
        r"let \g<name> = \g<iid>;",
        rust,
    )
    rust = re.sub(r"\(\(([^()]+)\)\s*<\s*0\)\s*(?:!=|<)\s*0", r"((\1) < 0)", rust)
    rust = re.sub(r"\bCComHeapPtr::new\(\s*\)", "CComHeapPtr::empty()", rust)
    # Guarded on the source actually naming SENDTO_ITEM. Unguarded, this rewrote EVERY
    # `Box::from_raw(x);` in every module into a call to a shell32 helper: recyclebin's
    # `RecycleBinGeneric::Release` came out calling `SENDTO_ITEM_delete`, a function that does
    # not exist in that crate and has nothing to do with recycle bins. It read like a model
    # hallucination and was ours.
    if "SENDTO_ITEM" in source:
        rust = re.sub(
            r"alloc::boxed::Box::from_raw\(\s*([A-Za-z_]\w*)\s*\)\s*;",
            r"SENDTO_ITEM_delete(\1);",
            rust,
        )
    rust = re.sub(r"\bSENDTO_ITEM\s*\(", "SENDTO_ITEM_new(", rust)
    heap_ptr_names = set(re.findall(r"\bCComHeapPtr\s*<[^;=]+>\s+([A-Za-z_]\w*)", source)) & set(
        re.findall(r"\b([A-Za-z_]\w*)\s*:\s*CComHeapPtr\s*<", rust)
    )
    for name in heap_ptr_names:
        rust = re.sub(rf"&(?:raw\s+)?mut\s+{re.escape(name)}\b", f"{name}.as_mut_slot()", rust)
    com_ptr_names = set(re.findall(r"\bCComPtr\s*<[^;=]+>\s+([A-Za-z_]\w*)", source))
    for name in com_ptr_names:
        rust = re.sub(rf"\b{re.escape(name)}\.as_raw\(\)", name, rust)
    for field in re.findall(r"\bCComPtr\s*<[^;=]+>\s+(m_[A-Za-z_]\w*)", source):
        rust = re.sub(rf"(\(\*this\)\.{re.escape(field)})\.as_raw\(\)", r"\1", rust)
        rust = re.sub(
            rf"\(\*this\)\.{re.escape(field)}\.Release\(\)\s*;",
            f"if !(*this).{field}.is_null() {{\n"
            f"        let value = (*this).{field} as *mut IUnknown;\n"
            f"        ((*(*value).lpVtbl).Release)(value);\n"
            f"        (*this).{field} = core::ptr::null_mut();\n"
            "    }",
            rust,
        )
    rust = re.sub(r"(\(\*this\)\.m_[A-Za-z_]\w*)\.as_raw\(\)", r"\1", rust)
    state_releases = set(re.findall(r"\(\*this\)\.(m_[A-Za-z_]\w*)\.Release\(\)", rust))
    for field in state_releases:
        rust = re.sub(
            rf"\(\*this\)\.{re.escape(field)}\.Release\(\)\s*;",
            f"if !(*this).{field}.is_null() {{\n"
            f"        let value = (*this).{field} as *mut IUnknown;\n"
            f"        ((*(*value).lpVtbl).Release)(value);\n"
            f"        (*this).{field} = core::ptr::null_mut();\n"
            "    }",
            rust,
        )
    rust = re.sub(r"GetAsyncKeyState\((VK_[A-Za-z_]\w*)\)", r"GetAsyncKeyState(\1 as INT)", rust)
    rust = re.sub(r"\bCSIDL_SENDTO\b(?!\s+as\s+INT)", "CSIDL_SENDTO as INT", rust)
    if re.search(
        r"GetSpecialFolder(?:_body)?\s*\(\s*NULL\s*,[^;]+CSIDL_SENDTO\s*\)\s*;",
        source,
    ):
        rust = re.sub(
            r"(CSIDL_SENDTO\s+as\s+INT\s*,?\s*)\)",
            r"\1core::ptr::null_mut())",
            rust,
            count=1,
        )
    rust = re.sub(
        r"((?:InsertMenuW|AppendMenuW)\([^;]*?)\bidCmd\s*,",
        r"\1idCmd as UINT_PTR,",
        rust,
        flags=re.S,
    )
    rust = re.sub(r"let\s+(idCmd)\s*:\s*WORD\s*=\s*LOWORD!", r"let \1: UINT = LOWORD!", rust)
    rust = re.sub(
        r"&\(\*([A-Za-z_]\w*)\)\.pidlChild\s+as\s+\*const\s+PITEMID_CHILD",
        r"core::ptr::addr_of!((*\1).pidlChild).cast::<PCUITEMID_CHILD>()",
        rust,
    )
    rust = re.sub(r"(itemWidth\s*[<=]\s*)(cxMenuCheck)\b", r"\1\2 as UINT", rust)
    rust = re.sub(r"(itemWidth\s*=\s*)(cxMenuCheck)\b", r"\1\2 as UINT", rust)
    rust = re.sub(r"(itemHeight\s*[<=]\s*)(cySmall)\b", r"\1\2 as UINT", rust)
    rust = re.sub(r"(itemHeight\s*=\s*)(cySmall)\b", r"\1\2 as UINT", rust)
    rust = re.sub(r"\b_wcsdup\(([A-Za-z_]\w*\.[A-Za-z_]\w*)\)", r"_wcsdup(\1.as_ptr())", rust)
    rust = re.sub(r"\bLOWORD\s*\(", "LOWORD!(", rust)
    if re.search(r"\*\s*plResult\s*=\s*TRUE", source):
        rust = re.sub(r"\*plResult\s*=\s*TRUE", "*plResult = TRUE as LRESULT", rust)
    rust = re.sub(
        r"&mut\s+(wsz[A-Za-z_]\w*)\s+as\s+\*mut\s+_",
        r"\1.as_mut_ptr().cast()",
        rust,
    )
    rust = re.sub(r"(?m)^(\s*)pData,(\s*)$", r"\1pData.cast(),\2", rust)
    rust = re.sub(r"\b(fi\.hIcon)\s*!=\s*0", r"!\1.is_null()", rust)
    rust = re.sub(
        r"(RegEnumKeyW\([^;]*?),\s*MAX_PATH(\s*\))",
        r"\1, MAX_PATH as DWORD\2",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"(RegEnumKeyW\([^;]*?),\s*MAX_PATH(\s*,\s*\))",
        r"\1, MAX_PATH as DWORD\2",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"(ExpandEnvironmentStringsW\([^;]*?),\s*MAX_PATH(\s*\))",
        r"\1, MAX_PATH as DWORD\2",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"if\s+!([A-Za-z_]\w*_body\([^;{}]*\))",
        r"if \1 == FALSE",
        rust,
    )
    rust = re.sub(
        r"(LoadStringW\([^;]*?,\s*)([A-Za-z_]\w*\.len\(\))(\s*,?\s*\))",
        r"\1\2 as INT\3",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"(\bmii\.cch\s*=\s*wcslen\([^;]+\))\s*;",
        r"\1 as UINT;",
        rust,
    )
    rust = re.sub(
        r"(CNewMenu_SelectNewItem_body\(\s*this\s*,\s*)(SHCNE_[A-Za-z_]\w*)",
        r"\1\2 as LONG",
        rust,
    )
    rust = re.sub(
        r"(CNewMenu_NewItemByNonCommand_body\([^;]*?wszName\.as_mut_ptr\(\),\s*)"
        r"wszName\.len\(\)",
        r"\1wszName.len() as DWORD",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"\blet\s+mut\s+([A-Za-z_]\w*)\s*:\s*HKEY\s*;",
        r"let mut \1: HKEY = core::ptr::null_mut();",
        rust,
    )
    rust = re.sub(r"\blet\s+(wszName)\s*:\s*\*mut", r"let mut \1: *mut", rust)
    if "CreateProcessW" in source:
        rust = rust.replace("core::ptr::null::<u16>()", "core::ptr::null()")
    rust = re.sub(
        r"MessageBoxW\(\s*0\s*,",
        "MessageBoxW(core::ptr::null_mut(),",
        rust,
    )
    if "switch" in source:
        match_start = rust.find("match ")
        if match_start >= 0:
            opening = rust.find("{", match_start)
            try:
                _inner, closing = _balanced_block(rust, opening)
            except CppComError:
                closing = -1
            if closing >= 0 and "_ =>" not in rust[opening:closing]:
                rust = rust[:closing] + "    _ => {}\n" + rust[closing:]
    rust = re.sub(
        r"(pshf:\s*)this\b",
        r"\1core::ptr::addr_of_mut!((*this).IShellFolder_iface)",
        rust,
    )
    rust = rust.replace("(*psd).str", "(*psd).str_")
    rust = rust.replace("(*psd).str_.cStr", "(*psd).str_.u.cStr")
    rust = re.sub(r"(\.cStr\[[^]]+\]\s*=\s*)0(?:u16)?\b", r"\g<1>0i8", rust)
    rust = re.sub(
        r"SHSetStrRet\(\s*&raw\s+mut\s+([^,]+),\s*"
        r"([A-Za-z_]\w*\[[^]]+\]\.colnameid)\s*\)",
        r"cpp_com_SHSetStrRetResource(&raw mut \1, \2 as DWORD)",
        rust,
    )
    rust = re.sub(
        r"SHSetStrRet\(\s*([A-Za-z_]\w*)\s*,\s*"
        r"\(\*([A-Za-z_]\w*)\)\.([A-Za-z_]\w*)\s*\)",
        r"cpp_com_SHSetStrRetW(\1, core::ptr::addr_of!((*\2).\3).cast::<WCHAR>())",
        rust,
    )
    # `ShellObjectCreator<T>` / `ShellObjectCreatorInit<T>` for a SIBLING class.
    #
    # The activation detection above is keyed to the class BEING translated, because its job
    # is to decide how THIS class is constructed. But a body routinely constructs a different
    # one — CFolder::Items builds a CFolderItems, CFolder::ParseName builds a CFolderItem —
    # and C++ instantiates the template per type, so the model renders the turbofish it was
    # given and nothing resolved it. The scaffold already emits `<Class>_CreateInstance` for
    # every class it generates, with exactly the `(init args…, riid, ppv)` signature the
    # template expands to, so the sibling's creator is already there to be called and this is
    # a NAMING gap, not missing codegen.
    #
    # Blanket, not sibling-only: for the class being translated the rewrite lands on that
    # same generated `<Class>_CreateInstance`, so one rule covers both and there is no
    # "is it me or a sibling" test to get wrong.
    # Explicit init-argument types in the template list are dropped for the same reason
    # the detection above ignores them: `<T, T1>` names the class plus the types of the
    # args already present at the call site, and `<Class>_CreateInstance` takes those args
    # positionally. Keeping them would spell a turbofish on a plain function.
    rust = re.sub(
        r"\bShellObjectCreator(?:Init)?\s*::?\s*<\s*([A-Za-z_]\w*)\s*(?:,[^<>]*)?>\s*\(",
        r"\1_CreateInstance(",
        rust,
    )
    # oaidl.h's VARIANT accessors are MACROS, every one of them — `V_VT`, `V_BSTR`, `V_UI4`
    # and the rest are `#define`s reaching into the union (`(*(v)).n1.n2.n3.ulVal`), not
    # functions. A model that has seen them as `V_VT(pvar)` in the C writes the same call
    # shape in Rust, which is `expected function, found macro` at every site.
    #
    # Safe as a blanket family rule precisely because there is no exception: no `V_<NAME>`
    # in that header is a function, so rewriting the call spelling cannot turn a correct
    # call into a wrong one. If the crate has no macro of that name the result is a
    # different and MORE accurate error — "cannot find macro" names the real gap, where
    # "expected function, found macro" pointed at the call.
    rust = re.sub(r"\bV_([A-Z][A-Z0-9_]*)\s*\(", r"V_\1!(", rust)
    # `assert` is a MACRO on both sides -- assert.h's in C, `core::assert!` in Rust -- so a
    # model that copies the C call shape writes `assert(x)` and gets
    # `expected function, found macro`. Same argument as the V_* family above: there is no
    # function spelled `assert` for this to collide with.
    rust = re.sub(r"\bassert\s*\(", "assert!(", rust)
    # An INVENTED macro spelling of `IID_PPV_ARG`, named after the interface:
    # `IID_FolderItems!(FolderItems, ppid)` where the C says `IID_PPV_ARG(FolderItems, ppid)`.
    # The model emits the sanctioned spelling nearly everywhere (CFolderItems' own Item() came
    # back correctly expanded in the same run), so this is a rendering slip rather than a
    # misunderstanding, and it does NOT compile — E0423/E0425 at the call, never silent.
    #
    # Safe to normalize rather than reject because the shape carries its own check: the
    # backreference forces the macro name and the first argument to be the same interface, so
    # a coincidental `IID_X!(Y, …)` is left alone. Rewriting to the sanctioned spelling (not
    # straight to the expansion) keeps ONE implementation of the two-argument expansion below.
    # The second argument routinely CONTAINS parens — `&mut (*this).m_contextmenu` — so it
    # cannot be `[^(),]+`; that spelling silently skipped every out-param that was a member
    # access. One level of nesting is enough for an address-of expression and keeps the
    # pattern from running past the closing paren.
    rust = re.sub(
        r"\bIID_([A-Za-z_]\w*)\s*!\s*\(\s*\1\s*,\s*((?:[^(),]|\([^()]*\))+?)\s*\)",
        r"IID_PPV_ARG(\1, \2)",
        rust,
    )
    # Last, so the alias/receiver rules above have already recovered the real
    # destination from any IID_PPV_ARG they understand.
    return _expand_residual_iid_ppv_arg(rust)


_CPP_SUPPORT_EXTERNS = {
    "CloseHandle": "    fn CloseHandle(handle: HANDLE) -> BOOL;",
    "CopyFileW": "    fn CopyFileW(existing: LPCWSTR, new: LPCWSTR, fail_if_exists: BOOL) -> BOOL;",
    "CreateFileW": (
        "    fn CreateFileW(name: LPCWSTR, access: DWORD, share: DWORD, security: LPVOID, "
        "creation: DWORD, flags: DWORD, template: HANDLE) -> HANDLE;"
    ),
    "CreateMenu": "    fn CreateMenu() -> HMENU;",
    "DestroyMenu": "    fn DestroyMenu(menu: HMENU) -> BOOL;",
    "GetAsyncKeyState": "    fn GetAsyncKeyState(key: INT) -> i16;",
    "InsertMenuW": (
        "    fn InsertMenuW(menu: HMENU, position: UINT, flags: UINT, id: UINT_PTR, "
        "text: LPCWSTR) -> BOOL;"
    ),
    "AppendMenuW": (
        "    fn AppendMenuW(menu: HMENU, flags: UINT, id: UINT_PTR, text: LPCWSTR) -> BOOL;"
    ),
    "SetMenuItemInfoW": (
        "    fn SetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, "
        "info: *const MENUITEMINFOW) -> BOOL;"
    ),
    "CreateProcessW": (
        "    fn CreateProcessW(app: LPCWSTR, command: LPWSTR, process_attrs: LPCVOID, "
        "thread_attrs: LPCVOID, inherit: BOOL, flags: DWORD, environment: LPCVOID, "
        "directory: LPCWSTR, startup: *const STARTUPINFOW, info: *mut PROCESS_INFORMATION) -> BOOL;"
    ),
    "DestroyIcon": "    fn DestroyIcon(icon: HICON) -> BOOL;",
    "CopyIcon": "    fn CopyIcon(icon: HICON) -> HICON;",
    "DestroyWindow": "    fn DestroyWindow(window: HWND) -> BOOL;",
    "DefWindowProcW": (
        "    fn DefWindowProcW(window: HWND, message: UINT, wparam: WPARAM, "
        "lparam: LPARAM) -> LRESULT;"
    ),
    "DispatchMessageW": "    fn DispatchMessageW(message: *const MSG) -> LRESULT;",
    "GetMessageW": (
        "    fn GetMessageW(message: *mut MSG, window: HWND, first: UINT, last: UINT) -> BOOL;"
    ),
    "GetWindowLongPtrW": (
        '    #[cfg_attr(target_pointer_width = "32", link_name = "GetWindowLongW")]\n'
        "    fn GetWindowLongPtrW(window: HWND, index: INT) -> LONG_PTR;"
    ),
    "KillTimer": "    fn KillTimer(window: HWND, id: UINT_PTR) -> BOOL;",
    "LoadIcon": (
        '    #[link_name = "LoadIconW"]\n'
        "    fn LoadIcon(instance: HINSTANCE, name: LPCWSTR) -> HICON;"
    ),
    "LoadIconW": "    fn LoadIconW(instance: HINSTANCE, name: LPCWSTR) -> HICON;",
    "PlaySoundW": ("    fn PlaySoundW(sound: LPCWSTR, module: HANDLE, flags: DWORD) -> BOOL;"),
    "PostQuitMessage": "    fn PostQuitMessage(exit_code: INT);",
    "SetTimer": (
        "    fn SetTimer(window: HWND, id: UINT_PTR, timeout: UINT, callback: "
        'Option<unsafe extern "system" fn(HWND, UINT, UINT_PTR, DWORD)>) -> UINT_PTR;'
    ),
    "SetWindowLongPtrW": (
        '    #[cfg_attr(target_pointer_width = "32", link_name = "SetWindowLongW")]\n'
        "    fn SetWindowLongPtrW(window: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;"
    ),
    "SHCreateWorkerWindowW": (
        "    fn SHCreateWorkerWindowW(callback: WNDPROC, parent: HWND, ex_style: DWORD, "
        "style: DWORD, menu: HMENU, extra: LONG_PTR) -> HWND;"
    ),
    "Shell_NotifyIconW": (
        "    fn Shell_NotifyIconW(message: DWORD, data: *mut NOTIFYICONDATAW) -> BOOL;"
    ),
    "TranslateMessage": "    fn TranslateMessage(message: *const MSG) -> BOOL;",
    "DrawIconEx": (
        "    fn DrawIconEx(dc: HDC, x: INT, y: INT, icon: HICON, width: INT, height: INT, "
        "step: UINT, brush: HANDLE, flags: UINT) -> BOOL;"
    ),
    "ExpandEnvironmentStringsW": (
        "    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;"
    ),
    "ExtractIconExW": (
        "    fn ExtractIconExW(file: LPCWSTR, index: INT, large: *mut HICON, small: *mut HICON, "
        "icons: UINT) -> UINT;"
    ),
    "GetMenuItemInfoW": (
        "    fn GetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, "
        "info: *mut MENUITEMINFOW) -> BOOL;"
    ),
    "GetSystemMetrics": "    fn GetSystemMetrics(index: UINT) -> INT;",
    "IUnknown_QueryService": (
        "    fn IUnknown_QueryService(object: *mut IUnknown, service: REFGUID, riid: REFIID, "
        "out: *mut *mut core::ffi::c_void) -> HRESULT;"
    ),
    "LoadImage": (
        '    #[link_name = "LoadImageW"]\n'
        "    fn LoadImage(instance: HINSTANCE, name: LPCWSTR, kind: UINT, width: INT, "
        "height: INT, flags: UINT) -> HANDLE;"
    ),
    "MessageBoxW": (
        "    fn MessageBoxW(owner: HWND, text: LPCWSTR, caption: LPCWSTR, kind: UINT) -> INT;"
    ),
    "PathYetAnotherMakeUniqueName": (
        "    fn PathYetAnotherMakeUniqueName(buffer: LPWSTR, path: LPCWSTR, short: LPCWSTR, "
        "long: LPCWSTR) -> BOOL;"
    ),
    "RegCloseKey": "    fn RegCloseKey(key: HKEY) -> LONG;",
    "RegCreateKeyEx": (
        '    #[link_name = "RegCreateKeyExW"]\n'
        "    fn RegCreateKeyEx(key: HKEY, subkey: LPCWSTR, reserved: DWORD, class: LPWSTR, "
        "options: DWORD, access: REGSAM, security: LPVOID, result: PHKEY, "
        "disposition: LPDWORD) -> LONG;"
    ),
    "RegEnumKeyW": (
        "    fn RegEnumKeyW(key: HKEY, index: DWORD, name: LPWSTR, capacity: DWORD) -> LONG;"
    ),
    "RegGetValueW": (
        "    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, "
        "kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;"
    ),
    "RegOpenKeyExW": (
        "    fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, "
        "result: PHKEY) -> LONG;"
    ),
    "RegQueryValueExW": (
        "    fn RegQueryValueExW(key: HKEY, value: LPCWSTR, reserved: LPDWORD, kind: LPDWORD, "
        "data: PBYTE, bytes: LPDWORD) -> LONG;"
    ),
    "RegSetValueExW": (
        "    fn RegSetValueExW(key: HKEY, value: LPCWSTR, reserved: DWORD, kind: DWORD, "
        "data: *const BYTE, bytes: DWORD) -> LONG;"
    ),
    "SHChangeNotify": (
        "    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);"
    ),
    "SHCreateDirectory": "    fn SHCreateDirectory(owner: HWND, path: LPCWSTR) -> INT;",
    "SHGetFileInfoW": (
        "    fn SHGetFileInfoW(path: LPCWSTR, attributes: DWORD, info: *mut SHFILEINFOW, "
        "bytes: UINT, flags: UINT) -> DWORD_PTR;"
    ),
    "SHGetPathFromIDListW": (
        "    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, path: LPWSTR) -> BOOL;"
    ),
    "SHILCreateFromPathW": (
        "    fn SHILCreateFromPathW(path: LPCWSTR, pidl: *mut LPITEMIDLIST, "
        "attributes: LPDWORD) -> HRESULT;"
    ),
    "StrRetToStrW": (
        "    fn StrRetToStrW(value: *mut STRRET, pidl: PCUITEMID_CHILD, out: *mut LPWSTR) "
        "-> HRESULT;"
    ),
    "ILCombine": (
        "    fn ILCombine(parent: PCIDLIST_ABSOLUTE, child: PCUITEMID_CHILD) -> PIDLIST_ABSOLUTE;"
    ),
    "WideCharToMultiByte": (
        "    fn WideCharToMultiByte(page: UINT, flags: DWORD, wide: LPCWSTR, wide_len: INT, "
        "multi: LPSTR, multi_len: INT, default_char: LPCSTR, used_default: *mut BOOL) -> INT;"
    ),
    "WriteFile": (
        "    fn WriteFile(file: HANDLE, buffer: LPCVOID, bytes: DWORD, written: LPDWORD, "
        "overlapped: LPVOID) -> BOOL;"
    ),
    "GlobalAlloc": "    fn GlobalAlloc(flags: UINT, bytes: SIZE_T) -> HGLOBAL;",
    "GlobalLock": "    fn GlobalLock(memory: HGLOBAL) -> LPVOID;",
    "GlobalUnlock": "    fn GlobalUnlock(memory: HGLOBAL) -> BOOL;",
    "GlobalFree": "    fn GlobalFree(memory: HGLOBAL) -> HGLOBAL;",
    "OpenClipboard": "    fn OpenClipboard(owner: HWND) -> BOOL;",
    "EmptyClipboard": "    fn EmptyClipboard() -> BOOL;",
    "SetClipboardData": "    fn SetClipboardData(format: UINT, memory: HANDLE) -> HANDLE;",
    "CloseClipboard": "    fn CloseClipboard() -> BOOL;",
    "GetLastError": "    fn GetLastError() -> DWORD;",
    "EnumDisplaySettingsW": (
        "    fn EnumDisplaySettingsW(device: LPCWSTR, mode: DWORD, devmode: *mut DEVMODEW) -> BOOL;"
    ),
    "lstrlenW": "    fn lstrlenW(text: LPCWSTR) -> INT;",
    "CoTaskMemAlloc": "    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;",
    "LoadStringW": (
        "    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;"
    ),
    "CPrintersExtractIconW_CreateInstane": (
        '    #[link_name = "CPrintersExtractIconW_RustBridge"]\n'
        "    fn CPrintersExtractIconW_CreateInstane(pidl: LPCITEMIDLIST, riid: REFIID, "
        "out: *mut LPVOID) -> HRESULT;"
    ),
    "CPrintersEnum_CreateInstance": (
        "    fn CPrintersEnum_CreateInstance(owner: HWND, flags: DWORD, "
        "out: *mut *mut IEnumIDList) -> HRESULT;"
    ),
    "StrChrW": "    fn StrChrW(text: LPCWSTR, ch: WCHAR) -> LPWSTR;",
    "CoTaskMemFree": "    fn CoTaskMemFree(memory: LPVOID);",
    "SHStrDupW": "    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;",
    "ILClone": "    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;",
    "ILFindLastID": "    fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;",
    "ILFree": "    fn ILFree(pidl: LPITEMIDLIST);",
    "ILRemoveLastID": "    fn ILRemoveLastID(pidl: LPITEMIDLIST) -> BOOL;",
    "SHGetDesktopFolder": ("    fn SHGetDesktopFolder(folder: *mut *mut IShellFolder) -> HRESULT;"),
    "SHGetNameFromIDList": (
        "    fn SHGetNameFromIDList(pidl: PCIDLIST_ABSOLUTE, sigdn: SIGDN, "
        "name: *mut PWSTR) -> HRESULT;"
    ),
    "SHCreateShellItem": (
        "    fn SHCreateShellItem(parent: PCIDLIST_ABSOLUTE, folder: *mut IShellFolder, "
        "child: LPCITEMIDLIST, item: *mut *mut IShellItem) -> HRESULT;"
    ),
    "SHCreateShellFolderView": (
        "    fn SHCreateShellFolderView(create: *const SFV_CREATE, "
        "view: *mut *mut IShellView) -> HRESULT;"
    ),
    "SHQueryRecycleBinW": (
        "    fn SHQueryRecycleBinW(root: LPCWSTR, info: *mut SHQUERYRBINFO) -> HRESULT;"
    ),
    "SHEmptyRecycleBinW": (
        "    fn SHEmptyRecycleBinW(owner: HWND, root: LPCWSTR, flags: DWORD) -> HRESULT;"
    ),
    "SHGetSpecialFolderLocation": (
        "    fn SHGetSpecialFolderLocation(owner: HWND, csidl: INT, "
        "pidl: *mut LPITEMIDLIST) -> HRESULT;"
    ),
    "ShellExecuteExW": "    fn ShellExecuteExW(info: *mut SHELLEXECUTEINFOW) -> BOOL;",
    "ILGetNext": "    fn ILGetNext(pidl: PCUIDLIST_RELATIVE) -> LPITEMIDLIST;",
    # Renamed from a SHELL32_CoCreateInitSF overload when shlfolder.cpp flipped to Rust; see
    # the Shell32RustCoCreateInitSF emitter below for why the trampoline outlived the overload.
    "SHELL32_CoCreateInitSFWithCsidl": (
        '    #[link_name = "Shell32RustCoCreateInitSF"]\n'
        "    fn SHELL32_CoCreateInitSFWithCsidl(root: LPCITEMIDLIST, clsid: *const GUID, "
        "csidl: INT, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;"
    ),
    "InsertMenuItemW": (
        "    fn InsertMenuItemW(menu: HMENU, item: UINT, by_position: BOOL, "
        "info: *const MENUITEMINFOW) -> BOOL;"
    ),
    "SHCreateShellItemArrayFromDataObject": (
        "    fn SHCreateShellItemArrayFromDataObject(data: *mut IDataObject, "
        "riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;"
    ),
    "SHELL_MapContextMenuVerbToCmdId": (
        '    #[link_name = "Shell32RustMapContextMenuVerbToCmdId"]\n'
        "    fn SHELL_MapContextMenuVerbToCmdId(info: LPCMINVOKECOMMANDINFO, "
        "map: *const CMVERBMAP) -> INT;"
    ),
    "SHELL_GetCommandStringImpl": (
        '    #[link_name = "Shell32RustGetCommandStringImpl"]\n'
        "    fn SHELL_GetCommandStringImpl(command: SIZE_T, flags: UINT, buffer: LPSTR, "
        "capacity: UINT, map: *const CMVERBMAP) -> HRESULT;"
    ),
}

_CPP_SUPPORT_C_EXTERNS = {
    # msvcrt's thread starter. cdecl (no WINAPI), and it returns `uintptr_t` — spelled
    # `usize` so the width follows the target rather than baking one arch's pointer size.
    # The START ROUTINE is `__stdcall`, not cdecl, which is why it is `extern "system"`
    # inside an `extern "C"` declaration: getting that inner ABI wrong is not a compile
    # error, it is a corrupted stack on i386 at the first thread start.
    "_beginthreadex": (
        "    fn _beginthreadex(security: *mut core::ffi::c_void, stack_size: UINT, "
        'start: Option<unsafe extern "system" fn(*mut core::ffi::c_void) -> UINT>, '
        "arglist: *mut core::ffi::c_void, initflag: UINT, thrdaddr: *mut UINT) -> usize;"
    ),
    "SHELL_GetUIObjectOfAbsoluteItem": (
        "    fn SHELL_GetUIObjectOfAbsoluteItem(owner: HWND, pidl: PCIDLIST_ABSOLUTE, "
        "riid: REFIID, out: *mut LPVOID) -> HRESULT;"
    ),
    "HLM_GetIconW": (
        "    fn HLM_GetIconW(index: UINT, destination: LPWSTR, length: DWORD, "
        "icon_index: *mut INT) -> BOOL;"
    ),
    "_ILIsDesktop": "    fn _ILIsDesktop(pidl: LPCITEMIDLIST) -> BOOL;",
    "_ILIsSpecialFolder": "    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;",
    "_ILGetDataPointer": "    fn _ILGetDataPointer(pidl: LPCITEMIDLIST) -> *mut PIDLDATA;",
    "SHELL32_CompareDetails": (
        '    #[link_name = "CPrinterFolder_CompareDetails_RustBridge"]\n'
        "    fn SHELL32_CompareDetails(folder: *mut IShellFolder2, lparam: LPARAM, "
        "left: PCUIDLIST_RELATIVE, right: PCUIDLIST_RELATIVE) -> HRESULT;"
    ),
    "_ILCreateMyDocuments": "    fn _ILCreateMyDocuments() -> LPITEMIDLIST;",
    "memcpy": (
        '    #[link_name = "memcpy"]\n'
        "    fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, "
        "bytes: usize) -> *mut core::ffi::c_void;"
    ),
    "malloc": "    fn malloc(bytes: usize) -> *mut ();",
    "wcsstr": "    fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR;",
    "_wcsdup": "    fn _wcsdup(text: LPCWSTR) -> LPWSTR;",
    "_wcsicmp": "    fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;",
}

_CPP_SUPPORT_GUIDS = {
    "BHID_DataObject",
    "BHID_SFObject",
    "BHID_SFUIObject",
    "BHID_SFViewObject",
    "CLSID_ShellItem",
    "CLSID_AdminFolderShortcut",
    "CLSID_FontsFolderShortcut",
    "CLSID_ShellFSFolder",
    "CLSID_MyDocuments",
    "IID_IDataObject",
    "IID_IShellFolder",
    "IID_IShellFolder2",
}
_CPP_SUPPORT_GUID_ALIASES = {"SID_SFolderView": "IID_IFolderView"}

# GUIDs whose C name is a guiddef.h MACRO, not a data symbol: `#define CLSID_NULL GUID_NULL`,
# so nothing exports `CLSID_NULL` and the extern must be bound to what uuid actually provides
# (`GUID_NULL`, sdk/lib/uuid/cguid_i.c). Two already-landed modules carry this by hand
# (`shell_dispatch.rs`, `cfs_folder.rs`); emitting it here means the next module does not have
# to discover it as a LINK error after everything already compiles.
#
# Bound with `#[link_name]` rather than through `_CPP_SUPPORT_GUID_ALIASES` above, which
# rewrites the IDENTIFIER: `CLSID_NULL` also occurs inside the ATL debug string
# "IDispatchImpl: plibid is CLSID_NULL!", and a blind rewrite would edit the message text.
_CPP_SUPPORT_GUID_LINK_NAMES = {"CLSID_NULL": "GUID_NULL"}


def _guid_extern_decl(name: str) -> str:
    """One `static NAME: GUID;`, with a `#[link_name]` when the C name is a macro alias."""
    link_name = _CPP_SUPPORT_GUID_LINK_NAMES.get(name)
    prefix = f'    #[link_name = "{link_name}"]\n' if link_name else ""
    return f"{prefix}    static {name}: GUID;"


def _rust_numeric_literal(value: str) -> str:
    """A C numeric literal respelled so Rust accepts it, with the VALUE untouched.

    C does not care about the case of a base prefix and shlobj.h writes `0X0017`; Rust rejects
    it outright ("invalid base prefix for number literal"), and a parse error there cascades
    into a wall of unrelated-looking resolution failures across the whole crate.

    Only the PREFIX is lowered — the digits are left exactly as the header wrote them, because
    the fact seal compares them and normalising digits too would be changing a constant rather
    than respelling it.
    """
    return re.sub(r"\b0([XBO])", lambda m: "0" + m.group(1).lower(), value.strip())


def _render_cmverbmap(source: str) -> str:
    match = re.search(
        r"static\s+const\s+CMVERBMAP\s+([A-Za-z_]\w*)\s*\[\s*\]\s*=\s*\{(?P<body>.*?)\}\s*;",
        source,
        flags=re.S,
    )
    if match is None:
        return ""
    entries = []
    for item in re.finditer(
        r"\{\s*(?P<verb>\"(?:\\.|[^\"])*\"|NULL)\s*(?:,\s*(?P<cmd>[A-Za-z_]\w*|\d+))?\s*\}",
        match.group("body"),
    ):
        token, command = item.group("verb"), item.group("cmd") or "0"
        verb = "core::ptr::null()" if token == "NULL" else f'b{token[:-1]}\\0".as_ptr() as LPCSTR'
        entries.append(f"    CMVERBMAP {{ Verb: {verb}, CmdId: {command} as WORD }},")
    if not entries:
        raise CppComError("CMVERBMAP table was found but no aggregate entries were parsed")
    return "\n".join(
        [
            "#[repr(C)]",
            "pub struct CMVERBMAP { pub Verb: LPCSTR, pub CmdId: WORD }",
            f"pub static mut {match.group(1)}: [CMVERBMAP; {len(entries)}] = [",
            *entries,
            "];",
        ]
    )


def _render_shvheader_tables(source: str, used_words: set[str]) -> str:
    """Render shell folder column tables from their file-scope C authority."""
    chunks = []
    table = re.compile(
        r"static\s+(?:const\s+)?shvheader\s+([A-Za-z_]\w*)\s*\[\s*\]\s*=\s*"
        r"\{(?P<body>.*?)\}\s*;",
        re.S,
    )
    for match in table.finditer(source):
        name = match.group(1)
        if name not in used_words:
            continue
        entries = []
        for item in re.finditer(r"\{([^{}]+)\}", match.group("body")):
            fields = [part.strip() for part in item.group(1).split(",")]
            if len(fields) != 4 or any(not field for field in fields):
                raise CppComError(f"{name}: shvheader entry is not a four-field aggregate")
            entries.append(
                "    shvheader { colnameid: "
                + f"({fields[0]}) as WORD, colstate: ({fields[1]}) as WORD, "
                + f"fmt: ({fields[2]}) as WORD, cxChar: ({fields[3]}) as WORD }},"
            )
        if not entries:
            raise CppComError(f"{name}: shvheader table has no parsed entries")
        chunks.extend(
            [
                "#[repr(C)]",
                "pub struct shvheader {",
                "    pub colnameid: WORD, pub colstate: WORD,",
                "    pub fmt: WORD, pub cxChar: WORD,",
                "}",
                f"pub static {name}: [shvheader; {len(entries)}] = [",
                *entries,
                "];",
            ]
        )
    return "\n".join(chunks)


_CPP_SUPPORT_CONSTANTS = {
    "CSIDL_BITBUCKET": ("INT", "0x000a"),
    "CSIDL_PERSONAL": ("INT", "0x0005"),
    "ENUM_CURRENT_SETTINGS": ("DWORD", "0xFFFF_FFFF"),
    # fileapi.h `#define INVALID_FILE_ATTRIBUTES ((DWORD)-1)`. Absent from the island, and
    # declared here rather than evaluated because the CAST is the type: written as a bare
    # `-1` it is an `int`, and every `GetFileAttributesW(...) == INVALID_FILE_ATTRIBUTES`
    # comparison would be a mismatch against the DWORD the API returns.
    "INVALID_FILE_ATTRIBUTES": ("DWORD", "0xFFFF_FFFF"),
    # winuser.h:180 `#define ENUM_REGISTRY_SETTINGS ((DWORD)-2)`; :179 is -1.
    "ENUM_REGISTRY_SETTINGS": ("DWORD", "0xFFFF_FFFE"),
    "EVCCBF_LASTNOTIFICATION": ("DWORD", "0x0001"),
    "E_UNEXPECTED": ("HRESULT", "0x8000_FFFFu32 as HRESULT"),
    "EVCF_HASSETTINGS": ("DWORD", "0x0001"),
    "IDI_ERROR": ("LPCWSTR", "32513usize as LPCWSTR"),
    "IDI_INFORMATION": ("LPCWSTR", "32516usize as LPCWSTR"),
    "IDI_WARNING": ("LPCWSTR", "32515usize as LPCWSTR"),
    "IDI_WINLOGO": ("LPCWSTR", "32517usize as LPCWSTR"),
    "MAX_PATH": ("usize", "260"),
    "NIN_BALLOONHIDE": ("DWORD", "WM_USER + 3"),
    "NIN_BALLOONSHOW": ("DWORD", "WM_USER + 2"),
    "NIN_BALLOONTIMEOUT": ("DWORD", "WM_USER + 4"),
    "NIN_BALLOONUSERCLICK": ("DWORD", "WM_USER + 5"),
    "SEE_MASK_IDLIST": ("ULONG", "0x0000_0004"),
    "SHERB_NOCONFIRMATION": ("DWORD", "0x0000_0001"),
    "SHERB_NOPROGRESSUI": ("DWORD", "0x0000_0002"),
    "SHERB_NOSOUND": ("DWORD", "0x0000_0004"),
    "SW_SHOWNORMAL": ("INT", "1"),
    "WM_TRAYNOTIFY": ("UINT", "WM_USER + 100"),
    "SFGAO_CANCOPY": ("SFGAOF", "0x0000_0001"),
    "SFGAO_CANLINK": ("SFGAOF", "0x0000_0004"),
    "SFGAO_CANRENAME": ("SFGAOF", "0x0000_0010"),
    "SFGAO_CANDELETE": ("SFGAOF", "0x0000_0020"),
    "SFGAO_HASPROPSHEET": ("SFGAOF", "0x0000_0040"),
}


def _shared_substrate_type_names(repo_root: Path) -> set[str]:
    names = set()
    for path in (
        repo_root / "sdk/lib/rust/shell32-rust/src/types.rs",
        repo_root / "sdk/lib/rust/shell32-rust/src/shellcom.rs",
    ):
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        names.update(
            re.findall(r"\bpub\s+(?:struct|union|enum|type)\s+(?:r#)?([A-Za-z_]\w*)", text)
        )
    return names


def _shared_substrate_value_names(repo_root: Path) -> set[str]:
    names = set()
    for path in (
        repo_root / "sdk/lib/rust/shell32-rust/src/types.rs",
        repo_root / "sdk/lib/rust/shell32-rust/src/shellcom.rs",
    ):
        if path.is_file():
            names.update(
                re.findall(
                    r"\bpub\s+(?:const|static(?:\s+mut)?)\s+([A-Za-z_]\w*)",
                    path.read_text(encoding="utf-8", errors="replace"),
                )
            )
    return names


def _sdk_pod_support(source_path: Path, used_words: set[str]) -> str:
    """Generate the simple referenced SDK POD closure from header authority."""
    repo_root = next(
        (parent for parent in source_path.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is None:
        return ""
    shared = _shared_substrate_type_names(repo_root)
    variants: dict[str, set[tuple[str, tuple[tuple[str, bool], ...], bool]]] = defaultdict(set)
    typedef = re.compile(
        r"typedef\s+struct(?:\s+[A-Za-z_]\w*)?\s*\{(?P<body>[^{}]*)\}\s*"
        r"(?P<aliases>[^;]+);",
        re.S,
    )
    header_roots = [repo_root / "sdk/include"] + [
        repo_root / f"build-{arch}" / "sdk" / "include" / "psdk"
        for arch in ("i386", "amd64", "arm64")
    ]
    headers = sorted(path for root in header_roots if root.is_dir() for path in _header_files(root))
    for header in headers:
        text = header.read_text(encoding="utf-8", errors="replace")
        for match in typedef.finditer(text):
            aliases = []
            for raw in match.group("aliases").split(","):
                token = " ".join(raw.split())
                alias = re.fullmatch(r"(?P<pointer>\*+\s*)?(?P<name>[A-Za-z_]\w*)", token)
                if alias is not None:
                    aliases.append((alias.group("name"), bool(alias.group("pointer"))))
            base = next((name for name, pointer in aliases if not pointer), None)
            if base is None:
                continue
            prefix = text[: match.start()]
            packed = prefix.rfind("pshpack1.h") > prefix.rfind("poppack.h")
            body = " ".join(match.group("body").split())
            variants[base].add((body, tuple(aliases), packed))

    # Identical repeated declarations are harmless; conflicting layouts or
    # aliases are not. Keep ambiguity visible to the compiler/substrate plan.
    records = {
        base: (body, list(aliases), packed)
        for base, found in variants.items()
        if len(found) == 1
        for body, aliases, packed in found
    }

    alias_to_base = {
        alias: base for base, (_body, aliases, _packed) in records.items() for alias, _ in aliases
    }
    selected = {
        alias_to_base[name]
        for name in used_words - shared
        if name in alias_to_base and alias_to_base[name] not in shared
    }
    # This SDK record contains optional interface pointers whose pointee types
    # are not consistently exposed as compiler-facing vtables.  Its layout is
    # stable and owned by the explicit support block below, which represents
    # those ABI-equivalent optional pointers as c_void.
    selected.discard("SFV_CREATE")
    # An ATL container has a Rust lowering in `crate::atl` that the prelude re-exports, so
    # rendering the header's own definition here emits a SHADOW that wins over it by scope.
    # Worse than a duplicate: this renderer erases template parameters, so ATL's
    # `CSimpleArray<T>` came out as a non-generic `CSimpleArray { m_aT: *mut c_void, ... }` —
    # right layout, wrong type, no `new()`, no `Drop`. The container's whole reason for
    # keeping its type is that `Drop` restores the C++ member destructor, and a shadow
    # silently takes that away again.
    for container in _RAII_CONTAINER_LOWERINGS:
        selected.discard(container)
    # Embedded SDK records are part of the same deterministic layout closure.
    changed = True
    while changed:
        changed = False
        for base in list(selected):
            body = records[base][0]
            for dependency in set(re.findall(r"\b[A-Za-z_]\w*\b", body)) & alias_to_base.keys():
                target = alias_to_base[dependency]
                if target not in selected and target not in shared:
                    selected.add(target)
                    changed = True

    all_bounds = {
        bound
        for base in selected
        for bound in re.findall(r"\[\s*([A-Za-z_]\w*)\s*\]", records[base][0])
    }
    numeric = {
        name: int(value, 0)
        for name, value in _sdk_numeric_definitions(source_path, all_bounds).items()
    }
    chunks = []
    for base in sorted(selected):
        body, aliases, packed = records[base]
        declarations = [
            declaration
            for item in body.split(";")
            if item.strip()
            # One C statement, several members: `INT8 SortCol, SortReverse;`.
            for declaration in _expand_declarators(
                re.sub(r"\b_[A-Za-z0-9_]+_\b(?:\s*\([^)]*\))?\s*", "", " ".join(item.split()))
            )
        ]
        fields = []
        try:
            for declaration in declarations:
                fields.append(
                    _parse_plain_state_field(
                        declaration,
                        allowed_pod_types=set(records) | _POD_LEAF_TYPES,
                        array_constants=numeric,
                    )
                )
        except CppComError:
            continue
        representation = "#[repr(C, packed)]" if packed else "#[repr(C)]"
        lines = [representation, f"pub struct {base} {{"]
        for state_field in fields:
            field_name = _rust_safe_field(state_field.name)
            # Identical to `rust_type` for a C SDK record, which cannot hold an ATL member;
            # spelled this way so every POD renderer states the same ownership invariant.
            lines.append(f"    pub {field_name}: {state_field.owned_rust_type},")
        lines.append("}")
        for alias, pointer in aliases:
            if alias != base and (alias in used_words or alias in shared):
                target = f"*mut {base}" if pointer else base
                lines.append(f"pub type {alias} = {target};")
        chunks.append("\n".join(lines))
    return "\n\n".join(chunks)


def _sdk_interface_support(
    source_path: Path, used_words: set[str], declared_types: set[str]
) -> tuple[str, set[str], set[str]]:
    """Render the flattened SDK interface closure referenced by worker bodies.

    Slot order and signatures come only from compiler-facing WIDL headers. All
    available architecture headers must agree before one shared Rust vtable is
    emitted.
    """
    repo_root = next(
        (parent for parent in source_path.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is None:
        return "", set(), set()
    indexes: dict[str, dict[str, Path]] = {}
    shared_found: dict[str, list[Path]] = defaultdict(list)
    shared_root = repo_root / "sdk" / "include" / "reactos"
    if shared_root.is_dir():
        for header in sorted(shared_root.glob("*.h")):
            text = header.read_text(encoding="utf-8", errors="replace")
            names = set(re.findall(r"typedef\s+struct\s+([A-Za-z_]\w*)Vtbl\b", text))
            names.update(re.findall(r"DECLARE_INTERFACE_\s*\(\s*([A-Za-z_]\w*)\s*,", text))
            for name in names:
                shared_found[name].append(header)
    shared_index = {name: paths[0] for name, paths in shared_found.items() if len(paths) == 1}
    for arch in ("i386", "amd64", "arm64"):
        root = repo_root / f"build-{arch}" / "sdk" / "include" / "psdk"
        if not root.is_dir():
            continue
        found: dict[str, list[Path]] = defaultdict(list)
        for header in sorted(root.glob("*.h")):
            text = header.read_text(encoding="utf-8", errors="replace")
            names = set(re.findall(r"typedef\s+struct\s+([A-Za-z_]\w*)Vtbl\b", text))
            names.update(re.findall(r"DECLARE_INTERFACE_\s*\(\s*([A-Za-z_]\w*)\s*,", text))
            for name in names:
                found[name].append(header)
        index = {name: paths[0] for name, paths in found.items() if len(paths) == 1}
        for name, header in shared_index.items():
            index.setdefault(name, header)
        indexes[arch] = index
    if not indexes:
        return "", set(), set()
    reference_arch = "amd64" if "amd64" in indexes else next(iter(indexes))
    reference = indexes[reference_arch]
    queue = sorted((used_words & reference.keys()) - declared_types)
    specs: dict[str, InterfaceSpec] = {}
    while queue:
        name = queue.pop(0)
        if name in specs or name in declared_types:
            continue
        spec = parse_widl_interface(reference[name], name)
        for arch, index in indexes.items():
            if arch == reference_arch or name not in index:
                continue
            other = parse_widl_interface(index[name], name)
            if other.methods != spec.methods:
                raise CppComError(
                    f"{name}: compiler-facing WIDL vtable differs on {reference_arch}/{arch}"
                )
        specs[name] = spec
        # Include return and parameter interface pointees recursively.
        dependencies: set[str] = set()
        for method in spec.methods:
            dependencies.update(
                token
                for c_type, _param in method.params
                for token in re.findall(r"\bI[A-Z][A-Za-z0-9_]*\b", c_type)
            )
            dependencies.update(re.findall(r"\bI[A-Z][A-Za-z0-9_]*\b", method.return_type))
        queue.extend(sorted((dependencies & reference.keys()) - specs.keys() - declared_types))
    rendered = "\n\n".join(render_interface(specs[name]) for name in sorted(specs))
    interface_words = set(re.findall(r"\b[A-Za-z_]\w*\b", rendered))
    return rendered, {f"IID_{name}" for name in specs}, interface_words


def render_cpp_support(
    source_path: Path,
    exclude_guids: set[str] | None = None,
    additional_source: str = "",
    exclude_types: set[str] | None = None,
    provided_types: set[str] | None = None,
    island_types: set[str] | None = None,
) -> str:
    """Render the deterministic non-class support closure selected by one C++ file."""
    source = source_path.read_text(encoding="utf-8", errors="replace")
    chunks = ["// Deterministic C++ support closure selected from source identifiers."]

    # Local numeric macros and shell32 resource IDs are compiler-facing facts.
    definitions = dict(
        re.findall(
            r"^\s*#define\s+([A-Za-z_]\w*)\s+\(?\s*"
            r"([0-9]+|0[xX][0-9A-Fa-f]+)\s*\)?\s*(?://[^\n]*)?$",
            source,
            re.M,
        )
    )
    resource_header = next(
        (
            parent / "shresdef.h"
            for parent in source_path.parents
            if (parent / "shresdef.h").is_file()
        ),
        None,
    )
    if resource_header is not None:
        resource_text = resource_header.read_text(encoding="utf-8", errors="replace")
        definitions.update(
            re.findall(
                r"^\s*#define\s+([A-Za-z_]\w*)\s+\(?\s*"
                r"([0-9]+|0[xX][0-9A-Fa-f]+)\s*\)?\s*$",
                resource_text,
                re.M,
            )
        )
    selection_source = source + "\n" + additional_source
    used_words = set(re.findall(r"\b[A-Za-z_]\w*\b", _masked_cpp(selection_source)))
    # A `--local-raii NAME=PATH` lowering IS the definition of NAME for this run, so the
    # substrate must not also emit its own. Both spellings compile; they are simply
    # different types, and the class fields end up holding one while the bodies call the
    # other's methods (the recyclebin substrate's `CStringW { buffer, len, capacity }`
    # against the ported ATL one's `m_pszData`/`m_nLength`/`m_nAlloc`).
    used_words -= provided_types or set()
    if re.search(r"\bshell32_hInstance\b", selection_source):
        used_words.add("shell32_hInstance")
    used_words.update(
        f"IID_{interface}"
        for interface in re.findall(rf"\b{_PPV_ARG_MACRO}\s*\(\s*([A-Za-z_]\w*)", source)
    )
    if "ShellObjectCreatorInit<CPrintersEnum>" in source:
        used_words.add("CPrintersEnum_CreateInstance")
    declared_types = set(
        re.findall(
            r"\bpub\s+(?:struct|union|enum|type)\s+(?:r#)?([A-Za-z_]\w*)",
            additional_source,
        )
    )
    # Interfaces the bindgen island owns. `additional_source` cannot reveal them: an
    # island-provided interface is deliberately NOT re-rendered, so it leaves no
    # `pub struct` for the scan above to find, and this closure would then render its own
    # second copy of the very type render_interface just declined to duplicate.
    declared_types.update(island_types or set())
    repo_root = next(
        (parent for parent in source_path.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is not None:
        declared_types.update(_shared_substrate_type_names(repo_root))
    shared_values = _shared_substrate_value_names(repo_root) if repo_root is not None else set()
    # File-scope PODs are source authority, not model work. This is shared by C
    # and C++ inputs; only simple, fully parsed layouts are emitted.
    source_pod_names = set(re.findall(r"\bstruct\s+([A-Za-z_]\w*)\s*\{", source))

    def _pod_is_referenced(name: str) -> bool:
        # Count the TAG **and its typedef aliases**. A record declared
        # `typedef struct _X { … } X, *PX;` is referred to as `X`/`PX` everywhere and the tag
        # `_X` appears exactly once — in its own declaration — so a tag-only test read the
        # struct as unused and dropped it. `_FILE_TYPE_GLOBALS` is that shape, and the miss
        # surfaced as `cannot find type PFILE_TYPE_GLOBALS` in a DIFFERENT struct's field.
        spellings = [name, *(alias for alias, _pointer in _typedef_aliases(source_path, name))]
        hits = sum(len(re.findall(rf"\b{re.escape(word)}\b", source)) for word in spellings)
        if hits > 1:
            return True
        return any(re.search(rf"\b{re.escape(word)}\b", additional_source) for word in spellings)

    referenced_pods = {name for name in source_pod_names if _pod_is_referenced(name)}
    for name in sorted(referenced_pods - (exclude_types or set())):
        definition = _pod_definition(source, name)
        if definition is None:
            continue
        body, packed = definition
        declarations = [
            declaration
            for item in body.split(";")
            if item.strip()
            # One C statement, several members: `INT8 SortCol, SortReverse;`.
            for declaration in _expand_declarators(" ".join(item.split()))
        ]
        constants = _numeric_array_constants(source_path, declarations)
        fields = []
        try:
            for declaration in declarations:
                fields.append(
                    _parse_plain_state_field(
                        declaration,
                        allowed_pod_types=source_pod_names | _POD_LEAF_TYPES,
                        array_constants=constants,
                    )
                )
        except CppComError:
            continue
        representation = "#[repr(C, packed)]" if packed else "#[repr(C)]"
        lines = [representation, f"pub struct {name} {{"]
        for state_field in fields:
            field_name = _rust_safe_field(state_field.name)
            # `owned_rust_type`, not `rust_type`: a free-function module has no generated
            # Release to run lifetimes for it, so the struct's drop glue is its ONLY
            # destructor and an ATL member flattened to a raw pointer leaks silently.
            # For a plain C field the two spellings are identical by construction.
            lines.append(f"    pub {field_name}: {state_field.owned_rust_type},")
        lines.append("}")
        # `typedef struct _X { … } X, *PX;` — emit the aliases, which are how the source
        # actually spells this record. The SDK-POD renderer already does this; the
        # SOURCE-POD renderer did not, so the struct landed under a tag nothing refers to.
        for alias, pointer in _typedef_aliases(source_path, name):
            if alias == name:
                continue
            if alias not in used_words and alias not in shared_values:
                continue
            target = f"*mut {name}" if pointer else name
            lines.append(f"pub type {alias} = {target};")
        chunks.append("\n".join(lines))
    for name in sorted((used_words & definitions.keys()) - shared_values):
        chunks.append(f"pub const {name}: UINT = {_rust_numeric_literal(definitions[name])};")
    for name in sorted((used_words & _CPP_SUPPORT_CONSTANTS.keys()) - shared_values):
        rust_type, value = _CPP_SUPPORT_CONSTANTS[name]
        chunks.append(f"pub const {name}: {rust_type} = {value};")
    # SDK wide-string macros, written out as real UTF-16 -- see `_sdk_wide_string_macros`
    # for why the island's version of these is the ANSI bytes wearing the wide name.
    for name, units in sorted(_sdk_wide_string_macros(source_path, used_words).items()):
        if name in definitions or name in _CPP_SUPPORT_CONSTANTS or name in shared_values:
            continue
        values = ", ".join(f"0x{unit:04X}" for unit in units)
        chunks.append(
            f"pub static {name}_WIDE: [WCHAR; {len(units)}] = [{values}];\n"
            f"pub const {name}: LPCWSTR = {name}_WIDE.as_ptr();"
        )
    owned_constants = definitions.keys() | _CPP_SUPPORT_CONSTANTS.keys() | shared_values
    for name, value in sorted(_sdk_numeric_definitions(source_path, used_words).items()):
        if name not in owned_constants:
            chunks.append(f"pub const {name}: DWORD = {_rust_numeric_literal(value)};")
    # MODULE-PRIVATE headers hold compiler-facing constants too, and the two scans above reach
    # neither kind: the local `#define` sweep reads only the TU's own .cpp and admits only a
    # bare literal, while `_sdk_numeric_definitions` searches `sdk/include`. shell32's
    # `RECYCLEBINMAXDRIVECOUNT ('Z' - 'A' + 1)` lives in `shellrecyclebin/recyclebin.h` and is
    # an EXPRESSION, so it fell through both and left `cannot find value` in every body that
    # sizes an array by it.
    #
    # Resolved through the same agreement-and-evaluate path as an array bound, so a constant
    # spelled differently in two headers stays unproven rather than being guessed, and only
    # arch-independent expressions evaluate (`'Z' - 'A' + 1` is 26 everywhere).
    # MACRO-SHAPED names only. `used_words` is every identifier in the selected bodies, and
    # the headers define lowercase ones too -- C++'s `true`/`false` among them -- so an
    # unrestricted sweep emitted `pub const false: UINT = 0;`, which is not even parseable
    # Rust. Screaming case is the same test the symbolic-constant audit uses.
    # MODULE-PRIVATE headers only. Widened to `sdk/include` as well, this evaluates SDK
    # macros the island ALREADY provides and shadows them with a worse type: `LB_ERR` is
    # `pub const LB_ERR: i32 = -1` in the island, and a second `UINT = 0xFFFFFFFF` beside it
    # is not a fix, it is a conflict. An SDK constant the island lacks is a substrate gap and
    # must stay loud; a macro in the module's own tree has nowhere else to come from.
    unresolved = {
        word for word in used_words - owned_constants if re.fullmatch(r"[A-Z][A-Z0-9_]*", word)
    } - set(_sdk_numeric_definitions(source_path, used_words))
    for name, value in sorted(_module_private_int_macros(source_path, unresolved).items()):
        if name not in owned_constants:
            # C's `((DWORD)-1)` is a NEGATIVE integer expression with an unsigned type, and
            # Rust cannot negate a `u32` at all (`pub const X: UINT = -1;` is E0600). Respell
            # it as the two's-complement value the C means, which is what the cast does.
            # `INT`, not `UINT`: a bare arithmetic macro like `('Z' - 'A' + 1)` has type
            # `int` in C, and typing it unsigned makes every `i32` comparison that uses it
            # a mismatch.
            chunks.append(f"pub const {name}: INT = {_rust_numeric_literal(str(value))};")
    sdk_pods = _sdk_pod_support(source_path, used_words)
    if sdk_pods:
        chunks.append(sdk_pods)
        # SDK POD fields can introduce interface/type dependencies that did not
        # occur in the selected workers themselves (SFV_CREATE's callback is a
        # representative shell32 case).  Feed those deterministic dependencies
        # back into interface discovery rather than leaving a dangling type.
        used_words.update(re.findall(r"\b[A-Za-z_]\w*\b", sdk_pods))

    # Header-owned constant UTF-16 arrays are source facts just like literals
    # inside method bodies. Emit only referenced, fully literal definitions.
    # Deduped by NAME: `selection_source` can present one declaration more than once (the
    # same header reached twice through the selection closure), and emitting the pair twice
    # is `E0428: the name ... is defined multiple times` on a file that is otherwise correct.
    # A second definition is never wanted, so this needs no cleverness about which wins.
    emitted_wide_arrays: set[str] = set()
    for match in re.finditer(
        r"\b(?:static\s+)?const\s+WCHAR\s+([A-Za-z_]\w*)\s*\[\s*\]\s*=\s*"
        r"(L\"(?:\\.|[^\"\\])*\")\s*;",
        selection_source,
    ):
        name, token = match.groups()
        if name not in used_words or name in emitted_wide_arrays:
            continue
        emitted_wide_arrays.add(name)
        units = _wide_literal_units(token)
        values = ", ".join(f"0x{unit:04X}" for unit in units)
        chunks.append(
            f"pub static {name}_WIDE: [WCHAR; {len(units)}] = [{values}];\n"
            f"pub const {name}: LPCWSTR = {name}_WIDE.as_ptr();"
        )

    interface_support, interface_iids, interface_words = _sdk_interface_support(
        source_path, used_words, declared_types
    )
    if interface_support:
        chunks.append(interface_support)
        used_words.update(interface_words)
        interface_pods = _sdk_pod_support(source_path, interface_words)
        if interface_pods:
            chunks.append(interface_pods)
        for name, value in sorted(_sdk_numeric_definitions(source_path, interface_words).items()):
            if name not in owned_constants:
                chunks.append(f"pub const {name}: DWORD = {_rust_numeric_literal(value)};")

    if "INVALID_FILETITLE_CHARACTERSW" in used_words:
        chunks.append(
            "pub static INVALID_FILETITLE_CHARACTERSW: [WCHAR; 10] = "
            "[0x005c, 0x002f, 0x003a, 0x002a, 0x003f, 0x0022, 0x003c, 0x003e, 0x007c, 0];"
        )

    if "SFV_CREATE" in used_words and not re.search(r"\bstruct\s+SFV_CREATE\b", "\n".join(chunks)):
        chunks.append(
            "#[repr(C)]\n"
            "pub struct SFV_CREATE {\n"
            "    pub cbSize: UINT,\n"
            "    pub pshf: *mut IShellFolder,\n"
            "    pub psvOuter: *mut core::ffi::c_void,\n"
            "    pub psfvcb: *mut core::ffi::c_void,\n"
            "}"
        )

    if "NOTIFYICONDATAW" in used_words:
        chunks.append(
            "pub type WPARAM = UINT_PTR;\n"
            "pub type LRESULT = LONG_PTR;\n"
            "#[repr(C)]\n"
            "pub struct MSG {\n"
            "    pub hwnd: HWND, pub message: UINT, pub wParam: WPARAM,\n"
            "    pub lParam: LPARAM, pub time: DWORD, pub pt: POINT,\n"
            "}\n"
            'pub type WNDPROC = Option<unsafe extern "system" fn(\n'
            "    HWND, UINT, WPARAM, LPARAM,\n"
            ") -> LRESULT>;\n"
            "#[repr(C)]\n"
            "pub struct NOTIFYICONDATAW {\n"
            "    pub cbSize: DWORD, pub hWnd: HWND, pub uID: UINT, pub uFlags: UINT,\n"
            "    pub uCallbackMessage: UINT, pub hIcon: HICON, pub szTip: [WCHAR; 128],\n"
            "    pub dwState: DWORD, pub dwStateMask: DWORD, pub szInfo: [WCHAR; 256],\n"
            "    pub uTimeout: UINT, pub szInfoTitle: [WCHAR; 64], pub dwInfoFlags: DWORD,\n"
            "    pub guidItem: GUID, pub hBalloonIcon: HICON,\n"
            "}\n"
            "pub type PNOTIFYICONDATAW = *mut NOTIFYICONDATAW;\n"
            "pub const NOTIFYICONDATAW_V3_SIZE: DWORD =\n"
            "    core::mem::offset_of!(NOTIFYICONDATAW, hBalloonIcon) as DWORD;"
        )

    if "ASSERT" in used_words:
        chunks.append("macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }")

    if "CUserNotification::WorkerWndProc" in source:
        chunks.append(
            'unsafe extern "system" fn CUserNotification_WorkerWndProc(\n'
            "    window: HWND, message: UINT, wparam: WPARAM, lparam: LPARAM,\n"
            ") -> LRESULT {\n"
            "    let this = GetWindowLongPtrW(window, 0) as *mut CUserNotification;\n"
            "    CUserNotification_WorkerWndProc_body(\n"
            "        this, window, message, wparam, lparam,\n"
            "    )\n"
            "}"
        )

    if "DWORDLONG" in used_words:
        chunks.append("pub type DWORDLONG = u64;")
    if "SVSIF" in used_words:
        chunks.append("pub type SVSIF = UINT;")
    if "LPTBBUTTONSB" in used_words:
        chunks.append("pub type LPTBBUTTONSB = *mut core::ffi::c_void;")
    if "LOWORD" in used_words:
        chunks.append(
            "macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }"
        )
    if "MAKEINTRESOURCE" in used_words:
        chunks.append(
            "#[inline]\nconst fn MAKEINTRESOURCE(value: UINT) -> LPCWSTR {\n"
            "    value as usize as LPCWSTR\n}"
        )
    if "EXPPS" in used_words:
        chunks.append("pub type EXPPS = UINT;")
    if "LPFNSVADDPROPSHEETPAGE" in used_words:
        chunks.append(
            'pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn('
            "HPROPSHEETPAGE, LPARAM) -> BOOL>;"
        )
    if "SHQUERYRBINFO" in used_words:
        chunks.append(
            "#[repr(C)]\n"
            "pub struct SHQUERYRBINFO {\n"
            "    pub cbSize: DWORD,\n"
            "    pub i64Size: i64,\n"
            "    pub i64NumItems: i64,\n"
            "}"
        )
    if "SHELLEXECUTEINFOW" in used_words:
        chunks.append(
            "#[repr(C)]\n"
            "pub struct SHELLEXECUTEINFOW {\n"
            "    pub cbSize: DWORD, pub fMask: ULONG, pub hwnd: HWND,\n"
            "    pub lpVerb: LPCWSTR, pub lpFile: LPCWSTR, pub lpParameters: LPCWSTR,\n"
            "    pub lpDirectory: LPCWSTR, pub nShow: INT, pub hInstApp: HINSTANCE,\n"
            "    pub lpIDList: LPVOID, pub lpClass: LPCWSTR, pub hkeyClass: HKEY,\n"
            "    pub dwHotKey: DWORD, pub icon_or_monitor: HANDLE, pub hProcess: HANDLE,\n"
            "}"
        )

    table = _render_cmverbmap(source)
    if table:
        chunks.append(table)
    column_tables = _render_shvheader_tables(source, used_words)
    if column_tables:
        chunks.append(column_tables)

    if "CComHeapPtr" in used_words:
        used_words.add("CoTaskMemFree")
    if "SENDTO_ITEM" in used_words:
        used_words.update(("CoTaskMemFree", "DestroyIcon"))
    if "CStringW" in used_words:
        used_words.update(("lstrlenW", "memcpy", "StringCbPrintfW"))
    if "StringCchCopyW" in used_words:
        used_words.update(("lstrlenW", "memcpy"))
    if "SHSetStrRet" in used_words:
        used_words.update(
            (
                "CoTaskMemAlloc",
                "LoadStringW",
                "lstrlenW",
                "memcpy",
                "MAX_PATH",
                "shell32_hInstance",
            )
        )
    if "CopyMemory" in used_words or "ZeroMemory" in used_words:
        used_words.add("memcpy")
    if "ITEMIDLIST_ABSOLUTE" in used_words:
        chunks.append("pub type ITEMIDLIST_ABSOLUTE = ITEMIDLIST;")
    if "ITEMID_CHILD" in used_words:
        chunks.append("pub type ITEMID_CHILD = ITEMIDLIST;")
    drop_effects = {
        "DROPEFFECT_NONE": 0,
        "DROPEFFECT_COPY": 1,
        "DROPEFFECT_MOVE": 2,
        "DROPEFFECT_LINK": 4,
    }
    selected_effects = [
        f"pub const {name}: DWORD = {value};"
        for name, value in drop_effects.items()
        if name in used_words
    ]
    if selected_effects:
        chunks.append("\n".join(selected_effects))
    selected = [decl for name, decl in _CPP_SUPPORT_EXTERNS.items() if name in used_words]
    if selected:
        chunks.append('extern "system" {\n' + "\n".join(selected) + "\n}")
    selected_c = [decl for name, decl in _CPP_SUPPORT_C_EXTERNS.items() if name in used_words]
    if selected_c:
        chunks.append('extern "C" {\n' + "\n".join(selected_c) + "\n}")
    if "StringCbPrintfW" in used_words:
        chunks.append(
            "unsafe fn cpp_com_string_cb_printf_w(\n"
            "    dest: LPWSTR, bytes: usize, format: LPCWSTR, args: &[LPCWSTR],\n"
            ") -> HRESULT {\n"
            "    let capacity = bytes / core::mem::size_of::<WCHAR>();\n"
            "    if dest.is_null() || format.is_null() || capacity == 0 { return E_FAIL; }\n"
            "    let (mut out, mut input, mut arg_index) = (0usize, 0usize, 0usize);\n"
            "    while *format.add(input) != 0 {\n"
            "        let unit = *format.add(input);\n"
            "        if unit == b'%' as WCHAR && *format.add(input + 1) == b's' as WCHAR {\n"
            "            if arg_index >= args.len() { return E_FAIL; }\n"
            "            let arg = args[arg_index]; arg_index += 1; input += 2;\n"
            "            if arg.is_null() { continue; }\n"
            "            let mut index = 0usize;\n"
            "            while *arg.add(index) != 0 {\n"
            "                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; "
            "return 0x8007007Au32 as HRESULT; }\n"
            "                *dest.add(out) = *arg.add(index); out += 1; index += 1;\n"
            "            }\n"
            "            continue;\n"
            "        }\n"
            "        if out + 1 >= capacity { *dest.add(capacity - 1) = 0; "
            "return 0x8007007Au32 as HRESULT; }\n"
            "        *dest.add(out) = unit; out += 1; input += 1;\n"
            "    }\n"
            "    *dest.add(out) = 0; S_OK\n"
            "}\n"
            "macro_rules! StringCbPrintfW {\n"
            "    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr $(,)?) => {\n"
            "        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1])\n"
            "    };\n"
            "    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr, $arg2:expr $(,)?) => {\n"
            "        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1, $arg2])\n"
            "    };\n"
            "}"
        )
    if "memcpy" in used_words:
        chunks.append(
            "#[inline]\nunsafe fn memcpy<D, S, N: CppComSize>("
            "dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {\n"
            "    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())\n}"
        )
    if "SHFree" in used_words:
        chunks.append(
            'extern "system" {\n    #[link_name = "SHFree"]\n'
            "    fn cpp_com_SHFree(memory: LPVOID);\n}\n"
            "#[inline]\nunsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }"
        )
    if "free" in used_words:
        chunks.append(
            'extern "C" {\n    #[link_name = "free"]\n'
            "    fn cpp_com_free(memory: *mut core::ffi::c_void);\n}\n"
            "#[inline]\nunsafe fn free<T>(memory: *mut T) { cpp_com_free(memory.cast()); }"
        )
    if "CComHeapPtr" in used_words:
        chunks.append(_CPP_COM_HEAP_PTR_SUPPORT)
    if "SENDTO_ITEM" in used_words:
        chunks.append(_CPP_SENDTO_ITEM_SUPPORT)
    # A GUID constant is REFERENCED, never called. The ppv header macros happen to
    # match the GUID spelling, and an extern static for one is a phantom: no such
    # symbol exists to link against. Deciding that positionally retires a denylist
    # that had to be remembered — it listed `IID_PPV_ARG` alone, so the day
    # `IID_NULL_PPV_ARG` first appeared it shipped exactly that phantom.
    # Measured over dll/ base/ sdk/include/ modules/: every `(IID|SID|CLSID)_*` name
    # followed by `(` outside a literal is a function-like macro (the ppv pair,
    # IID_PPV_ARGS, IID_GENERIC_CHECK_IID, IID_BS_LOOKUP_*). The only real IIDs that
    # look called — IID_IShellLinkA, IID_IAutoComplete — sit inside wine-test format
    # strings, which mask_code blanks before either set is built.
    called_words = set(re.findall(r"\b([A-Za-z_]\w*)\s*\(", _masked_cpp(selection_source)))
    guid_words = {
        word
        for word in used_words
        if word not in called_words and re.fullmatch(r"(?:IID|SID|CLSID)_[A-Za-z_]\w*", word)
    }
    for alias, target in _CPP_SUPPORT_GUID_ALIASES.items():
        if alias in used_words:
            used_words.add(target)
            guid_words.discard(alias)
            guid_words.add(target)
    selected_guids = sorted(
        ((used_words & _CPP_SUPPORT_GUIDS) | guid_words) - (exclude_guids or set()) - interface_iids
    )
    if selected_guids:
        chunks.append(
            'extern "C" {\n' + "\n".join(_guid_extern_decl(name) for name in selected_guids) + "\n}"
        )

    if "CopyMemory" in used_words or "ZeroMemory" in used_words:
        chunks.append(
            'extern "C" {\n'
            "    fn memset(dst: *mut core::ffi::c_void, value: INT,\n"
            "        bytes: usize) -> *mut core::ffi::c_void;\n"
            "}\n"
            "#[inline]\nunsafe fn CopyMemory<D, S, N: CppComSize>(\n"
            "    dst: *mut D, src: *const S, bytes: N) {\n"
            "    let _ = memcpy(dst, src, bytes);\n}\n"
            "#[inline]\nunsafe fn ZeroMemory<T, N: CppComSize>(dst: *mut T, bytes: N) {\n"
            "    let _ = memset(dst.cast(), 0, bytes.cpp_com_size());\n}"
        )

    if "StringCchCopyW" in used_words:
        chunks.append(
            "#[inline]\n"
            "unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {\n"
            "    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }\n"
            "    let len = lstrlenW(src) as usize;\n"
            "    if len >= capacity { *dst = 0; return E_FAIL; }\n"
            "    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();\n"
            "    let _ = memcpy(dst, src, bytes);\n"
            "    S_OK\n"
            "}"
        )
    if "SHSetStrRet" in used_words:
        chunks.append(
            "unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {\n"
            "    let len = lstrlenW(value) as usize;\n"
            "    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();\n"
            "    let buffer = CoTaskMemAlloc(bytes) as LPWSTR;\n"
            "    if buffer.is_null() { return E_OUTOFMEMORY; }\n"
            "    let _ = memcpy(buffer, value, bytes);\n"
            "    (*out).uType = STRRET_WSTR; (*out).u.pOleStr = buffer; S_OK\n"
            "}\n"
            "unsafe fn cpp_com_SHSetStrRetResource(out: *mut STRRET, id: DWORD) -> HRESULT {\n"
            "    let mut buffer = [0u16; MAX_PATH];\n"
            "    if LoadStringW(shell32_hInstance, id, buffer.as_mut_ptr(), MAX_PATH as INT) == 0 "
            "{ return E_FAIL; }\n"
            "    cpp_com_SHSetStrRetW(out, buffer.as_ptr())\n"
            "}"
        )
    if "CStringW" in used_words:
        chunks.append(_CPP_CSTRINGW_SUPPORT)
    if "CComPtr" in used_words:
        chunks.append(_CPP_COMPTR_GUARD_SUPPORT)
    if "shell32_hInstance" in used_words:
        chunks.append('extern "C" {\n    static mut shell32_hInstance: HINSTANCE;\n}')
    return _drop_duplicate_type_definitions("\n\n".join(chunks))


_DUPLICATE_TYPE_RE = re.compile(
    r"^(?:#\[repr\([^\]]*\)\]\n)?pub (?:struct|union|enum) (?P<name>[A-Za-z_]\w*)\b",
)


def _drop_duplicate_type_definitions(rendered: str) -> str:
    """Emit each top-level type ONCE, keeping the first rendering.

    This closure is assembled from several independent passes — local PODs, the SDK layouts,
    and the referenced-interface walk — and none of them can see what the others already
    emitted. When two passes both reach a type they both render it, and a repeated
    `pub struct` is E0428 at the definition, not a shadow: `DISPPARAMS` came out twice in
    every one of CFolder / CFolderItem / CFolderItems, from the POD pass and again behind the
    `IShellDispatch` interface walk.

    Keeping the FIRST is what makes this safe to do blindly. The passes are ordered
    most-specific first, so a later pass can only ever be re-deriving the same SDK layout; if
    two renderings ever genuinely DISAGREED, dropping the later one would hide that, so the
    duplicate is only dropped when the blocks are byte-identical and left alone otherwise —
    two different layouts under one name is a real conflict and must stay loud.
    """
    blocks = rendered.split("\n\n")
    first_by_name: dict[str, str] = {}
    kept: list[str] = []
    for block in blocks:
        match = _DUPLICATE_TYPE_RE.match(block)
        if match is None:
            kept.append(block)
            continue
        name = match.group("name")
        previous = first_by_name.get(name)
        if previous is None:
            first_by_name[name] = block
            kept.append(block)
        elif previous != block:
            # Same name, different layout — a real conflict. Keep both so it fails loudly
            # at the definition rather than silently picking one.
            kept.append(block)
    return "\n\n".join(kept)


_CPP_COMPTR_GUARD_SUPPORT = r"""pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self { Self { slot } }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}"""


_CPP_COM_HEAP_PTR_SUPPORT = r"""#[repr(transparent)]
pub struct CComHeapPtr<T> { ptr: *mut T }
impl<T> CComHeapPtr<T> {
    pub const fn empty() -> Self { Self { ptr: core::ptr::null_mut() } }
    pub const fn new(ptr: *mut T) -> Self { Self { ptr } }
    pub const fn as_raw(&self) -> *mut T { self.ptr }
    pub fn as_mut_slot(&mut self) -> *mut *mut T { core::ptr::addr_of_mut!(self.ptr) }
    pub unsafe fn Attach(&mut self, ptr: *mut T) {
        if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); }
        self.ptr = ptr;
    }
    pub fn Detach(&mut self) -> *mut T {
        let ptr = self.ptr; self.ptr = core::ptr::null_mut(); ptr
    }
}
impl<T> Drop for CComHeapPtr<T> {
    fn drop(&mut self) {
        unsafe { if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); } }
    }
}"""


_CPP_SENDTO_ITEM_SUPPORT = r"""#[repr(C)]
pub struct SENDTO_ITEM {
    pub pNext: *mut SENDTO_ITEM,
    pub pidlChild: PITEMID_CHILD,
    pub pszText: LPWSTR,
    pub hIcon: HICON,
}
unsafe fn SENDTO_ITEM_new(
    child: PITEMID_CHILD, text: LPWSTR, icon: HICON,
) -> *mut SENDTO_ITEM {
    let item = HeapAlloc(
        GetProcessHeap(), 0, core::mem::size_of::<SENDTO_ITEM>(),
    ) as *mut SENDTO_ITEM;
    if !item.is_null() {
        item.write(SENDTO_ITEM {
            pNext: core::ptr::null_mut(), pidlChild: child, pszText: text, hIcon: icon,
        });
    }
    item
}
unsafe fn SENDTO_ITEM_delete(item: *mut SENDTO_ITEM) {
    if item.is_null() { return; }
    CoTaskMemFree((*item).pidlChild.cast());
    CoTaskMemFree((*item).pszText.cast());
    if !(*item).hIcon.is_null() { DestroyIcon((*item).hIcon); }
    HeapFree(GetProcessHeap(), 0, item.cast::<core::ffi::c_void>());
}"""


_CPP_CSTRINGW_SUPPORT = r"""extern "system" {
    fn HeapReAlloc(heap: HANDLE, flags: DWORD, memory: LPVOID, bytes: usize) -> LPVOID;
    fn GetModuleHandleW(name: LPCWSTR) -> HINSTANCE;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
}

pub struct CStringW { buffer: LPWSTR, len: usize, capacity: usize }
impl CStringW {
    pub const fn new() -> Self { Self { buffer: core::ptr::null_mut(), len: 0, capacity: 0 } }
    pub unsafe fn from_resource(id: UINT) -> Self {
        let mut value = Self::new();
        let mut stack = [0u16; 256];
        let count = LoadStringW(
            GetModuleHandleW(core::ptr::null()), id, stack.as_mut_ptr(), stack.len() as INT,
        );
        if count > 0 { value.append_ptr_len(stack.as_ptr(), count as usize); }
        value
    }
    pub unsafe fn new_from_lpcwstr(text: LPCWSTR) -> Self {
        if (text as usize) <= 0xffff { return Self::from_resource(text as usize as UINT); }
        let mut value = Self::new(); value.append_ptr(text); value
    }
    pub fn as_ptr(&self) -> LPCWSTR {
        if self.buffer.is_null() { CPP_COM_EMPTY_WSTR.as_ptr() } else { self.buffer }
    }
    pub fn as_mut_ptr(&mut self) -> LPWSTR { self.as_ptr() as LPWSTR }
    unsafe fn reserve(&mut self, needed: usize) -> bool {
        if needed <= self.capacity { return true; }
        let capacity = needed.next_power_of_two().max(16);
        let bytes = capacity * core::mem::size_of::<WCHAR>();
        let next = if self.buffer.is_null() {
            HeapAlloc(GetProcessHeap(), 0, bytes)
        } else {
            HeapReAlloc(GetProcessHeap(), 0, self.buffer.cast(), bytes)
        } as LPWSTR;
        if next.is_null() { return false; }
        self.buffer = next; self.capacity = capacity; true
    }
    unsafe fn append_ptr_len(&mut self, text: LPCWSTR, count: usize) {
        if text.is_null() || !self.reserve(self.len + count + 1) { return; }
        let _ = memcpy(
            self.buffer.add(self.len), text,
            count * core::mem::size_of::<WCHAR>(),
        );
        self.len += count; *self.buffer.add(self.len) = 0;
    }
    unsafe fn append_ptr(&mut self, text: LPCWSTR) {
        if !text.is_null() { self.append_ptr_len(text, lstrlenW(text) as usize); }
    }
    unsafe fn append_char(&mut self, ch: WCHAR) { self.append_ptr_len(core::ptr::addr_of!(ch), 1); }
    pub unsafe fn concat(&mut self, text: LPCWSTR) { self.append_ptr(text); }
    pub unsafe fn assign(&mut self, text: LPCWSTR) {
        self.len = 0;
        if !self.buffer.is_null() { *self.buffer = 0; }
        self.append_ptr(text);
    }
    pub unsafe fn format(&mut self, format: LPCWSTR, arg: LPCWSTR) {
        let mut resource = [0u16; 256];
        let format = if (format as usize) <= 0xffff {
            LoadStringW(GetModuleHandleW(core::ptr::null()), format as usize as UINT,
                resource.as_mut_ptr(), resource.len() as INT);
            resource.as_ptr()
        } else { format };
        self.len = 0;
        if self.reserve(512) {
            let _ = StringCbPrintfW!(self.buffer, self.capacity * 2, format, arg);
            self.len = lstrlenW(self.buffer) as usize;
        }
    }
    pub unsafe fn format_message(&mut self, arg: LPCWSTR) {
        let mut format = [0u16; 512];
        let count = lstrlenW(self.as_ptr()).max(0) as usize;
        let count = count.min(format.len() - 1);
        let _ = memcpy(format.as_mut_ptr(), self.as_ptr(), count * 2);
        self.format(format.as_ptr(), arg);
    }
}
impl core::ops::AddAssign<LPCWSTR> for CStringW {
    fn add_assign(&mut self, rhs: LPCWSTR) { unsafe { self.append_ptr(rhs) } }
}
impl core::ops::AddAssign<WCHAR> for CStringW {
    fn add_assign(&mut self, rhs: WCHAR) { unsafe { self.append_char(rhs) } }
}
impl Drop for CStringW {
    fn drop(&mut self) {
        unsafe {
            if !self.buffer.is_null() { HeapFree(GetProcessHeap(), 0, self.buffer); }
        }
    }
}
static CPP_COM_EMPTY_WSTR: [WCHAR; 1] = [0];"""


# `r#` makes a keyword usable as an identifier, but the language forbids it for these
# four -- there is no spelling of a parameter named `self`, so the name must change.
_RUST_UNRAWABLE = frozenset({"crate", "self", "super", "Self"})


def _rust_safe_field(name: str) -> str:
    """Spell a C struct FIELD whose name is a Rust keyword, on bindgen's `keyword_` convention.

    NOT `r#keyword`, even though that is legal. `postprocess.fix_keyword_field_names` rewrites
    every field ACCESS (`.type`, `.r#type`, `.type_0`) onto `.type_`, matching the bindgen
    output this tree ships (`pub type_: DWORD`). Emitting the DECLARATION as `r#type` therefore
    disagreed with every use of it: CMenuFocusManager's `StackEntry` declared `pub r#type` while
    all 34 accesses read `.type_`, and the two passes have to agree on one spelling.

    Parameters are different and keep `rust_safe_parameter`'s `r#` — nothing rewrites those.
    """
    return f"{name}_" if name in _RUST_KEYWORDS else name


def rust_safe_parameter(name: str) -> str:
    """Spell a C parameter name so it is a legal Rust identifier.

    A C++ parameter may be named `move`, `type`, `ref`, `match`, `box` -- all Rust
    keywords. Rendering one bare produced a contract signature that CANNOT COMPILE, and
    the deterministic audit then rejected the model for writing `r#move`, the only legal
    form. shell32's `CDefView::SetClipboard(BOOL move)` failed exactly this way, and no
    number of retries could have fixed it: the contract itself was unsatisfiable.
    """
    if name in _RUST_UNRAWABLE:
        return f"{name}_"
    return f"r#{name}" if name in _RUST_KEYWORDS else name


def worker_signature(
    name: str, class_name: str | None, params: list[tuple[str, str]], return_type: str
) -> str:
    """Render the deterministic body-worker signature.

    COM methods receive the recovered object pointer first. Same-file helper
    functions are ordinary Rust workers with signatures derived directly from
    their C definitions; the model owns neither surface.
    """
    rendered = [f"this: *mut {class_name}"] if class_name else []
    for i, (c_type, param_name) in enumerate(params):
        # The generic scanner intentionally preserves declarator punctuation,
        # but for C++ references clang-style source often attaches `&`/`*` to
        # the name (`CStringW`, `&paths`). Move it back to the type before the
        # deterministic Rust mapping.
        declarator = param_name or f"arg{i}"
        while declarator.startswith(("&", "*")):
            c_type = f"{c_type} {declarator[0]}"
            declarator = declarator[1:]
        rendered.append(f"{rust_safe_parameter(declarator)}: {c_type_to_rust(c_type)}")
    ret = "()" if not return_type or return_type.strip() == "void" else c_type_to_rust(return_type)
    return f"pub unsafe fn {name}_body({', '.join(rendered)}) -> {ret}"


# Shared with call_seal/fact_seal. The private copy this replaced was a C lexer
# applied to Rust in four places (worker-body normalization), where a `'cleanup:`
# label or a `&'a` lifetime blanked everything to the next apostrophe. See lex_mask.


def strip_debug_statements(text: str, names: set[str] | None = None) -> str:
    """Remove complete shell32 debug-macro statements from a model-only view."""
    masked = _masked_cpp(text)
    spans = []
    debug_names = "|".join(re.escape(name) for name in sorted(names or _CPP_COM_DEBUG_MACROS))
    for match in re.finditer(rf"\b(?:{debug_names})\s*!?\s*\(", masked):
        before = masked[: match.start()].rstrip()
        if before and before[-1] not in "{;}":
            continue
        opening = masked.find("(", match.start())
        try:
            closing = _matching_paren(masked, opening)
        except CppComError:
            continue
        end = closing + 1
        while end < len(masked) and masked[end].isspace():
            end += 1
        if end < len(masked) and masked[end] == ";":
            spans.append((match.start(), end + 1))
    for start, end in reversed(spans):
        text = text[:start] + " " * (end - start) + text[end:]
    return text


def _rewrite_com_ptr_state(
    body: str, class_name: str, state_fields: list[PlainStateField]
) -> tuple[str, set[str]]:
    """Make CComPtr assignment/refcount behavior explicit before model translation."""
    callees = set()
    for state in state_fields:
        helper = state.assignment_helper
        if helper is None:
            continue
        qualified_helper = f"{class_name}_{helper}"
        # CComPtr::operator=(T*) is the only accepted mutation shape. Compound,
        # address-taking, Attach/Detach, and member calls remain untouched and
        # therefore fail later audits/compile rather than losing ownership.
        assignment = re.compile(
            rf"(?<![\w.>])(?:this\s*->\s*)?{re.escape(state.name)}\s*=\s*([^;]+);"
        )
        body, count = assignment.subn(rf"{qualified_helper}(this, \1);", body)
        if count:
            callees.add(qualified_helper)
        body = re.sub(rf"(?<![\w.>]){re.escape(state.name)}\b", f"this->{state.name}", body)
    return body, callees


def _rewrite_owned_string_state(
    body: str, class_name: str, state_fields: list[PlainStateField]
) -> tuple[str, set[str]]:
    """Make CStringW replacement assignment explicit before model translation."""
    callees = set()
    for state in state_fields:
        helper = state.string_assignment_helper
        if helper is None:
            continue
        qualified_helper = f"{class_name}_{helper}"
        assignment = re.compile(
            rf"(?<![\w.>])(?:this\s*->\s*)?{re.escape(state.name)}\s*=\s*([^;]+);"
        )
        body, count = assignment.subn(rf"{qualified_helper}(this, \1);", body)
        if count:
            callees.add(qualified_helper)
        body = re.sub(rf"(?<![\w.>]){re.escape(state.name)}\b", f"this->{state.name}", body)
    return body, callees


def prepare_helper_closure(
    deps: list[FunctionDep],
    class_name: str,
    state_fields: list[PlainStateField] | None = None,
    field_types: dict[str, dict[str, str]] | None = None,
    type_aliases: dict[str, str] | None = None,
) -> list[FunctionDep]:
    """Select and bind the exact same-file helper closure for one COM class.

    C++ overload resolution is deliberately not reimplemented. Calls are bound
    from scanner-owned parameter/local types and must have one uniquely best
    candidate. Every selected call is rewritten to its collision-free
    ``<target>_body`` Rust name before the model sees it. Ambiguity hard-fails
    before payment.
    """
    from .callgraph import FunctionDep, topological_sort

    class_deps = [dep for dep in deps if dep.cpp_class == class_name]
    if not class_deps:
        raise CppComError(f"{class_name}: no extracted method bodies")
    names = {dep.name for dep in class_deps}
    if f"{class_name}_ctor" not in names:
        class_deps.append(
            FunctionDep(
                name=f"{class_name}_ctor",
                cpp_class=class_name,
                cpp_method=class_name,
                body_text=f"{class_name}::{class_name}() {{}}",
            )
        )
    if f"{class_name}_dtor" not in names:
        class_deps.append(
            FunctionDep(
                name=f"{class_name}_dtor",
                cpp_class=class_name,
                cpp_method=f"~{class_name}",
                body_text=f"{class_name}::~{class_name}() {{}}",
            )
        )

    # Only internal-linkage functions belong to a class's body closure. An
    # external-linkage function in the same translation unit is still a module
    # ABI boundary (for example SHCreateShellItem); absorbing it would change
    # ownership, exports, and possibly activation semantics.
    free_deps = [dep for dep in deps if dep.cpp_class is None and dep.is_static]
    by_source: dict[str, list[FunctionDep]] = {}
    for dep in free_deps:
        # ``name`` is the scanner's stable target name. Overload suffixes are
        # assigned in declaration order; recover the spelling from the source
        # declarator instead of guessing by stripping ``_2``.
        match = re.search(r"\b([A-Za-z_]\w*)\s*\(", _masked_cpp(dep.body_text))
        source_name = match.group(1) if match else dep.name
        by_source.setdefault(source_name, []).append(dep)

    expanded = _expand_default_helper_arguments(class_deps + free_deps, by_source)
    class_deps = [dep for dep in expanded if dep.cpp_class == class_name]
    free_deps = [dep for dep in expanded if dep.cpp_class is None]
    by_source = {}
    for dep in free_deps:
        match = re.search(r"\b([A-Za-z_]\w*)\s*\(", _masked_cpp(dep.body_text))
        source_name = match.group(1) if match else dep.name
        by_source.setdefault(source_name, []).append(dep)

    helper_returns: dict[str, str] = {}
    for source_name, candidates in by_source.items():
        return_types = {
            _abi_c_type_key(candidate.return_type)
            for candidate in candidates
            if candidate.return_type
        }
        if len(return_types) == 1:
            helper_returns[source_name] = candidates[0].return_type
        for candidate in candidates:
            if candidate.return_type:
                helper_returns[f"{candidate.name}_body"] = candidate.return_type

    binding: dict[tuple[str, int], list[FunctionDep]] = {}
    for source_name, candidates in by_source.items():
        for dep in candidates:
            key = (source_name, len(dep.params))
            binding.setdefault(key, []).append(dep)

    helper_names = set(by_source)
    method_targets: dict[str, list[FunctionDep]] = {}
    for dep in class_deps:
        method = dep.cpp_method
        if method in (None, class_name, f"~{class_name}"):
            continue
        method_targets.setdefault(method, []).append(dep)
    method_names = set(method_targets)
    selected: dict[str, FunctionDep] = {}
    queue = list(class_deps)
    while queue:
        caller = queue.pop(0)
        definition_brace = caller.body_text.find("{")
        for start, end, source_name, arity in _named_call_spans(caller.body_text, helper_names):
            candidates = binding.get((source_name, arity), [])
            if not candidates:
                # A free helper and a member function may share a name; C++ resolves an
                # unqualified call by scope, and the member wins inside a method body.
                # CDefaultContextMenu has exactly this: a static MapVerbToCmdId/5 and a
                # method MapVerbToCmdId/3, the free one called as `::MapVerbToCmdId`.
                # The method is already in class_deps, so reachability is unaffected —
                # and the REWRITE pass below already skips these to let the method pass
                # take them. Only this walk refused, which is the asymmetry, not a gap.
                if any(
                    len(candidate.params) == arity
                    for candidate in method_targets.get(source_name, ())
                ):
                    continue
                available = sorted({n for name, n in binding if name == source_name})
                raise CppComError(
                    f"{class_name}: cannot bind local helper call {source_name}/{arity}; "
                    f"available arities: {available}"
                )
            target = (
                caller
                if caller in candidates and start < definition_brace
                else _bind_sibling_overload(
                    caller.body_text,
                    start,
                    end,
                    candidates,
                    caller,
                    state_fields or [],
                    class_name,
                    source_name,
                    helper_returns,
                    field_types,
                    type_aliases,
                )
            )
            if target.name not in selected:
                selected[target.name] = target
                queue.append(target)

    plan = list(selected.values()) + class_deps
    rewritten: list[FunctionDep] = []
    for dep in plan:
        body = _rewrite_wide_literals(dep.body_text)
        body, state_callees = _rewrite_com_ptr_state(body, class_name, state_fields or [])
        body, string_state_callees = _rewrite_owned_string_state(
            body, class_name, state_fields or []
        )
        state_callees |= string_state_callees
        local_targets: set[str] = set()
        spans = _named_call_spans(body, helper_names)
        definition_brace = body.find("{")
        for start, end, source_name, arity in reversed(spans):
            candidates = binding.get((source_name, arity), [])
            if not candidates:
                continue
            target = (
                dep
                if dep in candidates and start < definition_brace
                else _bind_sibling_overload(
                    body,
                    start,
                    end,
                    candidates,
                    dep,
                    state_fields or [],
                    class_name,
                    source_name,
                    helper_returns,
                    field_types,
                    type_aliases,
                )
            )
            body = body[:start] + f"{target.name}_body" + body[end:]
            # The helper definition's own declarator is not a callee. Removing
            # self is harmless for genuine recursion; _format_callee_context
            # intentionally omits it too.
            if target.name != dep.name:
                local_targets.add(f"{target.name}_body")
        method_spans = _named_call_spans(body, method_names)
        definition_brace = body.find("{")
        for start, end, source_name, _arity in reversed(method_spans):
            receiver = re.search(r"([A-Za-z_]\w*)\s*(?:->|\.)\s*$", body[:start])
            if receiver is not None and receiver.group(1) != "this":
                # Calls through another COM object are not implicit sibling
                # calls merely because their method spelling collides.
                continue
            if body[:start].rstrip().endswith("::"):
                # `::f(...)` names the GLOBAL f explicitly — that is how the source
                # disambiguates a free function from a member of the same name, so it
                # is never a sibling call however well the arity happens to line up.
                continue
            # Keep this worker's C++ definition declarator recognizable; only
            # sibling calls become deterministic Rust worker calls.
            if source_name == dep.cpp_method and start < definition_brace:
                continue
            all_candidates = method_targets[source_name]
            candidates = [
                candidate for candidate in all_candidates if len(candidate.params) == _arity
            ]
            # IID_PPV_ARG and similar source macros expand one apparent C++
            # argument into multiple ABI arguments later in normalization.
            # A non-overloaded method remains unambiguous despite the lexical
            # arity mismatch.
            if not candidates and len(all_candidates) == 1:
                candidates = all_candidates
            target = _bind_sibling_overload(
                body,
                start,
                end,
                candidates,
                dep,
                state_fields or [],
                class_name,
                source_name,
                helper_returns,
                field_types,
                type_aliases,
            )
            body = body[:start] + f"{target.name}_body" + body[end:]
            if target.name != dep.name:
                local_targets.add(f"{target.name}_body")
        external = {
            name for name in dep.callees if name not in helper_names and name not in method_names
        }
        rewritten.append(
            replace(dep, body_text=body, callees=external | local_targets | state_callees)
        )

    return topological_sort(rewritten)


# File-local C++ RAII structs known to appear as body locals in shell32 sources.
# Each is a hard pre-payment reject unless the invocation supplies a hand-verified
# Rust lowering via --local-raii NAME=path.rs (see provided_raii_contract).
# Body-local C++ owners with no deterministic Rust lowering yet. Being ABSENT from
# this set is not neutral: the model is then free to invent one, and it does —
# `CDataObjectHIDA hida(pDO);` came back as `CDataObjectHIDA::new(pDO)`, a Rust type
# that exists nowhere, in a run that reported success and charged for it. Listing a
# type converts that into a refusal BEFORE payment that names `--local-raii`.
# Measured cost of adding CDataObjectHIDA: 6 shell32 TUs refuse until lowered.
_LOCAL_RAII_TYPES = frozenset({"CDrop", "CDataObjectHIDA"})


def require_supported_worker_raii(deps: list, provided: frozenset | set = frozenset()) -> None:
    """Reject body-local C++ owners whose lifetime is not deterministic yet.

    A type in ``provided`` has a hand-verified Rust lowering supplied by the
    caller (--local-raii); its usage is then governed by the sealed contract
    from :func:`provided_raii_contract` instead of being rejected.
    """
    unsupported = []
    for dep in deps:
        for raii_type in sorted(_LOCAL_RAII_TYPES - set(provided)):
            if re.search(rf"\b{raii_type}\s+[A-Za-z_]\w*\s*\(", dep.body_text):
                unsupported.append(f"{dep.name}: {raii_type} local")
    if unsupported:
        raise CppComError(
            "unsupported body-local C++ RAII requires deterministic lowering before payment: "
            + ", ".join(unsupported)
            + " (supply a hand-verified Rust lowering via --local-raii NAME=path.rs)"
        )


def provided_raii_contract(name: str, rust_text: str) -> str:
    """Deterministic-contract block for one hand-provided local RAII lowering.

    The Rust definition is authored and verified by a human, spliced verbatim
    into the assembled module, and handed to every worker whose C++ body uses
    the type — so the model maps usage onto a fixed API instead of inventing
    drop semantics.
    """
    return (
        f"\n## Provided local RAII type: {name} (deterministic — already lowered)\n"
        f"The C++ file-local RAII struct `{name}` has a hand-verified Rust lowering that "
        "WILL be present in this module. Use it exactly as declared below. Do NOT "
        "re-implement, redefine, modify, or inline its logic.\n"
        f"```rust\n{rust_text.rstrip()}\n```\n"
        "Usage mapping (mandatory):\n"
        f"- C++ `{name} var(args...);` → Rust `let mut var = {name}::new(args...);`\n"
        "- The C++ destructor is `impl Drop`: it runs automatically at scope exit on "
        "every path (including each early `return`). Do NOT call cleanup manually and "
        "do NOT call `core::mem::drop` unless the C++ scope ends there.\n"
        "- Field reads/writes and method calls map 1:1 with the same spelling.\n"
    )


def raii_container_contract(fields: list[PlainStateField]) -> str:
    """Deterministic-contract block for the class's ATL container STATE fields.

    The counterpart to :func:`provided_raii_contract`, which covers body-LOCALS. A state
    field's container type survives into the Rust owner struct, so the model needs the
    same thing it needs for a local: a fixed API to map usage onto, and an explicit
    instruction not to re-implement the destructor. Without it, `m_Storage[n]` and
    `m_Entries.GetNext(it)` are exactly the kind of C++ spelling a model resolves by
    inventing a wrapper type.
    """
    containers = [
        (state, state.raii_container) for state in fields if state.raii_container is not None
    ]
    if not containers:
        return ""
    lines = [
        "\n## Object state: ATL containers (deterministic — already lowered)\n",
        "These fields keep their ATL container type in the Rust owner. The Rust types are "
        "in `crate::atl` and WILL be in scope. Use them exactly as declared; do NOT "
        "re-implement, redefine, or inline their logic, and do NOT write a destructor for "
        "them — `Drop` already runs the C++ member destructor at object teardown.\n",
    ]
    for state, container in containers:
        _init, usage = _RAII_CONTAINER_LOWERINGS[container.split("<", 1)[0]]
        lines.append(f"- `(*this).{state.name}: {container}` — {usage}")
    lines.append(
        "\nEvery method above takes `&mut self` where it mutates, so a field access is "
        "`(*this).<field>.<Method>(...)`. The accessors return RAW POINTERS where C++ "
        "returns a reference: `*mut T` is the address of the element in place, so C++ "
        "`info.field` on such a reference is `(*p).field` in Rust, and writing through it "
        "mutates the container's storage exactly as the C++ does."
    )
    return "\n".join(lines) + "\n"


def apply_interface_method_signatures(
    deps: list[FunctionDep],
    class_name: str,
    interfaces: list[InterfaceSpec],
    type_aliases: dict[str, str] | None = None,
) -> list[FunctionDep]:
    """Replace parsed COM method types with the flattened vtable authority.

    C++ error recovery can find an out-of-line ``WINAPI Class::Method`` body
    while losing its return type. WIDL already defines the exact ABI shape, so
    scanner types must never win for interface methods. Source parameter names
    are retained because the C++ body refers to them.
    """
    methods: dict[str, list[InterfaceMethod]] = {}
    for interface in interfaces:
        for method in interface.methods:
            if method.name in ("QueryInterface", "AddRef", "Release"):
                continue
            variants = methods.setdefault(method.name, [])
            shape = _interface_method_shape(method)
            if all(_interface_method_shape(prior) != shape for prior in variants):
                variants.append(method)

    out = []
    helper_counts: dict[str, int] = {}

    def bind_variant(
        candidate: FunctionDep, candidates: list[InterfaceMethod]
    ) -> list[tuple[int, InterfaceMethod]]:
        source_shape = tuple(_abi_c_type_key(c_type) for c_type, _ in candidate.params)
        matches = [
            (index, method)
            for index, method in enumerate(candidates)
            if tuple(_abi_c_type_key(c_type) for c_type, _ in method.params[1:]) == source_shape
        ]
        if len(matches) != 1:
            # The arity fallback is load-bearing: the scanner and the vtable routinely
            # SPELL one parameter differently (`LPCITEMIDLIST` vs `PCUITEMID_CHILD`), so
            # an exact key match fails on methods that genuinely do implement the slot.
            #
            # Unconstrained, though, it also binds a private overload that merely happens
            # to share the arity -- and the caller then OVERWRITES that helper's parameter
            # list with the vtable's, so a helper taking an `int` index silently claims a
            # PIDL pointer. CDefView has exactly this pair (`SelectAndPositionItem(int,
            # UINT, POINT *)` beside the STDMETHOD of the same name). Nothing downstream
            # can see the swap, which is what makes it worth refusing here.
            #
            # So: keep the fallback, but never let it cross a conversion C++ does not have.
            same_arity = [
                (index, method)
                for index, method in enumerate(candidates)
                if len(method.params) - 1 == len(candidate.params)
                and not _shapes_provably_disagree(candidate.params, method.params[1:], type_aliases)
            ]
            if len(same_arity) == 1:
                matches = same_arity
        return matches

    for dep in deps:
        # A `static` class method (e.g. a `static inline GetClassID()` helper) has
        # no `this` and can never occupy a vtable slot, even when it shares a name
        # with a virtual method. It is a plain class-scoped helper bound by the
        # sibling/arity machinery, not against the WIDL vtable shape.
        variants = (
            methods.get(dep.cpp_method or "")
            if dep.cpp_class == class_name and not dep.is_static
            else None
        )
        if not variants:
            out.append(dep)
            continue
        matching = bind_variant(dep, variants)
        if len(matching) != 1:
            # A non-static private overload may share a spelling with a COM
            # method. It is a proved helper only when a sibling body uniquely
            # binds the WIDL slot; reserve the canonical worker name for that
            # sibling and give this helper a collision-free deterministic name.
            sibling_binds = any(
                other is not dep
                and other.cpp_class == class_name
                and other.cpp_method == dep.cpp_method
                and not other.is_static
                and len(bind_variant(other, variants)) == 1
                for other in deps
            )
            if sibling_binds:
                stem = f"{class_name}_{dep.cpp_method}_helper"
                helper_counts[stem] = helper_counts.get(stem, 0) + 1
                suffix = "" if helper_counts[stem] == 1 else f"_{helper_counts[stem]}"
                out.append(replace(dep, name=stem + suffix))
                continue
            choices = ", ".join(
                "(" + ", ".join(c_type for c_type, _ in method.params[1:]) + ")"
                for method in variants
            )
            raise CppComError(
                f"{class_name}::{dep.cpp_method}: cannot bind scanner signature "
                f"({', '.join(c_type for c_type, _ in dep.params)}) to vtable variants: "
                f"{choices}"
            )
        variant_index, method = matching[0]
        vtable_params = list(method.params[1:])
        if len(vtable_params) != len(dep.params):
            raise CppComError(
                f"{class_name}::{dep.cpp_method}: scanner found {len(dep.params)} parameter(s), "
                f"but the flattened vtable requires {len(vtable_params)}"
            )
        params = [
            (vtable_type, source_name or vtable_name)
            for (vtable_type, vtable_name), (_, source_name) in zip(
                vtable_params, dep.params, strict=True
            )
        ]
        target_name = f"{class_name}_{dep.cpp_method}"
        if len(variants) > 1:
            target_name += f"_v{variant_index + 1}"
        out.append(replace(dep, name=target_name, return_type=method.return_type, params=params))
    return out


def _interface_method_shape(method: InterfaceMethod) -> tuple[str, tuple[str, ...]]:
    """Canonical ABI shape excluding the per-interface receiver pointer."""
    return (
        _abi_c_type_key(method.return_type),
        tuple(_abi_c_type_key(c_type) for c_type, _ in method.params[1:]),
    )


def _interface_method_target(
    class_name: str, method: InterfaceMethod, interfaces: list[InterfaceSpec]
) -> str:
    variants: list[InterfaceMethod] = []
    for interface in interfaces:
        for candidate in interface.methods:
            if candidate.name != method.name or candidate.name in (
                "QueryInterface",
                "AddRef",
                "Release",
            ):
                continue
            if all(
                _interface_method_shape(prior) != _interface_method_shape(candidate)
                for prior in variants
            ):
                variants.append(candidate)
    target = f"{class_name}_{method.name}"
    if len(variants) > 1:
        shape = _interface_method_shape(method)
        variant = next(
            i for i, item in enumerate(variants, 1) if _interface_method_shape(item) == shape
        )
        target += f"_v{variant}"
    return target


def lower_trivial_worker(c_body: str, expected_signature: str) -> str | None:
    """Lower exact no-state stub bodies without spending an LLM call.

    This deliberately recognizes only the whole-body shapes it can prove:
    ``UNIMPLEMENTED;`` and ``UNIMPLEMENTED; return SYMBOLIC_HRESULT;``.
    Any additional statement stays on the audited model path.
    """
    open_brace, close_brace = c_body.find("{"), c_body.rfind("}")
    if open_brace < 0 or close_brace <= open_brace:
        return None
    # An empty constructor body is not necessarily empty behavior: its
    # initializer list runs first. The deterministic factory already provides
    # zeroed plain-C state, so only exact zero initializers may collapse to a
    # no-op. Non-zero/member-construction forms stay on the audited model path.
    head = re.sub(r"/\*.*?\*/|//[^\n]*", " ", c_body[:open_brace], flags=re.S)
    initializer_mark = re.search(r"\)\s*:\s*", head)
    if initializer_mark is not None:
        initializer = head[initializer_mark.end() :].strip()
        for item in _split_args(initializer):
            match = re.fullmatch(r"[A-Za-z_]\w*\s*\(\s*([^()]*)\s*\)", item)
            if match is None or match.group(1).strip() not in {"0", "NULL", "nullptr", "FALSE"}:
                return None
    body = c_body[open_brace + 1 : close_brace]
    # TRACE/FIXME/WARN/ERR/MESSAGE are no-op macro_rules in the shell32 Rust
    # substrate. Remove only complete call statements before proving the
    # remainder is one of the closed trivial shapes below.
    body = strip_debug_statements(body)
    body = re.sub(r"/\*.*?\*/|//[^\n]*", " ", body, flags=re.S)
    body = re.sub(r"\bUNIMPLEMENTED\s*;", " ", body)
    body = " ".join(body.split())
    if not body:
        return expected_signature + " {}"
    printer_enum = re.fullmatch(
        r"return\s+ShellObjectCreatorInit\s*<\s*CPrintersEnum\s*>\s*\(\s*"
        r"([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*,\s*"
        r"IID_PPV_ARG\s*\(\s*IEnumIDList\s*,\s*([A-Za-z_]\w*)\s*\)\s*\)\s*;",
        body,
    )
    if printer_enum is not None:
        owner, flags, output = printer_enum.groups()
        return (
            expected_signature + f" {{ CPrintersEnum_CreateInstance({owner}, {flags}, {output}) }}"
        )
    returned = re.fullmatch(r"return\s+([A-Z][A-Z0-9_]*)\s*;", body)
    if returned and returned.group(1) not in _CPP_COM_LITERAL_WORDS:
        return f"{expected_signature} {{ {returned.group(1)} }}"
    out_assignment = re.fullmatch(
        r"\*\s*([A-Za-z_]\w*)\s*=\s*([A-Z][A-Z0-9_]*)\s*;\s*"
        r"return\s+([A-Z][A-Z0-9_]*)\s*;",
        body,
    )
    if out_assignment:
        pointer, value, result = out_assignment.groups()
        # C's `NULL` is `((void*)0)` and assigns to a pointer of ANY pointee type; Rust's
        # `NULL` is a concrete `*mut c_void` and assigns to nothing else. Emitting it
        # verbatim made every `*ppOut = NULL; return E_NOTIMPL;` stub a type error —
        # `expected *mut Folder, found *mut c_void` on CFolder::get_ParentFolder.
        # `null_mut()` infers the out-parameter's own pointee instead.
        if value in {"NULL", "nullptr"}:
            value = "core::ptr::null_mut()"
        return f"{expected_signature} {{ *{pointer} = {value}; {result} }}"
    return None


def _canonical_signature(signature: str) -> str:
    signature = re.sub(r"\bcrate::(?:types|shellcom)::", "", signature)
    signature = re.sub(r"(?<=[(,])\s*mut\s+", " ", signature)
    signature = re.sub(r"\s+", " ", signature).strip()
    signature = re.sub(r"\(\s+", "(", signature)
    signature = re.sub(r",\s*\)", ")", signature)
    signature = re.sub(r"\s+\)", ")", signature)
    # An omitted unit return and an explicit `-> ()` are the same Rust signature.
    return re.sub(r"\s*->\s*\(\)\s*$", "", signature)


def _source_local_pod_fields(c_body: str) -> dict[str, tuple[str, ...]]:
    """Return the field names of POD layouts declared inside one C worker.

    A body-local helper such as ``Stream_ReadChunk::sized_chunk`` is not COM or
    public ABI owned by the model: its authoritative definition is part of the
    snippet being translated.  Keep this deliberately narrow.  Anonymous types,
    unions, enums, nested declarations, bitfields, and multi-declarators remain
    outside the accepted worker contract.
    """
    result: dict[str, tuple[str, ...]] = {}
    for match in re.finditer(r"\bstruct\s+([A-Za-z_]\w*)\s*\{", c_body):
        name = match.group(1)
        open_brace = c_body.find("{", match.start())
        try:
            inner, _ = _balanced_block(c_body, open_brace)
        except CppComError:
            continue
        fields: list[str] = []
        valid = True
        for declaration in inner.split(";"):
            declaration = " ".join(declaration.split())
            if not declaration:
                continue
            field = re.search(r"([A-Za-z_]\w*)\s*(?:\[[^]]+\])?\s*$", declaration)
            if field is None or any(token in declaration for token in (",", ":", "{")):
                valid = False
                break
            fields.append(field.group(1))
        if valid and fields:
            result[name] = tuple(fields)
    return result


def _source_anonymous_pod_fields(c_body: str) -> set[tuple[str, ...]]:
    """Return field sequences for simple body-local anonymous C records.

    C commonly spells a one-use lookup-table row as ``struct { ... } Rows[]``.
    Rust must name that local type, but the chosen name has no ABI or source
    semantics. Its field sequence is still source authority and is audited just
    like a named local POD.
    """
    result = set()
    for match in re.finditer(
        r"\bstruct\s*\{(?P<body>[^{}]*)\}\s*[A-Za-z_]\w*\s*(?:\[[^]]*\])?\s*=",
        c_body,
        flags=re.S,
    ):
        fields = []
        valid = True
        for declaration in match.group("body").split(";"):
            declaration = " ".join(declaration.split())
            if not declaration:
                continue
            field = re.search(r"([A-Za-z_]\w*)\s*(?:\[[^]]+\])?\s*$", declaration)
            if field is None or any(token in declaration for token in (",", ":", "{")):
                valid = False
                break
            fields.append(field.group(1))
        if valid and fields:
            result.add(tuple(fields))
    return result


def _rust_local_pod_fields(rust: str) -> dict[str, tuple[str, ...]]:
    result: dict[str, tuple[str, ...]] = {}
    for match in re.finditer(
        r"#\s*\[\s*repr\s*\(\s*C\s*\)\s*\]\s*struct\s+([A-Za-z_]\w*)\s*\{", rust
    ):
        name = match.group(1)
        open_brace = rust.find("{", match.start())
        try:
            inner, _ = _balanced_block(rust, open_brace)
        except CppComError:
            continue
        fields = tuple(
            field.group(1)
            for field in re.finditer(r"(?:^|,)\s*(?:pub\s+)?(?:r#)?([A-Za-z_]\w*)\s*:", inner)
        )
        result[name] = fields
    return result


def _rust_local_struct_fields(rust: str) -> dict[str, tuple[str, ...]]:
    """Return fields of any simple Rust local struct, irrespective of repr.

    This is used only for a source-anonymous, local-only record whose Rust name
    and layout representation are unobservable. Named C PODs continue to require
    the stricter `repr(C)` map above.
    """
    result = {}
    for match in re.finditer(r"\bstruct\s+([A-Za-z_]\w*)\s*\{", rust):
        name = match.group(1)
        open_brace = rust.find("{", match.start())
        try:
            inner, _ = _balanced_block(rust, open_brace)
        except CppComError:
            continue
        result[name] = tuple(
            field.group(1)
            for field in re.finditer(r"(?:^|,)\s*(?:pub\s+)?(?:r#)?([A-Za-z_]\w*)\s*:", inner)
        )
    return result


_DEALLOCATOR_CALL = re.compile(r"\b(Box::from_raw|__rustconv_\w*|free|malloc|calloc|realloc)\s*\(")

#: C++ `delete p` / `delete[] p` on something that is NOT the COM instance.
#:
#: `delete this` is excluded deliberately, and it is the whole distinction: the scaffold
#: allocates the COM object with HeapAlloc(GetProcessHeap()), so its only correct free is the
#: generated `<Class>_delete` -- `free` there is a cross-heap free. Any OTHER `delete` is a
#: plain C++ object, which this tree allocates with malloc (`operator new` calls malloc,
#: `operator delete` calls free), so `free` is its faithful lowering.
_CPP_DELETE = re.compile(r"\bdelete\s*(?:\[\s*\])?\s*(?!this\b)[A-Za-z_(*]")

#: C++ `new T{...}` / `new T[n]` — the allocating half, which licenses `malloc` for the same
#: reason `delete` licenses `free`: `operator new` IS malloc here. PLACEMENT new (`new (p) T`)
#: is excluded, since it allocates nothing.
_CPP_NEW = re.compile(r"\bnew\s+(?!\()[A-Za-z_]")


def _stray_deallocators(c_body: str, rust: str) -> list[str]:
    """Deallocators the RUST calls that the C source never did.

    Keyed on "the source did not call this", not on the bare name. A body translating a
    genuine C `free(p)` must keep it, and a check that refused the name outright would
    reject a correct translation — the defect is an INVENTED deallocator, not the concept
    of freeing. `Box::from_raw`/`__rustconv_*` have no C spelling at all, so they are
    always invented.
    """
    masked = re.sub(r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"', " ", c_body, flags=re.S)
    in_source = {match.group(1) for match in _DEALLOCATOR_CALL.finditer(masked)}
    # C++ `delete p` IS a source deallocator, and the C++ half of this pipeline is full of it:
    # CChangeNotifyServer `new`s a CWatchItem and `delete`s it two functions later, and three
    # correct bodies were rejected for reproducing that. What it licenses is `free`, because in
    # THIS tree `operator delete` IS `free` (sdk/lib/cpprt/.../cpprt.cpp: `operator new` calls
    # malloc, `operator delete` calls free). That distinction is the whole point: a
    # CDirectoryWatcher is `new`ed in CDirectoryWatcher.cpp, still C++, and `delete`d here in
    # Rust, so the two halves must agree on the ALLOCATOR and only malloc/free does.
    #
    # `Box::from_raw` therefore stays forbidden and is NOT what `delete` licenses. Rust's Box
    # frees through Rust's global allocator, which is not malloc -- and shell32-rust is no_std
    # with no `alloc` crate at all, so a body reaching for it does not even build. The original
    # rule was right to refuse it; only its stated reason (the COM instance's HeapAlloc) was
    # too narrow to explain why.
    if _CPP_DELETE.search(masked):
        in_source.add("free")
    if _CPP_NEW.search(masked):
        in_source.add("malloc")
    return sorted(
        {
            match.group(1)
            for match in _DEALLOCATOR_CALL.finditer(rust)
            if match.group(1) not in in_source
        }
    )


def _sizeof_operand_spans(text: str) -> list[tuple[int, int]]:
    """Spans of every `sizeof(...)` operand.

    Inside `sizeof`, a named constant contributes only its TYPE -- `UNICODE_NULL` is
    `((WCHAR)0)`, so `sizeof(UNICODE_NULL)` is 2 on every target and the faithful Rust
    is `core::mem::size_of::<u16>()`, which cannot mention the name. Demanding the token
    back rejects a body that is exactly right; this is the same disagreement the named-cast
    exemption above resolves, and the audit loses for the same reason.

    Occurrence-scoped, not name-scoped: a constant used for its VALUE anywhere else in the
    body still enters the audit set from that other occurrence, so this cannot mask a
    genuinely dropped constant.
    """
    spans = []
    for match in re.finditer(r"\bsizeof\s*\(", text):
        opening = match.end() - 1
        closing = _matching_paren(text, opening)
        if closing > opening:
            spans.append((opening + 1, closing))
    return spans


def audit_worker_body(
    c_body: str,
    rust: str,
    expected_name: str,
    expected_signature: str | None = None,
    *,
    scaffold_owned: bool = True,
) -> list[str]:
    """Verify the narrow contract for an LLM-translated COM method body.

    This is intentionally not a semantic verifier.  It protects the boundary that
    matters for the hybrid pipeline: the response must remain a plain worker, must
    preserve source-level symbolic constants, and must not attempt to own COM ABI.

    ``scaffold_owned=False`` audits the `--free-functions` half of the same TU, where the
    generated scaffold owns neither the export nor the ABI: a free function IS the exported
    C symbol, so `#[no_mangle]` and an `extern "C"` DEFINITION are required there and
    forbidden here. Everything else — invented deallocators, reversed call nesting, dropped
    constants, an `extern` BLOCK declaring substrate the model could not see — is
    language-neutral and applies to both halves.
    """
    findings: list[str] = []
    # A worker never owns deallocation. The scaffold allocates with HeapAlloc(GetProcessHeap())
    # and `X_delete` is the only matching free, so any OTHER deallocator surviving into a body
    # is a cross-heap free waiting for a crate where the name happens to resolve — it is silent
    # everywhere until it corrupts. The deterministic lowering above should have claimed these;
    # reaching here means it did not recognise the spelling, which must be loud, not quiet.
    stray_frees = _stray_deallocators(c_body, rust)
    if stray_frees:
        findings.append(
            "worker owns a deallocator it must not: "
            + ", ".join(stray_frees)
            + " — the scaffold allocates with HeapAlloc(GetProcessHeap()), so the only correct "
            "free is the generated <Class>_delete"
        )
    if not re.search(rf"\bfn\s+{re.escape(expected_name)}\s*\(", rust):
        findings.append(f"missing exact worker function {expected_name}")
    elif expected_signature is not None:
        match = re.search(
            rf"\bpub\s+unsafe\s+fn\s+{re.escape(expected_name)}\s*\(.*?\)\s*"
            r"(?:->\s*[^\{]+)?(?=\{)",
            rust,
            flags=re.S,
        )
        if not match or _canonical_signature(match.group(0)) != _canonical_signature(
            expected_signature
        ):
            actual = _canonical_signature(match.group(0)) if match else "<unparsed>"
            findings.append(
                "worker signature differs from deterministic contract: "
                f"expected `{_canonical_signature(expected_signature)}`, got `{actual}`"
            )

    executable_rust = re.sub(r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"', " ", rust, flags=re.S)

    # The opaque call tokens are occurrence-specific. Reject a reversed nesting
    # relation after unsealing, so a cheap model cannot turn
    # `HeapFree(GetProcessHeap(), ...)` into `GetProcessHeap(HeapFree(), ...)`
    # while still satisfying the closed call-count set. Benign hoisting such as
    # `let heap = GetProcessHeap(); HeapAlloc(heap, ...)` remains allowed.
    from .call_seal import _EXCLUDED as call_excluded

    def nested_calls(text: str) -> set[tuple[str, str]]:
        open_brace, close_brace = text.find("{"), text.rfind("}")
        body = text[open_brace + 1 : close_brace] if 0 <= open_brace < close_brace else text
        masked = _masked_cpp(body)
        excluded = (
            call_excluded
            | _CPP_COM_TYPE_WORDS
            | {
                "if",
                "for",
                "return",
                "switch",
                "while",
            }
        )
        calls = []
        for match in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", masked):
            name = match.group(1)
            if name in excluded:
                continue
            opening = masked.find("(", match.end(1))
            try:
                closing = _matching_paren(masked, opening)
            except CppComError:
                continue
            calls.append((name, opening, closing))
        return {
            (outer_name, inner_name)
            for outer_name, outer_open, outer_close in calls
            for inner_name, inner_open, inner_close in calls
            if outer_open < inner_open and inner_close < outer_close
        }

    source_nesting = nested_calls(strip_debug_statements(c_body))
    rust_nesting = nested_calls(executable_rust)
    reversed_pairs = sorted(
        (outer, inner)
        for outer, inner in source_nesting
        if outer != inner and (inner, outer) in rust_nesting
    )
    if reversed_pairs:
        findings.append(
            "runtime call nesting reversed from deterministic envelope: "
            + ", ".join(f"{outer}({inner}(...))" for outer, inner in reversed_pairs)
        )
    forbidden = {
        ".unwrap()": "Option/panic ABI guess",
        "scopeguard::": "model-owned ownership guard",
    }
    if scaffold_owned:
        # The scaffold emits the export and the vtable slot; a worker that also exports its
        # own symbol competes with it. A free function, by contrast, IS the exported symbol.
        forbidden["#[no_mangle]"] = "export attribute"
    for token, description in forbidden.items():
        if token in executable_rust:
            findings.append(f"worker emitted forbidden {description}: {token}")
    abi_surface = re.sub(r"/\*.*?\*/|//[^\n]*", " ", rust, flags=re.S)
    # An extern BLOCK is forbidden in BOTH halves and for the same reason: it declares
    # substrate the model could not see, and an invented forward declaration compiles,
    # links, and is silently wrong (AGENTS.md, substrate closure). Only the extern `fn`
    # DEFINITION is legitimate on the free half, where it carries the C ABI of a real export.
    if re.search(r'\bextern\s+"(?:C|system)"\s*\{', abi_surface):
        findings.append('worker emitted forbidden ABI declaration: extern "…" { }')
    if scaffold_owned and re.search(
        r'\b(?:pub\s+)?(?:unsafe\s+)?extern\s+"(?:C|system)"\s+fn\s+[A-Za-z_]\w*', abi_surface
    ):
        findings.append('worker emitted forbidden ABI declaration: extern "')
    if re.search(r"(?m)\bgoto\s+[A-Za-z_]\w*\s*;|^\s*[A-Za-z_]\w*\s*:\s*$", executable_rust):
        findings.append("worker emitted raw C goto/label syntax")
    if re.search(r"\?[^;{}\n]+:\s*[^:={}]", executable_rust):
        findings.append("worker emitted raw C ternary syntax")
    # Calling QueryInterface through an input object's deterministic vtable is
    # ordinary method-body behavior.  Only a model-owned implementation would
    # compete with the generated COM scaffold.
    if re.search(r"\bfn\s+[A-Za-z_]\w*QueryInterface\s*\(", executable_rust):
        findings.append("worker emitted forbidden QueryInterface implementation")
    if re.search(r"\bfn\s+[A-Za-z_]\w*CreateInstance\s*\(", executable_rust):
        findings.append("worker emitted forbidden activation factory implementation")
    if re.search(
        r"\b(?:struct|static)\s+[A-Za-z_]\w*Vtbl\b|\b[A-Za-z_]\w*Vtbl\s*\{",
        executable_rust,
    ):
        findings.append("worker emitted forbidden vtable definition")
    declared_types = re.findall(r"\b(?:struct|union|enum)\s+([A-Za-z_]\w*)\b", executable_rust)
    if declared_types:
        source_local = _source_local_pod_fields(c_body)
        anonymous_local = _source_anonymous_pod_fields(c_body)
        rust_local = _rust_local_pod_fields(executable_rust)
        rust_any_local = _rust_local_struct_fields(executable_rust)
        invalid_types = [
            name
            for name in declared_types
            if (
                (name not in source_local or rust_local.get(name) != source_local[name])
                and rust_any_local.get(name) not in anonymous_local
            )
        ]
        if invalid_types:
            findings.append(
                "worker emitted forbidden ABI type declaration: " + ", ".join(invalid_types)
            )

    # Strings and comments contain printf formats and prose, not program constants.
    audited_c_body = strip_debug_statements(c_body)
    open_brace, close_brace = audited_c_body.find("{"), audited_c_body.rfind("}")
    source_body = (
        audited_c_body[open_brace + 1 : close_brace]
        if open_brace >= 0 and close_brace > open_brace
        else audited_c_body
    )
    source = re.sub(r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"', " ", source_body, flags=re.S)
    # Preprocessor DIRECTIVES are not constant uses, and a body-local macro is the case that
    # bites. `CChangeNotifyServer::ShouldNotify` opens with `#define RETURN(x) do { ... }` and
    # closes with `#undef RETURN`: every CALL site is `RETURN(FALSE)`, correctly skipped as a
    # callable token, but the bare `RETURN` in the `#undef` is not followed by `(` -- so it
    # read as a dropped symbolic constant and rejected a body that had expanded the macro
    # exactly as the preprocessor does. The continuation form matters too: the definition
    # spans four lines with trailing backslashes, and it is a definition, not a use.
    source = re.sub(r"^[ \t]*#[^\n]*(?:\\\n[^\n]*)*$", "", source, flags=re.M)
    symbolic = set()
    ignored_words = _CPP_COM_DEBUG_MACROS | _CPP_COM_LITERAL_WORDS | _CPP_COM_TYPE_WORDS
    # A C++ named cast's `<...>` is a TYPE, and Rust either spells it differently or drops
    # it entirely — `const_cast` becomes nothing at all. fact_seal already refuses to seal
    # these for exactly that reason; this audit demanding the same token back is the two
    # mechanisms disagreeing, and the audit loses. CDefaultContextMenu::Initialize was
    # rejected over `const_cast<PCUITEMID_CHILD_ARRAY>(...)` in a body that was correct.
    cast_type_spans = _named_cast_spans(source)
    sizeof_spans = _sizeof_operand_spans(source)
    for match in re.finditer(r"\b[A-Z][A-Z0-9_]+\b", source):
        word = match.group(0)
        if any(a <= match.start() < b for a, b in cast_type_spans):
            continue
        if any(a <= match.start() < b for a, b in sizeof_spans):
            continue
        # Callable macro/function names (FAILED(...), SUCCEEDED(...),
        # HRESULT_FROM_WIN32(...)) are behavior, not constants. The body audit
        # separately preserves their arguments, branches, and numeric literals.
        tail = source[match.end() :]
        callable_token = tail.lstrip().startswith("(")
        start = match.start()
        cast_type = (
            start > 0
            and source[start - 1] == "("
            and (start == 1 or not (source[start - 2].isalnum() or source[start - 2] == "_"))
            and bool(re.match(r"\s*\**\s*\)", tail))
        )
        declaration_type = bool(re.match(r"(?:\s+|\s*\*+\s*)[A-Za-z_]\w*", tail))
        if (
            word not in ignored_words
            and not callable_token
            and not cast_type
            and not declaration_type
        ):
            symbolic.add(word)
    missing = sorted(word for word in symbolic if not re.search(rf"\b{re.escape(word)}\b", rust))
    if missing:
        findings.append("dropped symbolic C constant(s): " + ", ".join(missing))
    # A macro the BODY ITSELF defines has no substrate to come from, so defining it in Rust is
    # the correct translation, not a redefinition. CFSFolder's _AddFindResult opens with
    # `#define SUPER_HIDDEN (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM)` — demanding that
    # come from an island is demanding something that cannot exist, and the worker is rejected
    # however correct it is. The operands stay protected: they are sealed constants in their
    # own right, and the `missing` check above still requires the name to survive.
    locally_defined = {
        match.group(1) for match in re.finditer(r"#[ \t]*define[ \t]+([A-Za-z_]\w*)", source_body)
    }
    redefined = sorted(
        word
        for word in symbolic
        if word not in locally_defined
        and re.search(rf"(?<!\*)\b(?:pub\s+)?const\s+{re.escape(word)}\s*[:=]", rust)
    )
    if redefined:
        findings.append(
            "worker redefined source symbolic constant(s) instead of using substrate: "
            + ", ".join(redefined)
        )

    source_integers = _integer_values(source)
    rust_integers = _integer_values(rust)
    for pointer in re.findall(r"\*\s*([A-Za-z_]\w*)\s*=\s*0\s*;", source):
        if (
            re.search(rf"\*\s*{re.escape(pointer)}\s*=\s*core::ptr::null_mut\(\)\s*;", rust)
            and source_integers[0] > 0
        ):
            source_integers[0] -= 1
    missing_integers = source_integers - rust_integers
    if missing_integers:
        rendered = []
        for value, count in sorted(missing_integers.items()):
            rendered.append(f"{value}" + (f" (x{count})" if count > 1 else ""))
        findings.append("dropped or changed numeric C literal(s): " + ", ".join(rendered))
    return findings


def render_interface(spec: InterfaceSpec, island_slots: tuple[str, ...] | None = None) -> str:
    """Render the interface, unless the bindgen island already provides it.

    When it does, rendering a second copy is not a harmless duplicate: it is a DISTINCT Rust
    type with the same name, so every value that crosses between a worker body (which sees the
    island's, via the prelude glob) and the scaffold (which saw the local one) is a type error
    -- and the ones that do not error are the frightening half, because two structurally
    identical `#[repr(C)]` types will happily transmute past a cast.

    The island is an INDEPENDENT derivation of the same vtable -- bindgen through the real
    preprocessor, versus this module's read of the header text -- so a disagreement on slot
    count is two sources contradicting each other about an ABI. Never a warning.
    """
    if island_slots is not None:
        if len(island_slots) != len(spec.methods):
            raise CppComError(
                f"{spec.name}: the bindgen island has {len(island_slots)} vtable slot(s) but "
                f"{spec.source} parses to {len(spec.methods)}. These are two independent "
                f"derivations of one vtable and they disagree; a positional mismatch "
                f"dispatches calls to the wrong method. island={list(island_slots)} "
                f"header={[method.name for method in spec.methods]}"
            )
        return (
            f"// {spec.name}: {len(island_slots)} slot(s) from the bindgen island, which agrees "
            f"with {spec.source}.\n"
            f"// Not re-rendered here -- a second `{spec.name}` would be a distinct type with "
            f"the same name."
        )
    lines = [
        f"// Deterministic from {spec.source}",
        "#[repr(C)]",
        f"pub struct {spec.name} {{",
        f"    pub lpVtbl: *const {spec.name}Vtbl,",
        "}",
        "#[repr(C)]",
        f"pub struct {spec.name}Vtbl {{",
    ]
    for method in spec.methods:
        params = ", ".join(f"{name}: {c_type_to_rust(ty)}" for ty, name in method.params)
        ret = c_type_to_rust(method.return_type)
        lines.append(f'    pub {method.name}: unsafe extern "system" fn({params}) -> {ret},')
    lines.extend(
        [
            "}",
            'extern "C" {',
            f"    pub static IID_{spec.name}: IID;",
            "}",
        ]
    )
    return "\n".join(lines)


#: The COM module-lock symbol pair the scaffold calls around each instance's birth and
#: death. It is a property of the OUTPUT CRATE, not of the --target: the host DLL defines it,
#: and a staticlib linked into two DLLs needs a name BOTH of them define. menuband-rust is
#: linked into shell32.dll AND rshell.dll, so emitting shell32's private spelling there links
#: shell32 fine and leaves rshell with an undefined symbol -- which only surfaces when some
#: rshell-reachable object first pulls the module in, i.e. a cohab flip several TUs later.
#: `reactos.targets.module_lock_for_crate()` maps the crate to its pair.
_DEFAULT_MODULE_LOCK = ("Shell32RustObjectLock", "Shell32RustObjectUnlock")


def render_class_scaffold(
    cls: CppComClass,
    interfaces: list[InterfaceSpec],
    *,
    allow_internal: bool = False,
    allow_plain: bool = False,
    island_slots: Mapping[str, tuple[str, ...]] | None = None,
    module_lock: tuple[str, str] | None = None,
) -> str:
    """Render the ABI-owned portion. Method worker bodies remain separate LLM output.

    ``allow_plain`` must be threaded from the caller, not re-derived here: this is the SECOND
    preflight of a run, and the first one already gated the paid work. Defaulting it to False
    refused the assembly of an already-bought plain class — every worker replayed from cache,
    then the scaffold rejected the class that had just been translated.
    """
    cls.require_supported(allow_internal=allow_internal, allow_plain=allow_plain)
    lock_sym, unlock_sym = module_lock or _DEFAULT_MODULE_LOCK
    by_name = {item.name: item for item in interfaces}
    missing = [entry.interface for entry in cls.com_map if entry.interface not in by_name]
    if missing:
        raise CppComError(f"{cls.name}: missing interface specs: {', '.join(missing)}")
    # COM maps may expose several IIDs through one inherited interface slot.
    # Same-interface aliases are direct (for example IID_IOleWindow routed to
    # IDeskBand), while a derived interface and its named base also share one
    # physical C++ base subobject (IContextMenu2/IContextMenu).  WIDL's declared
    # inheritance is the authority; never infer this merely from vtable-prefix
    # similarity because unrelated interfaces all share IUnknown's prefix.
    mapped_interfaces = {entry.interface for entry in cls.com_map}

    def physical_interface(name: str) -> str:
        candidates = []
        for candidate in sorted(mapped_interfaces):
            current = by_name[candidate]
            while current.base is not None and current.base in by_name:
                if current.base == name:
                    candidates.append(candidate)
                    break
                current = by_name[current.base]
        if not candidates:
            return name
        longest = max(len(by_name[item].methods) for item in candidates)
        owners = [item for item in candidates if len(by_name[item].methods) == longest]
        if len(owners) != 1:
            raise CppComError(
                f"{cls.name}: IID for {name} has ambiguous mapped derived slots: "
                + ", ".join(owners)
            )
        return owners[0]

    physical_by_interface = {name: physical_interface(name) for name in mapped_interfaces}
    layout_entries = [
        next(entry for entry in cls.com_map if entry.interface == name)
        for name in dict.fromkeys(physical_by_interface[entry.interface] for entry in cls.com_map)
    ]
    # Non-QI'd interface bases get their own standalone vtable subobject in the
    # object layout (struct field + vtable + ctor init) but are absent from the QI
    # switch above, which iterates only cls.com_map. Field order is irrelevant here:
    # the object is Rust-allocated and every interface pointer recovers `this` via
    # offset_of!, so the C++ multiple-inheritance layout need not be reproduced.
    layout_entries = layout_entries + [
        ComMapEntry(iid=f"IID_{name}", interface=name, macro="COM_INTERFACE_ENTRY_LAYOUT")
        for name in cls.layout_bases
    ]

    lines = []
    for enum in cls.enum_state_types:
        lines.append(f"pub type {enum.name} = INT;")
        lines.extend(f"pub const {name}: {enum.name} = {value};" for name, value in enum.variants)
        lines.append("")
    for pod in cls.pod_state_types:
        representation = "#[repr(C, packed)]" if pod.packed else "#[repr(C)]"
        lines.extend([representation, f"pub struct {pod.name} {{"])
        for state in pod.fields:
            field_name = _rust_safe_field(state.name)
            lines.append(f"    pub {field_name}: {state.owned_rust_type},")
        lines.extend(["}", ""])

    lines.extend(
        [
            f"// Deterministic COM scaffold for {cls.name}; "
            "method bodies are generated separately.",
            'extern "system" {',
            "    fn GetProcessHeap() -> HANDLE;",
            '    #[link_name = "HeapAlloc"]',
            "    fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) "
            "-> *mut core::ffi::c_void;",
            '    #[link_name = "HeapFree"]',
            "    fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, "
            "memory: *mut core::ffi::c_void) -> BOOL;",
            f"    fn {lock_sym}();",
            f"    fn {unlock_sym}();",
            "}",
            "trait CppComSize { fn cpp_com_size(self) -> usize; }",
            "impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }",
            "impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }",
            "#[inline]",
            "unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) "
            "-> *mut core::ffi::c_void {",
            "    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())",
            "}",
            "#[inline]",
            "unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {",
            "    cpp_com_HeapFree(heap, flags, memory.cast())",
            "}",
            *(
                []
                if any(interface.name == "IUnknown" for interface in interfaces)
                else ['extern "C" {', "    pub static IID_IUnknown: IID;", "}"]
            ),
            "",
            "#[repr(C)]",
            f"pub struct {cls.name} {{",
        ]
    )
    for entry in layout_entries:
        lines.append(f"    pub {entry.interface}_iface: {entry.interface},")
    if cls.com_map or cls.atl_refcounted:
        # A plain helper has no COM lifetime: its owner `new`s and `delete`s it directly, so
        # a refcount field would be dead weight the C++ layout never had.
        #
        # But "no COM map" does NOT mean "not refcounted". A class deriving ATL's
        # `CComObjectRootEx<>` inherits `m_dwRef` and the InternalAddRef/InternalRelease pair
        # whether or not it exposes an interface — CMenuFocusManager has no BEGIN_COM_MAP and
        # still does `obj->AddRef()` / `if (!obj->Release())`. Omitting the field there is not
        # a layout saving, it is a missing member the bodies then cannot express.
        lines.append("    pub ref_count: core::sync::atomic::AtomicI32,")
    for state in cls.plain_state_fields():
        lines.append(f"    pub {state.name}: {state.rust_type},")
    lines.extend(["}", ""])

    # A class declared as `typedef struct _X { … } X, *PX;` is REFERRED TO by its aliases,
    # not by the tag: filetypes.cpp writes `PFILE_TYPE_ENTRY Entry` in every signature and
    # `_FILE_TYPE_ENTRY` nowhere outside the declaration itself. The SDK-POD renderer already
    # emits these aliases; the class scaffold did not, so a translated class whose tag differs
    # from its typedef name compiled to a struct nothing could name.
    for alias, pointer in _typedef_aliases(cls.source, cls.name):
        if alias == cls.name:
            continue
        target = f"*mut {cls.name}" if pointer else cls.name
        lines.append(f"pub type {alias} = {target};")
    if lines and lines[-1] != "":
        lines.append("")

    for state in cls.plain_state_fields():
        if state.com_ptr_interface is None:
            continue
        helper = f"{cls.name}_{state.assignment_helper}"
        interface = state.com_ptr_interface
        lines.extend(
            [
                f"unsafe fn {helper}(this: *mut {cls.name}, value: *mut {interface}) {{",
                f"    let old = (*this).{state.name};",
                "    if old == value { return; }",
                "    if !value.is_null() {",
                "        let unknown = value as *mut IUnknown;",
                '        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =',
                "            core::mem::transmute((*(*unknown).lpVtbl).AddRef);",
                "        add_ref(unknown);",
                "    }",
                f"    (*this).{state.name} = value;",
                "    if !old.is_null() {",
                "        let unknown = old as *mut IUnknown;",
                '        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =',
                "            core::mem::transmute((*(*unknown).lpVtbl).Release);",
                "        release(unknown);",
                "    }",
                "}",
                "",
            ]
        )

    for state in cls.plain_state_fields():
        if not state.owned_cstring:
            continue
        helper = f"{cls.name}_{state.string_assignment_helper}"
        lines.extend(
            [
                f"unsafe fn {helper}(this: *mut {cls.name}, value: LPCWSTR) {{",
                f"    (*this).{state.name}.assign(value);",
                "}",
                "",
            ]
        )

    for entry in layout_entries:
        iface = by_name[entry.interface]
        field_name = f"{entry.interface}_iface"
        prefix = f"{cls.name}_{entry.interface}"
        for method in iface.methods:
            params = list(method.params)
            rendered = ", ".join(f"{name}: {c_type_to_rust(ty)}" for ty, name in params)
            ret = c_type_to_rust(method.return_type)
            lines.append(
                f'unsafe extern "system" fn {prefix}_{method.name}({rendered}) -> {ret} {{'
            )
            this_name = params[0][1]
            lines.append(
                f"    let this = ({this_name} as *mut u8)"
                f".sub(core::mem::offset_of!({cls.name}, {field_name})) "
                f"as *mut {cls.name};"
            )
            rest = ", ".join(name for _, name in params[1:])
            # The IUnknown trio is ATL's to implement -- but only when the class actually
            # derives from ATL. A hand-rolled COM class writes its own QueryInterface,
            # AddRef and Release in the C++, so they translate to `_body` workers like any
            # other method, and routing the vtable slot at `{cls}_QueryInterface` instead
            # aims it at a function only the ATL branch below ever defines. That is not
            # merely an undefined symbol: had anything else supplied that name, the class's
            # OWN QueryInterface would have been silently bypassed in the shipped vtable.
            if cls.com_map and method.name in ("QueryInterface", "AddRef", "Release"):
                target = f"{cls.name}_{method.name}"
            else:
                target = f"{_interface_method_target(cls.name, method, interfaces)}_body"
            call = f"{target}(this{', ' if rest else ''}{rest})"
            lines.append(f"    {call}")
            lines.append("}")
        lines.extend(
            [
                "",
                f"pub static {prefix.upper()}_VTBL: {entry.interface}Vtbl = "
                f"{entry.interface}Vtbl {{",
            ]
        )
        # Slot NAMES come from the island when the island owns the type, because the Windows
        # A/W macros expand inside the interface definition and rename the slot itself:
        # `STDMETHOD(DeleteFile)` is `DeleteFileW` in the island. Paired POSITIONALLY, which
        # is the only pairing a vtable has -- and render_interface has already refused the
        # case where the two derivations disagree on how many slots there are.
        slots = island_slots.get(entry.interface) if island_slots else None
        for index, method in enumerate(iface.methods):
            field = slots[index] if slots else method.name
            # bindgen wraps every slot in `Option<fn>` (same layout, by the null-pointer
            # optimization); a locally rendered vtable uses a bare `fn`.
            value = f"Some({prefix}_{method.name})" if slots else f"{prefix}_{method.name}"
            lines.append(f"    {field}: {value},")
        lines.extend(["};", ""])

    if not cls.com_map:
        # Plain helper: everything below this point is COM lifetime machinery (QueryInterface,
        # AddRef/Release, the object-map factory) that the class does not have. What it DOES
        # need is a construction path, because its C++ owner reaches it through `new`/`delete`
        # — `new COpenWithList` at COpenWithMenu.cpp:224/1049/1360, `delete` at 229/1059/1393.
        # Emit that pair explicitly rather than leaving each caller to invent one: the
        # free-function bridges for this very class were bought before the class existed, and
        # duly invented `COpenWithList_new`, `COpenWithList_destructor`, and a free through
        # `ExFreePoolWithTag` — a KERNEL pool allocator, in a usermode shell DLL, releasing
        # memory it never allocated.
        # The vptr store. A C++ constructor sets the vtable pointer in its PROLOGUE, before
        # the written body runs -- so it appears in no source the model ever sees, and a
        # translated `_ctor_body` correctly contains only `m_ref = 1`. Nothing else can emit
        # it: this scaffold is what defines the vtable statics, so this scaffold owes the
        # store. Without it `_new` hands back an object whose `lpVtbl` is still the
        # HEAP_ZERO_MEMORY null, and the first call through the interface is a null deref
        # (a hand-rolled COM class has no ATL base to install it, unlike shell32's
        # CComObject-derived classes). Emitted BEFORE `_ctor_body` because C++ orders it
        # that way and a constructor body may call its own virtuals.
        vptr_stores = [
            f"    (*this).{entry.interface}_iface.lpVtbl = "
            f"core::ptr::addr_of!({(cls.name + '_' + entry.interface).upper()}_VTBL);"
            for entry in layout_entries
        ]
        lines.extend(
            [
                f"/// `new {cls.name}` — operator new, the vptr store, then the constructor body.",
                (
                    f"pub unsafe fn {cls.name}_new("
                    + ", ".join(
                        f"{name}: {c_type_to_rust(c_type)}" for c_type, name in cls.ctor_params
                    )
                    + f") -> *mut {cls.name} {{"
                ),
                "    let this = HeapAlloc(",
                "        GetProcessHeap(),",
                "        HEAP_ZERO_MEMORY,",
                f"        core::mem::size_of::<{cls.name}>(),",
                f"    ) as *mut {cls.name};",
                "    if this.is_null() { return core::ptr::null_mut(); }",
                *vptr_stores,
                (
                    f"    {cls.name}_ctor_body(this"
                    + "".join(f", {name}" for _c_type, name in cls.ctor_params)
                    + ");"
                ),
                "    this",
                "}",
                "",
                "/// `delete` — the destructor body, then the matching free. Pairs with",
                f"/// `{cls.name}_new`; freeing through any other allocator is a heap bug.",
                f"pub unsafe fn {cls.name}_delete(this: *mut {cls.name}) {{",
                "    if this.is_null() { return; }",
                f"    {cls.name}_dtor_body(this);",
                "    HeapFree(GetProcessHeap(), 0, this);",
                "}",
                "",
                # Only a class with interfaces has a QueryInterface to compare IIDs in. A true
                # plain helper (no vtable, e.g. shell32's COpenWithList) does not, so emitting
                # it there would be dead code and a pointless diff on the next regeneration.
                *([_CPP_COM_IID_EQUAL, ""] if layout_entries else []),
            ]
        )
        return "\n".join(lines)

    first = cls.com_map[0]
    first_physical = physical_by_interface[first.interface]
    lines.extend(
        [
            f"unsafe fn {cls.name}_QueryInterface(",
            f"    this: *mut {cls.name}, riid: REFIID, ppv: *mut *mut core::ffi::c_void,",
            ") -> HRESULT {",
            "    if ppv.is_null() { return E_POINTER; }",
            "    *ppv = core::ptr::null_mut();",
            f"    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || "
            f"cpp_com_iid_equal(riid, core::ptr::addr_of!({first.iid})) {{",
            f"        *ppv = core::ptr::addr_of_mut!((*this).{first_physical}_iface) "
            "as *mut core::ffi::c_void;",
            "    }",
        ]
    )
    for entry in cls.com_map[1:]:
        physical = physical_by_interface[entry.interface]
        lines.extend(
            [
                f"    else if cpp_com_iid_equal(riid, core::ptr::addr_of!({entry.iid})) {{",
                f"        *ppv = core::ptr::addr_of_mut!((*this).{physical}_iface) "
                "as *mut core::ffi::c_void;",
                "    }",
            ]
        )
    lines.extend(
        [
            "    else { return E_NOINTERFACE; }",
            f"    {cls.name}_AddRef(this);",
            "    S_OK",
            "}",
            "",
            f"unsafe fn {cls.name}_AddRef(this: *mut {cls.name}) -> ULONG {{",
            "    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1",
            "}",
            "",
            f"unsafe fn {cls.name}_Release(this: *mut {cls.name}) -> ULONG {{",
            "    let refs = (*this).ref_count.fetch_sub(1, "
            "core::sync::atomic::Ordering::SeqCst) - 1;",
            "    if refs == 0 {",
            f"        {cls.name}_dtor_body(this);",
        ]
    )
    for worker in cls.base_dtor_workers:
        lines.append(f"        {cls.name}_{worker}_body(this);")
    drop_pods = pods_with_drop(cls.pod_state_types)
    for state in cls.plain_state_fields():
        if state.assignment_helper is not None:
            lines.append(
                f"        {cls.name}_{state.assignment_helper}(this, core::ptr::null_mut());"
            )
        # A by-value POD member whose own fields are RAII is destroyed by the C++ owner
        # exactly like a CStringW member is; its Rust drop glue is what reproduces that.
        by_value_pod = "*" not in state.c_type and state.c_type.strip() in drop_pods
        if state.owned_cstring or state.raii_container is not None or by_value_pod:
            # The container's `Drop` in atl.rs IS the C++ member destructor the
            # translation dropped. It runs AFTER the explicit dtor body, which is C++'s
            # own order — and every container here has an idempotent RemoveAll, so an
            # explicit `RemoveAll()` in that body is not a double free.
            lines.append(f"        core::ptr::drop_in_place(&mut (*this).{state.name});")
        if state.owned_heap_ptr:
            # The C++ CComHeapPtr member dtor's implicit CoTaskMemFree.
            if state.array_len is not None:
                lines.append(f"        for slot in (*this).{state.name}.iter() {{")
                lines.append("            if !slot.is_null() { CoTaskMemFree(slot.cast()); }")
                lines.append("        }")
            else:
                lines.append(
                    f"        if !(*this).{state.name}.is_null() "
                    f"{{ CoTaskMemFree((*this).{state.name}.cast()); }}"
                )
    lines.extend(
        [
            f"        {unlock_sym}();",
            "        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);",
            "    }",
            "    refs as ULONG",
            "}",
            "",
            _CPP_COM_IID_EQUAL,
            "",
        ]
    )

    internal = allow_internal and cls.internal_shell_object_creator and not cls.object_map_clsid
    internal_init = internal and cls.internal_initializer_params is not None
    activation_comment = (
        "// Internal activation: this class has no CLSID; the DLL constructs it directly "
        "through\n// shellutils.h ShellObjectCreator<T> or ATL's own "
        "_CreatorClass::CreateInstance, both of\n// which take (outer, riid, ppv) — the "
        "signature of this factory."
        if internal
        else f"// Activation: {cls.object_map_clsid or '<not found>'} currently maps to "
        f"{cls.name} in shell32.cpp.\n"
        "// The generated C++ adapter redirects ATL activation to this factory."
    )
    factory_decl = (
        f"pub unsafe fn {cls.name}_CreateInstance("
        if internal
        else (f'#[no_mangle] pub unsafe extern "system" fn {cls.name}_CreateInstance(')
    )
    factory_params = []
    if internal_init:
        factory_params.extend(
            f"    {name}: {c_type_to_rust(c_type)},"
            for c_type, name in cls.internal_initializer_params or ()
        )
    else:
        factory_params.append("    outer: *mut IUnknown,")
    factory_params.append("    riid: REFIID, ppv: *mut *mut core::ffi::c_void,")
    lines.extend(
        [
            activation_comment,
            factory_decl,
            *factory_params,
            ") -> HRESULT {",
            "    if ppv.is_null() { return E_POINTER; }",
            "    *ppv = core::ptr::null_mut();",
            *(
                []
                if internal_init
                else ["    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }"]
            ),
            f"    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, "
            f"core::mem::size_of::<{cls.name}>()) as *mut {cls.name};",
            "    if this.is_null() { return E_OUTOFMEMORY; }",
            f"    core::ptr::write(this, {cls.name} {{",
        ]
    )
    for entry in layout_entries:
        prefix = f"{cls.name}_{entry.interface}".upper()
        lines.append(
            f"        {entry.interface}_iface: {entry.interface} {{ "
            f"lpVtbl: core::ptr::addr_of!({prefix}_VTBL) }},"
        )
    lines.extend(
        [
            "        ref_count: core::sync::atomic::AtomicI32::new(1),",
        ]
    )
    for state in cls.plain_state_fields():
        lines.append(f"        {state.name}: {state.init_expression},")
    lines.extend(
        [
            "    });",
            f"    {lock_sym}();",
            f"    {cls.name}_ctor_body(this);",
        ]
    )
    if cls.final_construct:
        lines.extend(
            [
                f"    let construct_hr = {cls.name}_FinalConstruct_body(this);",
                "    if construct_hr < 0 {",
                f"        {cls.name}_Release(this);",
                "        return construct_hr;",
                "    }",
            ]
        )
    if internal_init:
        init_args = ", ".join(name for _c_type, name in cls.internal_initializer_params or ())
        lines.extend(
            [
                f"    let mut hr = {cls.name}_Initialize_body(this"
                f"{', ' if init_args else ''}{init_args});",
                "    if hr >= 0 {",
                f"        hr = {cls.name}_QueryInterface(this, riid, ppv);",
                "    }",
                f"    {cls.name}_Release(this);",
                "    hr",
                "}",
            ]
        )
        return "\n".join(lines)
    lines.extend(
        [
            f"    let hr = {cls.name}_QueryInterface(this, riid, ppv);",
            f"    {cls.name}_Release(this);",
            "    hr",
            "}",
        ]
    )
    return "\n".join(lines)


def render_object_map_adapter(
    cls: CppComClass, *, module_lock: tuple[str, str] | None = None
) -> str:
    """C++ glue that keeps ATL registration but delegates object creation to Rust."""
    cls.require_supported()
    lock_sym, unlock_sym = module_lock or _DEFAULT_MODULE_LOCK
    # require_supported already proves both the activation entry and the
    # non-aggregatable lifetime contract used by the generated factory.
    adapter = f"{cls.name}RustEntry"
    lines = [
        f'extern "C" HRESULT WINAPI {cls.name}_CreateInstance(',
        "    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);",
        "",
        f"class {adapter} : public {cls.name}",
        "{",
        "public:",
        "    class _CreatorClass",
        "    {",
        "    public:",
        "        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)",
        "        {",
        f"            return {cls.name}_CreateInstance(",
        "                static_cast<IUnknown *>(pv), riid, ppv);",
        "        }",
        "    };",
        "};",
        "",
        f"// ObjectMap replacement: OBJECT_ENTRY({cls.object_map_clsid}, {adapter})",
        "// One-time bridge functions (place after gModule is defined):",
        f"#ifndef {lock_sym.upper()}_BRIDGE_DEFINED",
        f"#define {lock_sym.upper()}_BRIDGE_DEFINED",
        f'extern "C" void WINAPI {lock_sym}() {{ gModule.Lock(); }}',
        f'extern "C" void WINAPI {unlock_sym}() {{ gModule.Unlock(); }}',
        "#endif",
    ]
    source = cls.source.read_text(encoding="utf-8", errors="replace")
    if "SHELL_MapContextMenuVerbToCmdId" in source:
        lines.extend(
            [
                "",
                'extern "C" INT WINAPI Shell32RustMapContextMenuVerbToCmdId(',
                "    LPCMINVOKECOMMANDINFO pInfo, const CMVERBMAP *pMap)",
                "{",
                "    return SHELL_MapContextMenuVerbToCmdId(pInfo, pMap);",
                "}",
            ]
        )
    if "SHELL_GetCommandStringImpl" in source:
        lines.extend(
            [
                "",
                'extern "C" HRESULT WINAPI Shell32RustGetCommandStringImpl(',
                "    SIZE_T CmdId, UINT uFlags, LPSTR Buf, UINT cchBuf, const CMVERBMAP *pMap)",
                "{",
                "    return SHELL_GetCommandStringImpl(CmdId, uFlags, Buf, cchBuf, pMap);",
                "}",
            ]
        )
    if "SHELL32_CoCreateInitSFWithCsidl" in source:
        # The csidl form used to be a second OVERLOAD of SHELL32_CoCreateInitSF, which Rust
        # cannot name — hence this trampoline. shlfolder is Rust now, so shfldr.h renamed it
        # and made both forms EXTERN_C; the trampoline stays only because the five already
        # translated folder classes declare it `extern "system"` (it is WINAPI, the renamed
        # function is cdecl), and repointing them is an ABI change for no functional gain.
        lines.extend(
            [
                "",
                'extern "C" HRESULT WINAPI Shell32RustCoCreateInitSF(',
                "    LPCITEMIDLIST pidlRoot, const GUID *clsid, INT csidl,",
                "    REFIID riid, LPVOID *ppvOut)",
                "{",
                "    return SHELL32_CoCreateInitSFWithCsidl(pidlRoot, clsid, csidl, riid, ppvOut);",
                "}",
            ]
        )
    return "\n".join(lines)


def discover_widl_header(
    repo_root: Path,
    arch: str,
    filename: str = "shobjidl.h",
    *,
    interface: str | None = None,
    extra_roots: Sequence[Path] = (),
) -> Path:
    if interface is not None:
        needles = (
            f"typedef struct {interface}Vtbl",
            f"DECLARE_INTERFACE_({interface},",
            f"DECLARE_INTERFACE_ ({interface},",
        )
        # WIDL-generated roots first, then the checked-in ones. The generated roots exist only
        # after a build of THIS worktree — never resolve them from a sibling, which would check
        # this tree's Rust against another lane's headers at another commit (AGENTS.md
        # "Fleet isolation applies to READS too").
        generated_roots = [
            repo_root / f"build-{arch}" / "sdk" / "include" / "psdk",
            repo_root / f"build-{arch}-cross" / "sdk" / "include" / "psdk",
            # reactos-private IDLs (e.g. shobjidl_undoc.idl) generate here.
            repo_root / f"build-{arch}" / "sdk" / "include" / "reactos",
        ]
        roots = [
            *generated_roots,
            repo_root / "sdk" / "include" / "psdk",
            repo_root / "sdk" / "include" / "reactos",
            # ...and the module's own directory LAST, for a module that declares its interfaces
            # in a hand-written header instead of an .idl (shellrecyclebin's recyclebin.h owns
            # IRecycleBin / IRecycleBin5 / IRecycleBinEnumList / IRecycleBinFile). The
            # DECLARE_INTERFACE_ needle below already matches that spelling; it simply was never
            # looked for outside the SDK. Last so an SDK interface of the same name still wins.
            *extra_roots,
        ]
        hits = []
        for root in roots:
            if not root.is_dir():
                continue
            for candidate in sorted(root.glob("*.h")):
                text = candidate.read_text(encoding="utf-8", errors="replace")
                if any(needle in text for needle in needles):
                    hits.append(candidate)
        if len(hits) == 1:
            return hits[0]
        if len(hits) > 1:
            raise CppComError(
                f"{interface}: flattened vtable is ambiguous across generated headers: "
                + ", ".join(str(path) for path in hits)
            )
        # Most interfaces are declared ONLY in a WIDL-generated header, so an unprovisioned
        # worktree fails here for a reason that has nothing to do with the caller's change.
        # Say which of the two it is: a missing build dir names its own fix, and a genuinely
        # undeclared interface stays a hard error rather than an inferred slot order.
        if not any(root.is_dir() for root in generated_roots):
            raise CppComError(
                f"{interface}: no generated SDK headers for {arch} — this worktree has no "
                f"build-{arch}, and the checked-in headers do not declare it. "
                f"fix: ./vibeos-cli build --arch {arch}"
            )
        raise CppComError(
            f"{interface}: no flattened vtable definition found in compiler-facing SDK headers "
            f"for {arch}; refusing to infer slot order"
        )

    candidates = [
        repo_root / f"build-{arch}" / "sdk" / "include" / "psdk" / filename,
        repo_root / f"build-{arch}-cross" / "sdk" / "include" / "psdk" / filename,
    ]
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    raise CppComError(
        f"generated SDK header {filename} not found for {arch}; build that arch first or pass "
        "--sdk-header explicitly. Refusing to guess COM slot order."
    )
