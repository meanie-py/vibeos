from pathlib import Path

from rustconv.dup_census import function_hashes, run_census
from rustconv.scanner import scan_file


def _functions(tmp_path: Path, source: str):
    path = tmp_path / "sample.c"
    path.write_text(source)
    return path, scan_file(path).functions


def test_alpha_hash_matches_renamed_params_and_locals_but_exact_does_not(tmp_path):
    path, functions = _functions(
        tmp_path,
        """
        int first(int value) { int result = value + 1; return result; }
        int second(int input) { int output = input + 1; return output; }
        """,
    )
    first = function_hashes(functions[0], path.suffix)
    second = function_hashes(functions[1], path.suffix)
    assert first[0] != second[0]
    assert first[1] == second[1]
    assert first[2] and second[2]


def test_alpha_hash_preserves_external_callee_and_field_names(tmp_path):
    path, functions = _functions(
        tmp_path,
        """
        int a(OBJ *x) { int y = CallA(x->left); return y; }
        int b(OBJ *p) { int q = CallB(p->right); return q; }
        """,
    )
    assert (
        function_hashes(functions[0], path.suffix)[1]
        != function_hashes(functions[1], path.suffix)[1]
    )


def test_census_reports_exact_and_alpha_only_groups(tmp_path):
    source = """
    int a(int x) { return x + 1; }
    int b(int x) { return x + 1; }
    int c(int y) { return y + 1; }
    """
    path = tmp_path / "sample.c"
    path.write_text(source)
    report = run_census([tmp_path], tmp_path)
    assert report["functions"] == 3
    assert report["exact"]["reusable_copies"] == 1
    assert report["alpha"]["reusable_copies"] == 2
    assert report["alpha_only"]["groups"] == 1


def test_constructor_initializer_is_part_of_behavior_fingerprint(tmp_path):
    path = tmp_path / "sample.cpp"
    path.write_text("C::C() : value(1) {}\nD::D() : value(2) {}\nE::E() : value(1) {}\n")
    functions = scan_file(path).functions
    hashes = [function_hashes(function, path.suffix)[0] for function in functions]
    assert hashes[0] != hashes[1]
    assert hashes[0] == hashes[2]
