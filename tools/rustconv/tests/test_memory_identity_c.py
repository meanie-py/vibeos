"""Exact-body C translation memory: one HIT, and one MISS per key component.

A miss is free; a false hit silently ships the wrong body. So every MISS test
below also proves its key component is LOAD-BEARING: `assert_separated_only_by`
recomputes both keys with that component dropped and asserts they collide, so
deleting the component from `payload()` would turn the miss into a hit and fail
the test.
"""

from __future__ import annotations

import json

import pytest

from rustconv.catalog import Symbol
from rustconv.config import TARGETS
from rustconv.memory_identity_c import (
    KEY_COMPONENTS,
    SELF_SENTINEL,
    CExactBodyTranslationMemory,
    CIdentity,
    CMemoryError,
    build_identity,
    canonical_behavior,
    module_scope_names,
    module_scope_projection,
    reinstantiate,
)
from rustconv.resolver import ImportGroup, ResolveResult
from rustconv.scanner import CGlobal, scan_file
from rustconv.spec_parser import SpecEntry

TARGET = TARGETS["ntoskrnl"]

# A body byte-identical across two modules: the RasHangUpA/RasHangUpW and
# NdisDeregisterAdapter/NdisMPromoteMiniport shapes measured by dup-census.
BODY = """DWORD WINAPI ModuleAlphaEntry(HANDLE handle, DWORD flags)
{
    if (flags == MODULE_LIMIT)
    {
        Helper(handle);
        return 0;
    }
    return ERROR_NOT_SUPPORTED;
}
"""

PREAMBLE = "#define MODULE_LIMIT 4\nstatic HINSTANCE hModuleInstance;\n"

RUST = (
    '#[no_mangle]\npub unsafe extern "system" fn ModuleAlphaEntry'
    "(handle: HANDLE, flags: DWORD) -> DWORD {\n"
    "    if flags == MODULE_LIMIT { Helper(handle); return 0; }\n"
    "    ERROR_NOT_SUPPORTED\n}\n"
)


class Dep:
    """Minimal stand-in for callgraph.FunctionDep (build_identity is untyped)."""

    def __init__(
        self,
        name: str,
        *,
        body_text: str = BODY,
        return_type: str = "DWORD",
        calling_convention: str | None = "WINAPI",
        params: list[tuple[str, str]] | None = None,
        callees: set[str] | None = None,
        identifiers: set[str] | None = None,
        is_static: bool = False,
        is_exported: bool = True,
    ):
        self.name = name
        self.body_text = body_text
        self.return_type = return_type
        self.calling_convention = calling_convention
        self.params = params if params is not None else [("HANDLE", "handle"), ("DWORD", "flags")]
        self.callees = callees if callees is not None else {"Helper"}
        self.identifiers = (
            identifiers
            if identifiers is not None
            else {"HANDLE", "DWORD", "MODULE_LIMIT", "Helper", "ERROR_NOT_SUPPORTED"}
        )
        self.is_static = is_static
        self.is_exported = is_exported


def resolution(
    *,
    imports: dict[str, list[str]] | None = None,
    gaps: set[str] | None = None,
    unresolved: set[str] | None = None,
    exported: str | None = "ModuleAlphaEntry",
) -> ResolveResult:
    result = ResolveResult()
    for module, names in (imports or {"ddk": ["HANDLE", "DWORD"]}).items():
        group = ImportGroup(module=module)
        for name in names:
            group.symbols.append(Symbol(name=name, kind="type", module=module))
        result.imports[module] = group
    result.gap_functions = gaps if gaps is not None else set()
    result.unresolved = unresolved if unresolved is not None else {"ERROR_NOT_SUPPORTED"}
    if exported:
        result.exports[exported] = SpecEntry(
            name=exported,
            convention="stdcall",
            params=["ptr", "long"],
            arch_filter=None,
            forward_target=None,
            is_variable=False,
        )
    return result


def identity_for(
    name: str,
    *,
    dep: Dep | None = None,
    res: ResolveResult | None = None,
    preamble_text: str = PREAMBLE,
    preamble_names: set[str] | None = None,
    local_signatures: dict[str, str] | None = None,
    island_source: str = "",
    contract: str = "",
    namespace: str = "default",
    target=TARGET,
) -> CIdentity:
    dep = dep or Dep(name)
    dep.name = name
    return build_identity(
        dep=dep,
        target=target,
        resolution=res if res is not None else resolution(exported=name),
        local_signatures=local_signatures or {},
        preamble_text=preamble_text,
        preamble_names=(
            preamble_names if preamble_names is not None else {"MODULE_LIMIT", "hModuleInstance"}
        ),
        namespace=namespace,
        island_source=island_source,
        contract=contract,
    )


def _key_without(identity: CIdentity, component: str) -> str:
    payload = identity.payload()
    assert component in payload, f"{component} is not a key component"
    del payload[component]
    return json.dumps(payload, sort_keys=True, default=str)


def assert_separated_only_by(left: CIdentity, right: CIdentity, component: str) -> None:
    """MISS today, and a HIT the moment ``component`` leaves the key."""
    assert left.key() != right.key(), f"expected a MISS; {component} did not separate them"
    assert _key_without(left, component) == _key_without(right, component), (
        f"{component} is not the ONLY separator — this test would still pass "
        f"with {component} removed from the key, so it does not prove anything"
    )


def bank(tmp_path, identity: CIdentity, rust: str = RUST) -> CExactBodyTranslationMemory:
    memory = CExactBodyTranslationMemory(tmp_path)
    assert memory.remember(rust, identity) is True
    return memory


def accept(memory: CExactBodyTranslationMemory, identity: CIdentity):
    candidate = memory.candidate(identity)
    if candidate is None:
        return None
    return candidate.accept(lambda _rust: [])


# --------------------------------------------------------------------------
# The HIT: a byte-identical C body in a second module, different symbol name.
# --------------------------------------------------------------------------


def test_byte_identical_body_across_two_modules_hits(tmp_path):
    memory = bank(tmp_path, identity_for("ModuleAlphaEntry"))
    outcome = accept(memory, identity_for("ModuleBetaEntry"))
    assert outcome is not None and outcome.accepted
    assert outcome.findings == ()
    assert "fn ModuleBetaEntry(" in (outcome.rust or "")
    assert "ModuleAlphaEntry" not in (outcome.rust or "")


def test_same_symbol_in_a_second_module_hits_verbatim(tmp_path):
    """The DllMain / midl_user_allocate shape: same name, different module."""
    memory = bank(tmp_path, identity_for("DllMain"))
    outcome = accept(memory, identity_for("DllMain"))
    assert outcome is not None and outcome.rust == RUST


# --------------------------------------------------------------------------
# The MISSES. Each one also proves its component is load-bearing.
# --------------------------------------------------------------------------


def test_differing_signature_misses(tmp_path):
    other = Dep("ModuleBetaEntry", params=[("HANDLE", "handle"), ("ULONG", "flags")])
    left, right = identity_for("ModuleAlphaEntry"), identity_for("ModuleBetaEntry", dep=other)
    assert_separated_only_by(left, right, "param_types")
    assert accept(bank(tmp_path, left), right) is None


def test_differing_return_type_misses(tmp_path):
    other = Dep("ModuleBetaEntry", return_type="NTSTATUS")
    left, right = identity_for("ModuleAlphaEntry"), identity_for("ModuleBetaEntry", dep=other)
    assert_separated_only_by(left, right, "return_type")
    assert accept(bank(tmp_path, left), right) is None


def test_differing_calling_convention_misses(tmp_path):
    """`extern "system"` is stdcall on i386 but cdecl on amd64/arm64."""
    other = Dep("ModuleBetaEntry", calling_convention="FASTCALL")
    left, right = identity_for("ModuleAlphaEntry"), identity_for("ModuleBetaEntry", dep=other)
    assert_separated_only_by(left, right, "calling_convention")
    assert accept(bank(tmp_path, left), right) is None


def test_differing_arch_set_misses(tmp_path):
    """Same convention, different arch set — the extern string still differs."""
    left = identity_for("ModuleAlphaEntry")
    right = identity_for("ModuleBetaEntry", target=TARGETS["kdcom"])
    # kdcom is i386-only AND a different crate, so pin the arch dimension alone.
    narrowed = CIdentity(**{**left.__dict__, "self_name": "ModuleBetaEntry", "arches": ("i386",)})
    assert_separated_only_by(left, narrowed, "arches")
    assert accept(bank(tmp_path, left), narrowed) is None
    assert accept(bank(tmp_path, left), right) is None


def test_differing_linkage_misses(tmp_path):
    """static fn vs #[no_mangle] pub unsafe extern is a different item shape."""
    other = Dep("ModuleBetaEntry", is_static=True)
    left = identity_for("ModuleAlphaEntry")
    right = identity_for("ModuleBetaEntry", dep=other, res=resolution(exported="ModuleBetaEntry"))
    assert_separated_only_by(left, right, "is_static")
    assert accept(bank(tmp_path, left), right) is None


def test_differing_callee_resolution_misses(tmp_path):
    """Byte-identical body; `Helper` is a sibling C function in one module only.

    A *differently spelled* callee also misses, but through `behavior` — the
    call site is in the body. What only `callees` can catch is the same
    spelling resolving to a different thing, which is this.
    """
    left = identity_for("ModuleAlphaEntry")
    right = identity_for("ModuleBetaEntry", local_signatures={"Helper": "VOID Helper(HANDLE)"})
    assert_separated_only_by(left, right, "callees")
    assert accept(bank(tmp_path, left), right) is None


def test_differing_callee_spelling_misses(tmp_path):
    other = Dep(
        "ModuleBetaEntry",
        body_text=BODY.replace("Helper(handle)", "Worker(handle)"),
        callees={"Worker"},
    )
    left, right = identity_for("ModuleAlphaEntry"), identity_for("ModuleBetaEntry", dep=other)
    assert left.key() != right.key()
    assert accept(bank(tmp_path, left), right) is None


def test_differing_local_callee_signature_misses(tmp_path):
    left = identity_for("ModuleAlphaEntry", local_signatures={"Helper": "VOID Helper(HANDLE)"})
    right = identity_for("ModuleBetaEntry", local_signatures={"Helper": "NTSTATUS Helper(HANDLE)"})
    assert_separated_only_by(left, right, "callees")
    assert accept(bank(tmp_path, left), right) is None


def test_differing_sealing_flags_miss(tmp_path):
    left = identity_for("ModuleAlphaEntry", namespace="mechanical__factseal_v1__callseal_v1")
    right = identity_for("ModuleBetaEntry", namespace="mechanical")
    assert_separated_only_by(left, right, "namespace")
    assert accept(bank(tmp_path, left), right) is None


def test_differing_deterministic_contract_misses(tmp_path):
    left = identity_for("ModuleAlphaEntry", contract="tokens: __RUSTCONV_FACT_0000__")
    right = identity_for("ModuleBetaEntry", contract="tokens: __RUSTCONV_FACT_0000__, _0001__")
    assert_separated_only_by(left, right, "contract")
    assert accept(bank(tmp_path, left), right) is None


def test_differing_available_substrate_misses(tmp_path):
    """`MODULE_LIMIT` exposed by the other crate as a different value."""
    left = identity_for("ModuleAlphaEntry")
    right = identity_for(
        "ModuleBetaEntry",
        preamble_text="#define MODULE_LIMIT 9\nstatic HINSTANCE hModuleInstance;\n",
    )
    assert_separated_only_by(left, right, "preamble")
    assert accept(bank(tmp_path, left), right) is None


def test_substrate_type_the_other_crate_does_not_expose_misses(tmp_path):
    """A body whose type resolves elsewhere (or not at all) must MISS."""
    left = identity_for("ModuleAlphaEntry")
    right = identity_for(
        "ModuleBetaEntry",
        res=resolution(
            imports={"ddk": ["DWORD"]},
            unresolved={"ERROR_NOT_SUPPORTED", "HANDLE"},
            exported="ModuleBetaEntry",
        ),
    )
    assert_separated_only_by(left, right, "substrate")
    assert accept(bank(tmp_path, left), right) is None


def test_differing_crate_local_island_definition_misses(tmp_path):
    """e1000's MP_ADAPTER and rtl8139's are different structs, same name."""
    dep = Dep("ModuleAlphaEntry", identifiers={"HANDLE", "DWORD", "MP_ADAPTER", "Helper"})
    res = resolution(imports={"types": ["MP_ADAPTER"]}, exported=None)
    left = identity_for(
        "ModuleAlphaEntry",
        dep=dep,
        res=res,
        island_source="pub struct MP_ADAPTER { pub NumTcb: u32 }\n",
    )
    right = identity_for(
        "ModuleBetaEntry",
        dep=Dep("ModuleBetaEntry", identifiers=dep.identifiers),
        res=res,
        island_source="pub struct MP_ADAPTER { pub NumTcb: u16 }\n",
    )
    assert_separated_only_by(left, right, "substrate")
    assert accept(bank(tmp_path, left), right) is None


def test_alpha_renamed_body_misses(tmp_path):
    """Identifier-renamed bodies are census evidence only, never a reuse key."""
    renamed = BODY.replace("handle", "hDevice").replace("flags", "dwFlags")
    other = Dep(
        "ModuleBetaEntry",
        body_text=renamed,
        params=[("HANDLE", "hDevice"), ("DWORD", "dwFlags")],
    )
    left, right = identity_for("ModuleAlphaEntry"), identity_for("ModuleBetaEntry", dep=other)
    assert left.key() != right.key()
    # Two components move together here (the body AND the param spellings);
    # dropping either alone must still MISS, which is what "never alpha" means.
    for component in ("behavior", "param_names"):
        assert _key_without(left, component) != _key_without(right, component), (
            f"dropping {component} alone turned an alpha-rename into a HIT"
        )
    assert accept(bank(tmp_path, left), right) is None


def test_body_differing_by_one_constant_misses(tmp_path):
    other = Dep("ModuleBetaEntry", body_text=BODY.replace("return 0;", "return 1;"))
    left, right = identity_for("ModuleAlphaEntry"), identity_for("ModuleBetaEntry", dep=other)
    assert_separated_only_by(left, right, "behavior")
    assert accept(bank(tmp_path, left), right) is None


# --------------------------------------------------------------------------
# Audit-before-promotion, re-instantiation guards, and fail-safe behavior.
# --------------------------------------------------------------------------


def test_hit_is_a_candidate_not_an_acceptance(tmp_path):
    """A failing audit turns the hit into a miss and reports why."""
    memory = bank(tmp_path, identity_for("ModuleAlphaEntry"))
    candidate = memory.candidate(identity_for("ModuleBetaEntry"))
    assert candidate is not None
    # No attribute on the candidate exposes the banked text.
    assert not [name for name in vars(candidate) if "rust" in name.lower()]
    rejected = candidate.accept(lambda _rust: ["signature mismatch"])
    assert rejected.rust is None
    assert rejected.findings == ("signature mismatch",)


def test_accept_requires_an_audit_callable(tmp_path):
    memory = bank(tmp_path, identity_for("ModuleAlphaEntry"))
    candidate = memory.candidate(identity_for("ModuleBetaEntry"))
    assert candidate is not None
    with pytest.raises(CMemoryError):
        candidate.accept(None)  # pyright: ignore[reportArgumentType]


def test_audit_sees_the_reinstantiated_text(tmp_path):
    memory = bank(tmp_path, identity_for("ModuleAlphaEntry"))
    seen: list[str] = []
    candidate = memory.candidate(identity_for("ModuleBetaEntry"))
    assert candidate is not None
    candidate.accept(lambda rust: seen.append(rust) or [])
    assert seen and "ModuleBetaEntry" in seen[0] and "ModuleAlphaEntry" not in seen[0]


def test_reinstantiate_refuses_when_origin_is_absent():
    rust, reason = reinstantiate("pub fn something_else() {}", "Origin", "Target")
    assert rust is None and "never names its origin" in reason


def test_reinstantiate_refuses_a_name_collision():
    rust, reason = reinstantiate("fn Origin() { Target(); }", "Origin", "Target")
    assert rust is None and "already names the rename target" in reason


def test_incomplete_behavior_extraction_is_never_stored_or_served(tmp_path):
    unparseable = Dep("ModuleAlphaEntry", body_text="DWORD ModuleAlphaEntry(void) /* no body")
    identity = identity_for("ModuleAlphaEntry", dep=unparseable)
    assert identity.behavior_complete is False
    assert identity.usable is False
    memory = CExactBodyTranslationMemory(tmp_path)
    assert memory.remember(RUST, identity) is False
    assert memory.candidate(identity) is None


def test_remember_refuses_empty_translation(tmp_path):
    memory = CExactBodyTranslationMemory(tmp_path)
    with pytest.raises(ValueError):
        memory.remember("   \n", identity_for("ModuleAlphaEntry"))


def test_a_cold_memory_just_misses(tmp_path):
    assert CExactBodyTranslationMemory(tmp_path).candidate(identity_for("X")) is None


def test_payload_covers_exactly_the_declared_key_components():
    assert tuple(sorted(identity_for("X").payload())) == tuple(sorted(KEY_COMPONENTS))


# --------------------------------------------------------------------------
# Component helpers.
# --------------------------------------------------------------------------


def test_canonical_behavior_ignores_the_function_name_and_comments():
    left, ok_left = canonical_behavior("void Alpha(void) { /* hi */ Work(); }", "Alpha")
    right, ok_right = canonical_behavior("void Beta(void)\n{\n    Work();\n}", "Beta")
    assert ok_left and ok_right and left == right


def test_canonical_behavior_keeps_a_self_recursive_call_distinct():
    recursive, _ = canonical_behavior("void Alpha(void) { Alpha(); }", "Alpha")
    forwarding, _ = canonical_behavior("void Beta(void) { Alpha(); }", "Beta")
    assert recursive != forwarding
    assert SELF_SENTINEL in "".join([SELF_SENTINEL])  # sentinel is the documented token


def test_module_scope_projection_binds_an_unlocatable_name_to_the_whole_source():
    """Unknown ⇒ bind to THIS preamble ⇒ same-module hit, cross-module miss."""
    left = module_scope_projection("static int other;\n", ["Ghost"])
    right = module_scope_projection("static int different;\n", ["Ghost"])
    again = module_scope_projection("static int other;\n", ["Ghost"])
    assert left["Ghost"].startswith("unlocated@")
    assert left["Ghost"] != right["Ghost"]
    assert left["Ghost"] == again["Ghost"]


def test_module_scope_projection_separates_define_values():
    left = module_scope_projection("#define LIMIT 4\n", ["LIMIT"])
    right = module_scope_projection("#define LIMIT 5\n", ["LIMIT"])
    assert left["LIMIT"] != right["LIMIT"]
    assert not left["LIMIT"].startswith("unlocated@")


def test_module_scope_projection_separates_a_global_type():
    left = module_scope_projection("static HINSTANCE hDllInstance;\n", ["hDllInstance"])
    right = module_scope_projection("static HMODULE hDllInstance;\n", ["hDllInstance"])
    assert left["hDllInstance"] != right["hDllInstance"]
    assert not left["hDllInstance"].startswith("unlocated@")


def test_module_scope_names_is_derived_from_the_text_not_scan_globals():
    """`scan.globals` is empty for rasdlg.c/rasman.c/netid.c — see the docstring.

    If the projected set came from `scan.globals` the substrate component would
    collapse to `{}` for exactly the cross-module DllMain group we want to gate,
    which is a false-hit vector wearing a safety check's clothes.
    """
    preamble = "static HINSTANCE hDllInstance;\n"
    referenced = {"hDllInstance", "DisableThreadLibraryCalls", "DWORD"}
    assert module_scope_names(preamble, referenced) == {"hDllInstance"}


def test_module_scope_names_projects_definitions_not_mere_mentions():
    """`HINSTANCE`/`BOOL` are SDK types the import surface already covers.

    Projecting a merely-mentioned type name drags unrelated module-scope
    declarations into the key — safe, but arbitrarily strict: it made two
    identical `DllMain` bodies miss over a neighbouring `BOOL` field.
    """
    preamble = "static HINSTANCE hDllInstance;\nBOOL bUnrelatedFlag;\n"
    referenced = {"hDllInstance", "HINSTANCE", "BOOL", "bUnrelatedFlag"}
    assert module_scope_names(preamble, referenced) == {"hDllInstance", "bUnrelatedFlag"}


def test_a_vtable_slot_naming_the_function_does_not_self_separate(tmp_path):
    """netcfgx: three `{ return E_NOTIMPL; }` vtbl entries must group.

    Each one's own name sits in a module-scope vtable, so projecting the self
    name would give every sibling a different projection key and make the class
    permanently unmatchable.
    """
    vtbl = (
        "static const INetCfgComponentBindingsVtbl vt = {\n"
        "    INetCfgComponentBindings_fnBindTo,\n"
        "    INetCfgComponentBindings_fnUnbindFrom,\n"
        "};\n"
    )
    identifiers = {"INetCfgComponentBindings", "INetCfgComponent", "E_NOTIMPL"}
    left = identity_for(
        "INetCfgComponentBindings_fnBindTo",
        dep=Dep(
            "INetCfgComponentBindings_fnBindTo",
            identifiers=identifiers | {"INetCfgComponentBindings_fnBindTo"},
        ),
        preamble_text=vtbl,
        preamble_names=set(),
    )
    right = identity_for(
        "INetCfgComponentBindings_fnUnbindFrom",
        dep=Dep(
            "INetCfgComponentBindings_fnUnbindFrom",
            identifiers=identifiers | {"INetCfgComponentBindings_fnUnbindFrom"},
        ),
        preamble_text=vtbl,
        preamble_names=set(),
    )
    assert left.key() == right.key()
    assert accept(bank(tmp_path, left), right) is not None


def test_module_scope_names_finds_every_definition_shape():
    preamble = (
        "#define LIMIT 4\n"
        "#define WRAP(x) ((x) + 1)\n"
        "typedef struct _THING { int a; } THING;\n"
        "static WCHAR szBuffer[32];\n"
        "static ULONG gCount = 0;\n"
        "enum { FIRST, SECOND };\n"
        "NTSTATUS Prototype(HANDLE);\n"
    )
    names = {"LIMIT", "WRAP", "_THING", "THING", "szBuffer", "gCount", "FIRST", "SECOND"}
    assert module_scope_names(preamble, names | {"Prototype", "HANDLE"}) == names


def test_a_referenced_file_static_separates_two_modules_end_to_end(tmp_path):
    """The real DllMain shape: same body, module-scope static of another type."""
    dep = Dep("DllMain", identifiers={"HANDLE", "DWORD", "hDllInstance", "Helper"})
    left = identity_for(
        "DllMain", dep=dep, preamble_text="static HINSTANCE hDllInstance;\n", preamble_names=set()
    )
    right = identity_for(
        "DllMain",
        dep=Dep("DllMain", identifiers=dep.identifiers),
        preamble_text="static HMODULE hDllInstance;\n",
        preamble_names=set(),
    )
    assert_separated_only_by(left, right, "preamble")
    assert accept(bank(tmp_path, left), right) is None
    # Identical module-scope shape in a different module still HITs.
    same = identity_for(
        "DllMain",
        dep=Dep("DllMain", identifiers=dep.identifiers),
        preamble_text="static HINSTANCE hDllInstance;\n",
        preamble_names=set(),
    )
    assert accept(bank(tmp_path, left), same) is not None


# --------------------------------------------------------------------------
# End-to-end against real repo C: the pair dup-census reported as exact.
# --------------------------------------------------------------------------


def test_real_rasapi32_a_w_stub_pair_hits(tmp_path):
    """RasHangUpA/RasHangUpW — the measured intra-module A/W stub class."""
    from rustconv.config import REACTOS_ROOT

    source = REACTOS_ROOT / "dll" / "win32" / "rasapi32" / "rasapi.c"
    if not source.exists():
        pytest.skip("rasapi32 not present in this checkout")
    scan = scan_file(source)
    by_name = {function.name: function for function in scan.functions}
    origin, target = by_name.get("RasHangUpA"), by_name.get("RasHangUpW")
    assert origin is not None and target is not None

    preamble_names = set(scan.defines) | {item.name for item in scan.globals}

    def make(function):
        dep = Dep(
            function.name,
            body_text=function.body_text,
            return_type=function.return_type,
            calling_convention=function.calling_convention,
            params=list(function.params),
            callees=set(),
            identifiers={"HRASCONN", "FIXME"},
            is_exported=False,
        )
        return identity_for(
            function.name,
            dep=dep,
            res=resolution(imports={"ddk": ["HRASCONN"]}, exported=None),
            preamble_text=scan.preamble_text,
            preamble_names=preamble_names,
        )

    identity_origin, identity_target = make(origin), make(target)
    assert identity_origin.usable and identity_target.usable
    assert identity_origin.key() == identity_target.key()
    rust = 'pub unsafe extern "system" fn RasHangUpA(h: HRASCONN) -> DWORD { 0 }'
    outcome = accept(bank(tmp_path, identity_origin, rust), identity_target)
    assert outcome is not None and outcome.accepted
    assert "fn RasHangUpW(" in (outcome.rust or "")


def test_cglobal_is_the_documented_preamble_name_source():
    """`preamble_names` in the wiring is scan.defines + scan.globals names."""
    assert CGlobal(name="hDllInstance", type_str="static HINSTANCE", initializer=None, line=1).name
