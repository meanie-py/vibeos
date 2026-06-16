"""Tiny stdlib HTTP client — no third-party deps (keeps the tool worktree-shadow-safe).

One function, `request()`, with bearer/basic auth and JSON in/out. Backends (Mattermost,
Vikunja, Authentik) are thin wrappers over this.
"""

from __future__ import annotations

import base64
import json as _json
import urllib.error
import urllib.request
from typing import Any

from .errors import HttpError


def request(
    method: str,
    url: str,
    *,
    token: str | None = None,
    basic: str | None = None,
    json_body: Any = None,
    params: dict | None = None,
    headers: dict | None = None,
    timeout: float = 15.0,
) -> Any:
    """Perform an HTTP request and return parsed JSON (or None for empty bodies).

    token  → Authorization: Bearer <token>
    basic  → Authorization: Basic base64(<basic>)   (e.g. "user:pass")
    Raises HttpError on a non-2xx status.
    """
    if params:
        from urllib.parse import urlencode

        sep = "&" if "?" in url else "?"
        url = f"{url}{sep}{urlencode(params)}"

    hdrs = {"Accept": "application/json"}
    if headers:
        hdrs.update(headers)
    if token:
        hdrs["Authorization"] = f"Bearer {token}"
    elif basic:
        hdrs["Authorization"] = "Basic " + base64.b64encode(basic.encode()).decode()

    data = None
    if json_body is not None:
        data = _json.dumps(json_body).encode()
        hdrs["Content-Type"] = "application/json"

    req = urllib.request.Request(url, data=data, headers=hdrs, method=method.upper())
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read()
    except urllib.error.HTTPError as e:
        body = ""
        try:
            body = e.read().decode("utf-8", "ignore")
        except Exception:
            pass
        raise HttpError(e.code, url, body) from None
    except urllib.error.URLError as e:
        raise HttpError(0, url, str(e.reason)) from None

    if not raw:
        return None
    try:
        return _json.loads(raw)
    except ValueError:
        return raw.decode("utf-8", "ignore")


def get(url, **kw):
    return request("GET", url, **kw)


def post(url, **kw):
    return request("POST", url, **kw)


def put(url, **kw):
    return request("PUT", url, **kw)


def delete(url, **kw):
    return request("DELETE", url, **kw)
