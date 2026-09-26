"""Regression tests for the compiler-anchor source subset; no compiler or model calls."""

from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from rustconv import anchor_candidate, anchor_lower, profile_derive
from rustconv.profile_derive import ProfileDeriveError, derive


def windows_amd64_facts():
    return {
        "__SIZEOF_SHORT__": 2,
        "__SIZEOF_INT__": 4,
        "__SIZEOF_LONG__": 4,
        "__SIZEOF_LONG_LONG__": 8,
        "__SIZEOF_POINTER__": 8,
        "__SIZEOF_WCHAR_T__": 2,
        "__CHAR_BIT__": 8,
        "char_unsigned": False,
    }


class AnchorLowerTests(unittest.TestCase):
    def test_saved_abi_replay_requires_exact_candidate(self):
        with tempfile_dir() as temp:
            path = Path(temp) / "abi.json"
            raw = "pub type Long = ::core::ffi::c_long;\n"
            abi = {"arch": "amd64", "compiler_sha256": "a" * 64, "facts": windows_amd64_facts()}
            path.write_text(
                json.dumps(
                    {"raw_sha256": hashlib.sha256(raw.encode()).hexdigest(), "compiler_abi": abi}
                )
            )
            self.assertEqual(anchor_lower.replay_abi(path, raw), abi)
            with self.assertRaisesRegex(ValueError, "does not match"):
                anchor_lower.replay_abi(path, raw + "// changed")

    def test_integer_widths_follow_compiler_abi_not_host_rust_names(self):
        raw = """pub type S = ::core::ffi::c_short;
pub type I = ::core::ffi::c_int;
pub type L = ::core::ffi::c_long;
pub type LL = ::core::ffi::c_longlong;
pub type U = ::core::ffi::c_ulong;
pub type C = ::core::ffi::c_char;
pub type P = ::core::ffi::c_void;
pub type size_t = u64;
pub type ptrdiff_t = i64;
"""
        lowered, _ = anchor_lower.lower(raw, windows_amd64_facts())
        self.assertIn("pub type S = i16;", lowered)
        self.assertIn("pub type I = i32;", lowered)
        # Windows x64 C long remains 32 bits even though pointers are 64 bits.
        self.assertIn("pub type L = i32;", lowered)
        self.assertIn("pub type LL = i64;", lowered)
        self.assertIn("pub type U = u32;", lowered)
        self.assertIn("pub type C = i8;", lowered)
        self.assertIn("pub type P = ::core::ffi::c_void;", lowered)
        self.assertIn("pub type size_t = usize;", lowered)
        self.assertIn("pub type ptrdiff_t = isize;", lowered)

    def test_comments_and_string_literals_are_not_rewritten(self):
        raw = """// ::core::ffi::c_long and pub type size_t = u32;
const TEXT: &str = "::core::ffi::c_long; pub type size_t = u32;";
pub type size_t = u32;
pub type Long = ::core::ffi::c_long;
"""
        lowered, _ = anchor_lower.lower(raw, windows_amd64_facts())
        self.assertTrue(lowered.startswith("// ::core::ffi::c_long and pub type size_t = u32;\n"))
        self.assertIn('const TEXT: &str = "::core::ffi::c_long; pub type size_t = u32;";', lowered)
        self.assertIn("pub type size_t = usize;", lowered)
        self.assertIn("pub type Long = i32;", lowered)

    def test_unapproved_nightly_feature_is_refused(self):
        raw = "#![feature(generic_const_exprs)]\npub fn f() {}\n"
        with self.assertRaisesRegex(ValueError, "Unsupported nightly feature.*generic_const_exprs"):
            anchor_lower.lower(raw, windows_amd64_facts())


class AnchorCandidateTests(unittest.TestCase):
    def test_existing_output_is_refused_before_any_generation(self):
        with (
            patch.object(anchor_candidate.profile_derive, "derive") as derive,
            tempfile_dir() as temp,
        ):
            root = Path(temp)
            source = root / "x.c"
            module = root / "module"
            module.mkdir()
            output = root / "already"
            output.mkdir()
            (output / "request.json").write_text('{"raw_sha256":"pinned"}')
            with self.assertRaisesRegex(ValueError, "Existing compiler output"):
                anchor_candidate.generate(source, module, "amd64", output)
            derive.assert_not_called()

    def test_banked_prior_output_stops_generate_before_c2rust(self):
        with tempfile_dir() as temp:
            root = Path(temp)
            source = root / "dll" / "sample" / "math.c"
            source.parent.mkdir(parents=True)
            source.write_text("int Math(void) { return 1; }\n")
            module = source.parent
            output = root / "new-output"
            bank = root / "bank"
            (bank / "2026-attempt").mkdir(parents=True)
            meta = bank / "2026-attempt" / "meta.json"
            meta.write_text(json.dumps({"target": "sample", "label": "math.c"}))
            with (
                patch.object(anchor_candidate, "REACTOS_ROOT", root),
                patch.object(anchor_candidate, "discover_banks", return_value=([bank], [])),
                patch.object(
                    anchor_candidate.profile_derive,
                    "derive",
                    return_value=SimpleNamespace(errors=[], module=SimpleNamespace(name="sample")),
                ),
                patch.object(
                    anchor_candidate.shutil,
                    "which",
                    side_effect=AssertionError("must stop before c2rust lookup"),
                ),
                self.assertRaisesRegex(ValueError, "Existing translation must be reused"),
            ):
                anchor_candidate.generate(source, module, "amd64", output)
            self.assertFalse(output.exists())


class ProfileOwnershipTests(unittest.TestCase):
    def test_retained_selected_tu_resolves_to_its_real_target(self):
        with tempfile_dir() as temp:
            root = Path(temp)
            module = root / "dll" / "crt"
            module.mkdir(parents=True)
            retained = module / "math.c"
            retained.write_text("int Math(void) { return 0; }\n")
            (module / "main.c").write_text("int Main(void) { return 0; }\n")
            (module / "CMakeLists.txt").write_text(
                "list(APPEND SOURCE main.c\n # math.c # Rust replacement\n)\n"
                "add_library(crt MODULE ${SOURCE})\nset_module_type(crt win32dll)\n"
            )
            calls = []

            def capture(*args, **kwargs):
                calls.append(kwargs)
                return "profile"

            with patch.object(profile_derive, "derive", side_effect=capture):
                profile_derive.derive_source(retained, root, arch="amd64")
            self.assertEqual(len(calls), 1)
            self.assertEqual(calls[0]["target"], "crt")
            self.assertTrue(calls[0]["flipped_sources"])

    def test_selected_subset_filters_switch_audit_and_keeps_its_own_error_blocking(self):
        with tempfile_dir() as temp:
            root = Path(temp)
            _make_profile_tree(root)
            module = root / "dll" / "win32" / "widget"
            module.mkdir(parents=True)
            (module / "CMakeLists.txt").write_text(
                "list(APPEND SOURCE good.c bad.c)\n"
                "add_library(widget MODULE ${SOURCE})\n"
                "set_module_type(widget win32dll)\n"
            )
            good = module / "good.c"
            bad = module / "bad.c"
            good.write_text(
                "int good(int x) { switch (x) { case 1: return 1; default: return 0; } }\n"
            )
            bad.write_text(
                "int bad(int x) { switch (x) { case UNKNOWN_CASE: return 1; "
                "default: return 0; } }\n"
            )

            all_sources = derive(module, root, arch="amd64", use_compile_db=False)
            selected_good = derive(
                module, root, arch="amd64", use_compile_db=False, sources=(good,)
            )
            selected_bad = derive(module, root, arch="amd64", use_compile_db=False, sources=(bad,))

            self.assertEqual({p.name for p in selected_good.module.compiled_tus}, {"good.c"})
            self.assertTrue(any(f.code == "PARTIAL_TU_SCOPE" for f in selected_good.findings))
            self.assertTrue(any(f.code == "SWITCH_CONST_UNOWNED" for f in all_sources.errors))
            self.assertFalse(any(f.code == "SWITCH_CONST_UNOWNED" for f in selected_good.errors))
            self.assertTrue(
                any(f.code == "SWITCH_CONST_UNOWNED" for f in selected_bad.blocking_errors())
            )

    def test_subset_refuses_unowned_and_missing_sources(self):
        with tempfile_dir() as temp:
            root = Path(temp)
            _make_profile_tree(root)
            module = root / "dll" / "win32" / "widget"
            module.mkdir(parents=True)
            (module / "CMakeLists.txt").write_text(
                "add_library(widget MODULE listed.c missing.c)\nset_module_type(widget win32dll)\n"
            )
            listed = module / "listed.c"
            listed.write_text("int listed(void) { return 0; }\n")
            orphan = module / "orphan.c"
            orphan.write_text("int orphan(void) { return 0; }\n")
            missing = module / "missing.c"

            with self.assertRaisesRegex(ProfileDeriveError, "not proven members"):
                derive(module, root, arch="amd64", use_compile_db=False, sources=(orphan,))
            with self.assertRaisesRegex(ProfileDeriveError, "Selected source is missing"):
                derive(module, root, arch="amd64", use_compile_db=False, sources=(missing,))

    def test_selected_subset_drops_other_tus_dead_tu_error_but_keeps_own(self):
        with tempfile_dir() as temp:
            root = Path(temp)
            _make_profile_tree(root)
            module = root / "dll" / "win32" / "widget"
            module.mkdir(parents=True)
            (module / "CMakeLists.txt").write_text(
                "add_definitions(-DVERSION=0)\n"
                "add_library(widget MODULE live.c dead.c)\n"
                "set_module_type(widget win32dll)\n"
            )
            live = module / "live.c"
            live.write_text("int live(void) { return 0; }\n")
            dead = module / "dead.c"
            dead.write_text(
                "#if VERSION > 5\n"
                + "".join(f"/* provably inactive line {i} */\n" for i in range(20))
                + "int dead(void) { return 1; }\n#endif\n"
            )

            full = derive(module, root, arch="amd64", use_compile_db=False)
            selected_live = derive(
                module, root, arch="amd64", use_compile_db=False, sources=(live,)
            )
            selected_dead = derive(
                module, root, arch="amd64", use_compile_db=False, sources=(dead,)
            )
            dead_finding = next(f for f in full.errors if f.code == "DEAD_TU")

            self.assertEqual(dead_finding.subject, "dll/win32/widget/dead.c")
            self.assertEqual(selected_live.blocking_errors("dll/win32/widget/live.c"), [])
            self.assertEqual(
                [f.code for f in selected_live.deferred_errors("dll/win32/widget/live.c")],
                ["DEAD_TU"],
            )
            own_dead = selected_dead.blocking_errors("dll/win32/widget/dead.c")
            self.assertEqual([f.code for f in own_dead], ["DEAD_TU"])
            self.assertTrue(any(f.code == "PARTIAL_TU_SCOPE" for f in selected_live.findings))

    def test_source_scoped_blocking_keeps_other_tus_visible_but_nonblocking(self):
        module = profile_derive.CMakeModule(
            cmake=Path("CMakeLists.txt"),
            module_dir=Path("."),
            name="m",
            target_kind="library",
            module_type="kernelmodedriver",
            unicode=False,
        )
        dead = profile_derive.Finding(
            "error", "DEAD_TU", "dead file is inactive", "drivers/m/dead.c"
        )
        dp = profile_derive.DerivedProfile(
            module=module,
            env=None,
            closure=None,
            switch_consts=[],
            const_owners=[],
            islands=[],
            wrapper_inc=[],
            allow_files=[],
            extra_inc=[],
            ver_defs=[],
            findings=[dead],
        )
        self.assertEqual(dp.blocking_errors("drivers/m/dead.c"), [dead])
        self.assertEqual(dp.blocking_errors("drivers/m/live.c"), [])
        self.assertEqual(dp.deferred_errors("drivers/m/live.c"), [dead])
        self.assertEqual(dp.blocking_errors(), [dead])


def tempfile_dir():
    return tempfile.TemporaryDirectory()


def _make_profile_tree(root: Path) -> None:
    for rel in ("sdk/include", "sdk/include/psdk", "sdk/include/ddk", "sdk/include/reactos/idl"):
        (root / rel).mkdir(parents=True, exist_ok=True)


if __name__ == "__main__":
    unittest.main()
