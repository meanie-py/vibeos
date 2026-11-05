"""Compatibility API for native Rust artifact mechanisms; no Python algorithm fallback."""

from __future__ import annotations

if __package__:
    from .build_infra_compat import invoke as _invoke
else:
    from build_infra_compat import invoke as _invoke


def stable_read(path, expected=None):
    arguments = locals()
    return bytes(_invoke("artifact_economics_stable_read", arguments))


def marker_fields(payload):
    arguments = locals()
    return [bytes(field) for field in _invoke("artifact_economics_marker_fields", arguments)]


def map_bodies(report, repo):
    arguments = locals()
    result = _invoke("artifact_economics_map_bodies", arguments)
    mapping = result["mapping"]
    mapping["bodies"] = {name: mapping["bodies"][name] for name in result["body_order"]}
    return mapping


def size_model(rows):
    arguments = locals()
    return _invoke("artifact_economics_size_model", arguments)


def size_by_architecture(rows):
    arguments = locals()
    return _invoke("artifact_economics_size_by_architecture", arguments)


def summarize(mapping, repo, *, eligibility_top=0, cargo="cargo", scratch=None):
    arguments = locals()
    arguments["body_order"] = list(mapping["bodies"])
    return _invoke("artifact_economics_summarize", arguments)


def analyze(inventory, repo, *, eligibility_top=0, cargo="cargo", scratch=None):
    arguments = locals()
    return _invoke("artifact_economics_analyze", arguments)


def main():
    if __package__:
        from .build_infra_compat import main as native_main
    else:
        from build_infra_compat import main as native_main
    return native_main("artifact-economics")


if __name__ == "__main__":
    raise SystemExit(main())
