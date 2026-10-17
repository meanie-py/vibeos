"""fetch_pi_wifi_firmware: the pinned hashes decide, never the download."""

import hashlib
from pathlib import Path

import fetch_pi_wifi_firmware as fw
import pytest


def _serving(contents: dict[str, bytes]):
    calls = []

    def get(url: str) -> bytes:
        calls.append(url)
        return contents[url.removeprefix(fw.BASE)]

    return get, calls


def _pinned(monkeypatch, blobs: dict[str, bytes]) -> None:
    """Re-pin FILES to these contents' hashes, so the test needs no network."""
    files = [(up, name, hashlib.sha256(blobs[up]).hexdigest()) for up, name, _ in fw.FILES]
    monkeypatch.setattr(fw, "FILES", files)


def test_fetches_verifies_and_names_files_as_the_driver_loads_them(tmp_path: Path, monkeypatch):
    blobs = {up: f"blob {i}".encode() for i, (up, _, _) in enumerate(fw.FILES)}
    _pinned(monkeypatch, blobs)
    get, calls = _serving(blobs)
    out = fw.fetch(tmp_path, get)
    assert len(calls) == 3 and all("fetched" in line for line in out)
    want = blobs["cypress/cyfmac43455-sdio-standard.bin"]
    assert (tmp_path / "cyfmac43455-sdio.bin").read_bytes() == want
    # Second run: everything present with the right hash, nothing downloaded.
    get2, calls2 = _serving(blobs)
    assert all("present" in line for line in fw.fetch(tmp_path, get2)) and calls2 == []


def test_a_changed_upstream_file_fails_and_writes_nothing(tmp_path: Path, monkeypatch):
    blobs = {up: b"good" for up, _, _ in fw.FILES}
    _pinned(monkeypatch, blobs)
    tampered = dict(blobs)
    tampered["brcm/brcmfmac43455-sdio.txt"] = b"changed"
    get, _ = _serving(tampered)
    with pytest.raises(RuntimeError, match="pinned"):
        fw.fetch(tmp_path, get)
    assert not (tmp_path / "brcmfmac43455-sdio.txt").exists()


def test_a_wrong_cached_file_is_replaced(tmp_path: Path, monkeypatch):
    blobs = {up: f"x{i}".encode() for i, (up, _, _) in enumerate(fw.FILES)}
    _pinned(monkeypatch, blobs)
    (tmp_path / "cyfmac43455-sdio.clm_blob").write_bytes(b"stale")
    get, calls = _serving(blobs)
    fw.fetch(tmp_path, get)
    want = blobs["cypress/cyfmac43455-sdio.clm_blob"]
    assert (tmp_path / "cyfmac43455-sdio.clm_blob").read_bytes() == want
    assert len(calls) == 3


def test_the_real_pins_are_the_pi_5s_files():
    names = {name for _, name, _ in fw.FILES}
    assert names == {"cyfmac43455-sdio.bin", "cyfmac43455-sdio.clm_blob", "brcmfmac43455-sdio.txt"}
    assert all(len(h) == 64 for _, _, h in fw.FILES)
    assert len(fw.COMMIT) == 40
