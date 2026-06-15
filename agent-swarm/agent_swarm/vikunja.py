"""Vikunja client — the task source of truth (epics=projects, quests=tasks, subtasks).

Vikunja's REST convention is unusual: **PUT = create, POST = update**. Encapsulated here
so callers never trip on it.
"""
from __future__ import annotations

from . import http


class VikunjaClient:
    def __init__(self, *, api, token, default_project=None):
        self.api = (api or "").rstrip("/")
        self.token = token
        self.default_project = default_project

    @classmethod
    def from_config(cls, cfg):
        return cls(api=cfg.vikunja_api, token=cfg.vikunja_token,
                   default_project=cfg.vikunja_project_id)

    def _t(self):
        return self.token

    # --- tasks (quests) --------------------------------------------------
    def list_tasks(self, project_id=None, *, done=False, sort_by="priority", order="desc"):
        """done=False → open only; done=None → all (open + done)."""
        pid = project_id or self.default_project
        params = {"sort_by": sort_by, "order_by": order}
        if done is False:
            params["filter"] = "done = false"
        return http.get(f"{self.api}/projects/{pid}/tasks", token=self._t(), params=params) or []

    # --- assignees + labels (routing signals for the bridge) -------------
    def assign(self, task_id, user_id):
        return http.put(f"{self.api}/tasks/{task_id}/assignees", token=self._t(),
                        json_body={"user_id": user_id})

    def unassign(self, task_id, user_id):
        return http.delete(f"{self.api}/tasks/{task_id}/assignees/{user_id}", token=self._t())

    def ensure_label(self, title):
        existing = http.get(f"{self.api}/labels", token=self._t()) or []
        for l in existing:
            if (l.get("title") or "").lower() == title.lower():
                return l
        return http.put(f"{self.api}/labels", token=self._t(), json_body={"title": title})

    def add_label(self, task_id, label_id):
        return http.put(f"{self.api}/tasks/{task_id}/labels", token=self._t(),
                        json_body={"label_id": label_id})

    def remove_label(self, task_id, label_id):
        return http.delete(f"{self.api}/tasks/{task_id}/labels/{label_id}", token=self._t())

    def get_task(self, task_id):
        return http.get(f"{self.api}/tasks/{task_id}", token=self._t())

    def search_tasks(self, query, project_id=None):
        pid = project_id or self.default_project
        return http.get(f"{self.api}/projects/{pid}/tasks", token=self._t(),
                        params={"s": query}) or []

    def create_task(self, title, *, project_id=None, priority=None, description=None, **fields):
        pid = project_id or self.default_project
        body = {"title": title, **fields}
        if priority is not None:
            body["priority"] = priority
        if description is not None:
            body["description"] = description
        return http.put(f"{self.api}/projects/{pid}/tasks", token=self._t(), json_body=body)

    def update_task(self, task_id, **fields):
        return http.post(f"{self.api}/tasks/{task_id}", token=self._t(), json_body=fields)

    def set_done(self, task_id, done=True):
        return self.update_task(task_id, done=done)

    def comment(self, task_id, text):
        return http.put(f"{self.api}/tasks/{task_id}/comments", token=self._t(),
                        json_body={"comment": text})

    def list_comments(self, task_id):
        return http.get(f"{self.api}/tasks/{task_id}/comments", token=self._t()) or []

    # --- projects (epics) ------------------------------------------------
    def get_project(self, project_id):
        return http.get(f"{self.api}/projects/{project_id}", token=self._t())

    def create_project(self, title, *, description="", **fields):
        return http.put(f"{self.api}/projects", token=self._t(),
                        json_body={"title": title, "description": description, **fields})

    def list_projects(self):
        return http.get(f"{self.api}/projects", token=self._t()) or []

    def delete_project(self, project_id):
        return http.delete(f"{self.api}/projects/{project_id}", token=self._t())
