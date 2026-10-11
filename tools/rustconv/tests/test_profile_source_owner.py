from rustconv import profile_derive as profile


def test_partial_flip_audits_the_requested_retained_source(tmp_path, monkeypatch):
    (tmp_path / "pin.c").write_text("void Pin(void) {}")
    (tmp_path / "main.c").write_text("void Main(void) {}")
    (tmp_path / "CMakeLists.txt").write_text(
        "list(APPEND SOURCE\n main.c\n # pin.c # Rust replacement\n)\nadd_library(audio MODULE ${SOURCE})\nset_module_type(audio kernelmodedriver)\n"
    )
    calls = []
    monkeypatch.setattr(profile, "derive", lambda *a, **kw: calls.append(kw))
    profile.derive_source(tmp_path / "pin.c", tmp_path, arch="amd64")
    assert calls[0]["target"] == "audio"
    assert calls[0]["flipped_sources"] is True


def test_static_source_owner_wins_over_nearby_dll(tmp_path, monkeypatch):
    (tmp_path / "math.c").write_text("int Fn(void) {return 0;}")
    (tmp_path / "CMakeLists.txt").write_text(
        "add_library(crt MODULE generated.c)\nset_module_type(crt win32dll)\nadd_library(crt_static STATIC math.c)\n"
    )
    calls = []
    monkeypatch.setattr(profile, "derive", lambda *a, **kw: calls.append(kw))
    profile.derive_source(
        tmp_path / "math.c", tmp_path, arch="amd64", preferred_target="crt", usermode=True
    )
    assert calls[0]["target"] == "crt_static"
    assert calls[0]["usermode"] is True
