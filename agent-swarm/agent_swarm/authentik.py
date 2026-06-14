"""Authentik client — the identity layer (agents = M2M service accounts).

Agents are service accounts under the `vibeos-swarm` group → a subgroup per human lead.
They authenticate machine-to-machine only (no interactive password) — the lockdown is
structural. The registrar token (akadmin) provisions; runtime M2M creds are derived from
the human lead's authority at login.
"""
from __future__ import annotations

from . import http
from .errors import HttpError


class AuthentikClient:
    def __init__(self, *, api, token, swarm_group="vibeos-swarm"):
        self.api = (api or "").rstrip("/")
        self.token = token
        self.swarm_group = swarm_group

    @classmethod
    def from_config(cls, cfg):
        return cls(api=cfg.authentik_api, token=cfg.authentik_registrar_token,
                   swarm_group=cfg.authentik_group)

    def _t(self):
        return self.token

    def me(self):
        return http.get(f"{self.api}/core/users/me/", token=self._t())

    # --- groups ----------------------------------------------------------
    def find_group(self, name):
        d = http.get(f"{self.api}/core/groups/", token=self._t(), params={"search": name})
        for g in (d or {}).get("results", []):
            if g["name"] == name:
                return g
        return None

    def create_group(self, name, *, parent=None, attributes=None):
        body = {"name": name}
        if parent:
            body["parent"] = parent
        if attributes:
            body["attributes"] = attributes
        return http.post(f"{self.api}/core/groups/", token=self._t(), json_body=body)

    def ensure_group(self, name, *, parent=None, attributes=None):
        return self.find_group(name) or self.create_group(name, parent=parent, attributes=attributes)

    # --- users / service accounts ---------------------------------------
    def find_user(self, username):
        d = http.get(f"{self.api}/core/users/", token=self._t(), params={"username": username})
        for u in (d or {}).get("results", []):
            if u["username"] == username:
                return u
        return None

    def create_service_account(self, username, *, expiring=False):
        """Create a service account (+ app-password token). Returns the API payload, which
        includes username, user_pk, user_uid, and token."""
        return http.post(f"{self.api}/core/users/service_account/", token=self._t(),
                         json_body={"name": username, "create_group": False,
                                    "expiring": expiring})

    def get_user(self, pk):
        return http.get(f"{self.api}/core/users/{pk}/", token=self._t())

    def add_user_to_group(self, group_pk, user_pk):
        """Authentik exposes group membership add directly."""
        try:
            return http.post(f"{self.api}/core/groups/{group_pk}/add_user/", token=self._t(),
                             json_body={"pk": int(user_pk)})
        except HttpError:
            # fallback: merge into the user's group list via PATCH
            u = self.get_user(user_pk)
            groups = list(u.get("groups", []))
            if group_pk not in groups:
                groups.append(group_pk)
            return http.request("PATCH", f"{self.api}/core/users/{user_pk}/",
                                token=self._t(), json_body={"groups": groups})

    def set_user_attributes(self, pk, attributes):
        return http.request("PATCH", f"{self.api}/core/users/{pk}/", token=self._t(),
                            json_body={"attributes": attributes})

    def delete_user(self, pk):
        return http.delete(f"{self.api}/core/users/{pk}/", token=self._t())

    def remove_user_from_group(self, group_pk, user_pk):
        return http.post(f"{self.api}/core/groups/{group_pk}/remove_user/", token=self._t(),
                         json_body={"pk": int(user_pk)})
