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
        pid = project_id or self.default_project
        params = {"sort_by": sort_by, "order_by": order}
        if not done:
            params["filter"] = "done = false"
        return http.get(f"{self.api}/projects/{pid}/tasks", token=self._t(), params=params) or []

    def get_task(self, task_id):
        return http.get(f"{self.api}/tasks/{task_id}", token=self._t())

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
