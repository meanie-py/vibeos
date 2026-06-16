"""Delivery drain: self-filtering (by name AND user_id) and post-id dedup."""

import sqlite3

from conftest import FakeMM, post

from agent_swarm import queue


def test_self_filter_by_name_and_user_id(env):
    posts = [
        post("p1", 100, "board", "@tester please ack"),  # from another → delivered
        post("p2", 101, "tester", "@tester my own post"),  # self by name → filtered
        post("p3", 102, "RAWID", "@tester unresolved self", user_id="RAWID"),  # self by id
    ]
    r = queue.drain("tester", "tester", ["ch"], FakeMM(posts), self_ids=["tester", "RAWID"])
    assert [m.post_id for m in r.high] == ["p1"]
    assert r.high[0].sender == "board"


def test_post_id_dedup_survives_cursor_reset(env):
    posts = [post("p1", 100, "board", "@tester one"), post("p2", 101, "board", "@tester two")]
    mm = FakeMM(posts)
    first = queue.drain("tester", "tester", ["ch"], mm)
    assert {m.post_id for m in first.high} == {"p1", "p2"}

    # force re-exposure: reset the mention cursor to before the posts
    db = sqlite3.connect(env / "swarm.db")
    db.execute("UPDATE cursors SET mention_at=0 WHERE identity='tester'")
    db.commit()
    db.close()

    second = queue.drain("tester", "tester", ["ch"], mm)
    assert second.high == []  # already delivered → no re-delivery


def test_new_post_after_dedup_still_delivers(env):
    mm = FakeMM([post("p1", 100, "board", "@tester one")])
    queue.drain("tester", "tester", ["ch"], mm)
    mm.posts.append(post("p2", 200, "board", "@tester two"))
    r = queue.drain("tester", "tester", ["ch"], mm)
    assert [m.post_id for m in r.high] == ["p2"]


def test_low_priority_counted_not_delivered(env):
    posts = [post("p1", 100, "board", "@tester hi"), post("p2", 101, "board", "just chatter")]
    r = queue.drain("tester", "tester", ["ch"], FakeMM(posts))
    assert [m.post_id for m in r.high] == ["p1"]
    assert r.low_count == 1

    r2 = queue.drain("tester", "tester", ["ch"], FakeMM(posts), high_only=True)
    assert r2.low_count == 0  # high_only suppresses the count
