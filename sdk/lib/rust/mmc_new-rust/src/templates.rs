//! ATL container members mmc_new calls that `atlclasses-rust` does not (yet) have, written
//! against the container's PUBLIC surface (the node layout is private to that crate).
//!
//! `CAtlList<T>` has no `RemoveAt` / `RemoveTail` / `GetPrev` (its `FindPos`, atlcoll.h's
//! POSITION-returning `Find`, moved into atlclasses-rust). All three lists here hold raw
//! pointers (`T: Copy`), so
//! `RemoveAt` is a rebuild of the list without the node: O(n), and every POSITION taken before
//! the call is invalid afterwards (ATL keeps the others valid). The call sites only take a
//! fresh position after removing (UnregisterView, AddToRecentFiles, CAddDialog's moves).
//! The proper fix is `RemoveAt`/`RemoveTail`/`GetPrev` in atlclasses-rust.
use crate::prelude::*;

pub trait CAtlListExt<T: Copy> {
    /// atlcoll.h `RemoveAt(pos)`.
    unsafe fn RemoveAt(&mut self, pos: POSITION);
    /// atlcoll.h `RemoveTail()`: the removed element (a NULL-equivalent zero when empty).
    unsafe fn RemoveTail(&mut self) -> T;
    /// atlcoll.h `GetPrev(pos)`: the element AT `pos`, then `pos` moves to the previous node.
    unsafe fn GetPrev(&self, pos: &mut POSITION) -> *mut T;
}

impl<T: Copy> CAtlListExt<T> for CAtlList<T> {
    unsafe fn RemoveAt(&mut self, pos: POSITION) {
        let mut kept: CAtlList<T> = CAtlList::new();
        let mut it = self.GetHeadPosition();
        while !it.is_null() {
            let current = it;
            let element = *self.GetNext(&mut it);
            if current != pos {
                kept.AddTail(element);
            }
        }
        core::mem::swap(self, &mut kept);
        // `kept` (the old nodes) drops here; its elements are bit-copies held by `self` now.
    }

    unsafe fn RemoveTail(&mut self) -> T {
        let mut tail: POSITION = core::ptr::null_mut();
        let mut it = self.GetHeadPosition();
        while !it.is_null() {
            tail = it;
            self.GetNext(&mut it);
        }
        let element = if tail.is_null() {
            core::mem::zeroed()
        } else {
            *self.GetAt(tail)
        };
        if !tail.is_null() {
            self.RemoveAt(tail);
        }
        element
    }

    unsafe fn GetPrev(&self, pos: &mut POSITION) -> *mut T {
        let element = self.GetAt(*pos);
        let mut previous: POSITION = core::ptr::null_mut();
        let mut it = self.GetHeadPosition();
        while !it.is_null() && it != *pos {
            previous = it;
            self.GetNext(&mut it);
        }
        *pos = previous;
        element
    }
}
