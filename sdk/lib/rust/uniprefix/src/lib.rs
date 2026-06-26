//! `UNICODE_PREFIX_TABLE` — 1:1 Rust port of
//! `sdk/lib/rtl/unicodeprefix.c` (Alex Ionescu).
//!
//! This is an intrusive splay-tree-of-prefix-trees pointer graph
//! (`CONTAINING_RECORD`, circular `CaseMatch` lists, a `NextPrefixTree`
//! chain), not a slice algorithm — so it's a faithful unsafe
//! transliteration validated behaviorally, in the same spirit as the
//! sibling `splaytree`/`avltable` crates. The splay primitives are
//! reused from the `splaytree` crate (the genuine `RtlSplay`/
//! `RtlDelete`/`RtlRealSuccessor`); the `RtlLeftChild`-family macros
//! are inlined here exactly as in `ndk/rtlfuncs.h`.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

use core::mem::offset_of;
use core::ptr::null_mut;
use splaytree::SplayLinks as Links;

// NODE_TYPE_CODE constants (unicodeprefix.c, from the fastfat driver).
const PFX_NTC_TABLE: i16 = 0x0800;
const PFX_NTC_ROOT: i16 = 0x0801;
const PFX_NTC_CHILD: i16 = 0x0802;
const PFX_NTC_CASE_MATCH: i16 = 0x0803;

// RTL_GENERIC_COMPARE_RESULTS (+ the genuine `-1` "string is actually
// a prefix" sentinel, which callers treat as a non-less/greater
// match).
const GENERIC_LESS: i32 = 0;
const GENERIC_GREATER: i32 = 1;
const GENERIC_EQUAL: i32 = 2;

/// `UNICODE_STRING`.
#[repr(C)]
pub struct UnicodeString {
    pub length: u16,
    pub maximum_length: u16,
    pub buffer: *mut u16,
}

/// `UNICODE_PREFIX_TABLE_ENTRY`.
#[repr(C)]
pub struct PrefixEntry {
    pub node_type_code: i16,
    pub name_length: i16,
    pub next_prefix_tree: *mut PrefixEntry,
    pub case_match: *mut PrefixEntry,
    pub links: Links,
    pub prefix: *mut UnicodeString,
}

/// `UNICODE_PREFIX_TABLE`. Its first three fields are layout-compatible
/// with `PrefixEntry` (the genuine code casts a table pointer to an
/// entry pointer and only ever touches `NodeTypeCode`/`NameLength`/
/// `NextPrefixTree` through that alias).
#[repr(C)]
pub struct PrefixTable {
    pub node_type_code: i16,
    pub name_length: i16,
    pub next_prefix_tree: *mut PrefixEntry,
    pub last_next_entry: *mut PrefixEntry,
}

#[cfg(not(test))]
extern "system" {
    fn RtlpUpcaseUnicodeChar(c: u16) -> u16;
}

#[cfg(test)]
unsafe fn RtlpUpcaseUnicodeChar(c: u16) -> u16 {
    if (0x61..=0x7A).contains(&c) {
        c - 0x20
    } else {
        c
    }
}

// ── RtlLeftChild-family macros (ndk/rtlfuncs.h), inlined ───────────

#[inline]
unsafe fn sl_left(l: *mut Links) -> *mut Links {
    (*l).left_child
}
#[inline]
unsafe fn sl_right(l: *mut Links) -> *mut Links {
    (*l).right_child
}
#[inline]
unsafe fn sl_parent(l: *mut Links) -> *mut Links {
    (*l).parent
}
#[inline]
unsafe fn sl_is_root(l: *mut Links) -> bool {
    (*l).parent == l
}
#[inline]
unsafe fn sl_is_left_child(l: *mut Links) -> bool {
    (*(*l).parent).left_child == l
}
#[inline]
unsafe fn sl_init(l: *mut Links) {
    (*l).parent = l;
    (*l).left_child = null_mut();
    (*l).right_child = null_mut();
}
#[inline]
unsafe fn sl_insert_left(parent: *mut Links, child: *mut Links) {
    (*parent).left_child = child;
    (*child).parent = parent;
}
#[inline]
unsafe fn sl_insert_right(parent: *mut Links, child: *mut Links) {
    (*parent).right_child = child;
    (*child).parent = parent;
}

/// `CONTAINING_RECORD(links, UNICODE_PREFIX_TABLE_ENTRY, Links)`.
#[inline]
unsafe fn entry_of(l: *mut Links) -> *mut PrefixEntry {
    (l as *mut u8).sub(offset_of!(PrefixEntry, links)) as *mut PrefixEntry
}

#[inline]
unsafe fn table_as_entry(t: *mut PrefixTable) -> *mut PrefixEntry {
    t as *mut PrefixEntry
}

/// `ComputeUnicodeNameLength`.
#[no_mangle]
pub unsafe extern "system" fn ComputeUnicodeNameLength(
    unicode_name: *const UnicodeString,
) -> u32 {
    let chars = (*unicode_name).length as u32 / 2;
    let mut names_found: u32 = 1;
    let buf = (*unicode_name).buffer;
    let mut i: u32 = 0;
    while i < chars.wrapping_sub(1) {
        if *buf.add(i as usize) == b'\\' as u16 {
            names_found += 1;
        }
        i += 1;
    }
    names_found
}

/// `CompareUnicodeStrings` — returns `GenericLessThan/GreaterThan/
/// Equal` or the genuine `-1` prefix sentinel.
#[no_mangle]
pub unsafe extern "system" fn CompareUnicodeStrings(
    prefix: *const UnicodeString,
    string: *const UnicodeString,
    case_check_char: u32,
) -> i32 {
    // Raw-pointer element access (like the C): the genuine reads
    // `Buffer[i]` at `i == min(len)` — one past the counted length,
    // into the (NUL-terminated) buffer tail. Bounded slices would
    // spuriously panic there.
    let p = (*prefix).buffer;
    let s = (*string).buffer;
    let prefix_length = (*prefix).length as usize / 2;
    let string_length = (*string).length as usize / 2;
    let scan_length = string_length.min(prefix_length);

    if prefix_length == 1
        && *p == b'\\' as u16
        && string_length > 1
        && *s == b'\\' as u16
    {
        return -1;
    }

    let mut case_check = case_check_char as usize;
    if case_check > scan_length {
        case_check = scan_length;
    }

    let mut i = 0usize;
    while i < case_check {
        if *p.add(i) != *s.add(i) {
            break;
        }
        i += 1;
    }

    let mut found_prefix = *p.add(i);
    let mut found_string = *s.add(i);

    if i == case_check {
        // case-insensitive tail scan
        loop {
            found_prefix = *p.add(i);
            found_string = *s.add(i);
            if found_prefix != found_string {
                found_prefix = RtlpUpcaseUnicodeChar(found_prefix);
                found_string = RtlpUpcaseUnicodeChar(found_string);
                if found_prefix != found_string {
                    break;
                }
            }
            i += 1;
            if i >= scan_length {
                break;
            }
        }
    }

    if i < scan_length {
        if found_prefix == b'\\' as u16 {
            return GENERIC_LESS;
        }
        if found_string == b'\\' as u16 {
            return GENERIC_GREATER;
        }
        if found_prefix < found_string {
            return GENERIC_LESS;
        }
        return GENERIC_GREATER;
    }

    if prefix_length < string_length {
        if *s.add(prefix_length) == b'\\' as u16 {
            return -1;
        }
        return GENERIC_LESS;
    }
    if prefix_length > string_length {
        return GENERIC_GREATER;
    }
    GENERIC_EQUAL
}

/// `RtlInitializeUnicodePrefix`.
#[no_mangle]
pub unsafe extern "system" fn RtlInitializeUnicodePrefix(
    prefix_table: *mut PrefixTable,
) {
    (*prefix_table).name_length = 0;
    (*prefix_table).last_next_entry = null_mut();
    (*prefix_table).node_type_code = PFX_NTC_TABLE;
    (*prefix_table).next_prefix_tree = table_as_entry(prefix_table);
}

/// `RtlInsertUnicodePrefix`.
#[no_mangle]
pub unsafe extern "system" fn RtlInsertUnicodePrefix(
    prefix_table: *mut PrefixTable,
    prefix: *mut UnicodeString,
    prefix_table_entry: *mut PrefixEntry,
) -> u8 {
    let name_count = ComputeUnicodeNameLength(prefix) as i16;

    (*prefix_table_entry).name_length = name_count;
    (*prefix_table_entry).prefix = prefix;
    sl_init(&mut (*prefix_table_entry).links);

    let mut previous_entry = table_as_entry(prefix_table);
    let mut current_entry = (*previous_entry).next_prefix_tree;
    while (*current_entry).name_length > name_count {
        previous_entry = current_entry;
        current_entry = (*current_entry).next_prefix_tree;
    }

    if (*current_entry).name_length != name_count {
        (*previous_entry).next_prefix_tree = prefix_table_entry;
        (*prefix_table_entry).next_prefix_tree = current_entry;
        (*prefix_table_entry).node_type_code = PFX_NTC_ROOT;
        (*prefix_table_entry).case_match = prefix_table_entry;
        return 1;
    }

    let mut entry = current_entry;
    loop {
        let result = CompareUnicodeStrings((*entry).prefix, prefix, 0);
        if result == GENERIC_EQUAL {
            let mut next_entry = entry;
            loop {
                if CompareUnicodeStrings((*next_entry).prefix, prefix, u32::MAX)
                    == GENERIC_EQUAL
                {
                    return 0;
                }
                next_entry = (*next_entry).case_match;
                if next_entry == entry {
                    break;
                }
            }
            (*prefix_table_entry).node_type_code = PFX_NTC_CASE_MATCH;
            (*prefix_table_entry).next_prefix_tree = null_mut();
            (*prefix_table_entry).case_match = (*entry).case_match;
            (*entry).case_match = prefix_table_entry;
            break;
        }

        if result == GENERIC_GREATER {
            if !sl_left(&mut (*entry).links).is_null() {
                entry = entry_of(sl_left(&mut (*entry).links));
            } else {
                (*prefix_table_entry).node_type_code = PFX_NTC_CHILD;
                (*prefix_table_entry).next_prefix_tree = null_mut();
                (*prefix_table_entry).case_match = prefix_table_entry;
                sl_insert_left(
                    &mut (*entry).links,
                    &mut (*prefix_table_entry).links,
                );
                break;
            }
        } else if !sl_right(&mut (*entry).links).is_null() {
            entry = entry_of(sl_right(&mut (*entry).links));
        } else {
            (*prefix_table_entry).node_type_code = PFX_NTC_CHILD;
            (*prefix_table_entry).next_prefix_tree = null_mut();
            (*prefix_table_entry).case_match = prefix_table_entry;
            sl_insert_right(
                &mut (*entry).links,
                &mut (*prefix_table_entry).links,
            );
            break;
        }
    }

    let next_entry = (*current_entry).next_prefix_tree;
    (*current_entry).node_type_code = PFX_NTC_CHILD;
    (*current_entry).next_prefix_tree = null_mut();

    let root_links = splaytree::splay(&mut (*entry).links);
    let entry = entry_of(root_links);
    (*entry).node_type_code = PFX_NTC_ROOT;
    (*previous_entry).next_prefix_tree = entry;
    (*entry).next_prefix_tree = next_entry;
    1
}

/// `RtlFindUnicodePrefix`.
#[no_mangle]
pub unsafe extern "system" fn RtlFindUnicodePrefix(
    prefix_table: *mut PrefixTable,
    full_name: *mut UnicodeString,
    case_insensitive_index: u32,
) -> *mut PrefixEntry {
    let name_count = ComputeUnicodeNameLength(full_name) as i16;

    let mut previous_entry = table_as_entry(prefix_table);
    let mut current_entry = (*previous_entry).next_prefix_tree;
    while (*current_entry).name_length > name_count {
        previous_entry = current_entry;
        current_entry = (*current_entry).next_prefix_tree;
    }

    while (*current_entry).name_length != 0 {
        let mut splay_links: *mut Links = &mut (*current_entry).links;
        while !splay_links.is_null() {
            let entry = entry_of(splay_links);
            let result = CompareUnicodeStrings((*entry).prefix, full_name, 0);
            if result == GENERIC_GREATER {
                splay_links = sl_left(splay_links);
                continue;
            } else if result == GENERIC_LESS {
                splay_links = sl_right(splay_links);
                continue;
            }

            if case_insensitive_index == 0 {
                let entry = if (*entry).node_type_code == PFX_NTC_CHILD {
                    let next_entry = (*current_entry).next_prefix_tree;
                    (*current_entry).node_type_code = PFX_NTC_CHILD;
                    (*current_entry).next_prefix_tree = null_mut();
                    let nl = splaytree::splay(&mut (*entry).links);
                    let entry = entry_of(nl);
                    (*entry).node_type_code = PFX_NTC_ROOT;
                    (*previous_entry).next_prefix_tree = entry;
                    (*entry).next_prefix_tree = next_entry;
                    entry
                } else {
                    entry
                };
                return entry;
            }

            let mut next_entry = entry;
            loop {
                let r = CompareUnicodeStrings(
                    (*next_entry).prefix,
                    full_name,
                    case_insensitive_index,
                );
                if r != GENERIC_LESS && r != GENERIC_GREATER {
                    return next_entry;
                }
                next_entry = (*next_entry).case_match;
                if next_entry == entry {
                    break;
                }
            }
            break;
        }

        previous_entry = current_entry;
        current_entry = (*current_entry).next_prefix_tree;
    }

    null_mut()
}

/// `RtlNextUnicodePrefix`.
#[no_mangle]
pub unsafe extern "system" fn RtlNextUnicodePrefix(
    prefix_table: *mut PrefixTable,
    restart: u8,
) -> *mut PrefixEntry {
    let mut case_match_entry: *mut PrefixEntry = null_mut();
    if !(*prefix_table).last_next_entry.is_null() {
        case_match_entry = (*(*prefix_table).last_next_entry).case_match;
    }

    let entry: *mut PrefixEntry;
    if restart != 0 || (*prefix_table).last_next_entry.is_null() {
        let mut e = (*prefix_table).next_prefix_tree;
        if (*e).node_type_code == PFX_NTC_TABLE {
            return null_mut();
        }
        let mut splay_links: *mut Links = &mut (*e).links;
        while !sl_left(splay_links).is_null() {
            splay_links = sl_left(splay_links);
        }
        e = entry_of(splay_links);
        entry = e;
    } else if (*case_match_entry).node_type_code == PFX_NTC_CASE_MATCH {
        entry = case_match_entry;
    } else {
        let mut splay_links = splaytree::real_successor(&mut (*case_match_entry).links);
        if splay_links.is_null() {
            let mut sl: *mut Links =
                &mut (*(*prefix_table).last_next_entry).links;
            while !sl_is_root(sl) {
                sl = sl_parent(sl);
            }
            let mut e = entry_of(sl);
            e = (*e).next_prefix_tree;
            if (*e).name_length <= 0 {
                return null_mut();
            }
            while !sl_left(sl).is_null() {
                sl = sl_left(sl);
            }
            splay_links = sl;
        }
        entry = entry_of(splay_links);
    }

    (*prefix_table).last_next_entry = entry;
    entry
}

/// `RtlRemoveUnicodePrefix`.
#[no_mangle]
pub unsafe extern "system" fn RtlRemoveUnicodePrefix(
    prefix_table: *mut PrefixTable,
    prefix_table_entry: *mut PrefixEntry,
) {
    (*prefix_table).last_next_entry = null_mut();

    if (*prefix_table_entry).node_type_code == PFX_NTC_CASE_MATCH {
        let mut entry = (*prefix_table_entry).case_match;
        while (*entry).case_match != prefix_table_entry {
            entry = (*entry).case_match;
        }
        (*entry).case_match = (*prefix_table_entry).case_match;
    } else if (*prefix_table_entry).node_type_code == PFX_NTC_ROOT
        || (*prefix_table_entry).node_type_code == PFX_NTC_CHILD
    {
        if (*prefix_table_entry).case_match != prefix_table_entry {
            let mut entry = (*prefix_table_entry).case_match;
            while (*entry).case_match != prefix_table_entry {
                entry = (*entry).case_match;
            }
            (*entry).case_match = (*prefix_table_entry).case_match;

            (*entry).node_type_code = (*prefix_table_entry).node_type_code;
            (*entry).next_prefix_tree = (*prefix_table_entry).next_prefix_tree;
            // `Entry->Links = PrefixTableEntry->Links;` (struct copy)
            (*entry).links.parent = (*prefix_table_entry).links.parent;
            (*entry).links.left_child = (*prefix_table_entry).links.left_child;
            (*entry).links.right_child = (*prefix_table_entry).links.right_child;

            if sl_is_root(&mut (*prefix_table_entry).links) {
                (*entry).links.parent = &mut (*entry).links;
                let mut ref_entry = (*entry).next_prefix_tree;
                while (*ref_entry).next_prefix_tree != entry {
                    ref_entry = (*ref_entry).next_prefix_tree;
                }
                (*ref_entry).next_prefix_tree = entry;
            } else if sl_is_left_child(&mut (*prefix_table_entry).links) {
                (*sl_parent(&mut (*prefix_table_entry).links)).left_child =
                    &mut (*entry).links;
            } else {
                (*sl_parent(&mut (*prefix_table_entry).links)).right_child =
                    &mut (*entry).links;
            }

            if !sl_left(&mut (*entry).links).is_null() {
                (*sl_left(&mut (*entry).links)).parent = &mut (*entry).links;
            }
            if !sl_right(&mut (*entry).links).is_null() {
                (*sl_right(&mut (*entry).links)).parent = &mut (*entry).links;
            }
        } else {
            let mut sl: *mut Links = &mut (*prefix_table_entry).links;
            while !sl_is_root(sl) {
                sl = sl_parent(sl);
            }
            let entry = entry_of(sl);

            let splay_links = splaytree::delete(&mut (*prefix_table_entry).links);
            if splay_links.is_null() {
                let mut ref_entry = (*entry).next_prefix_tree;
                while (*ref_entry).next_prefix_tree != entry {
                    ref_entry = (*ref_entry).next_prefix_tree;
                }
                (*ref_entry).next_prefix_tree = (*entry).next_prefix_tree;
            } else if (&mut (*entry).links as *mut Links) != splay_links {
                let new_entry = entry_of(splay_links);
                let mut ref_entry = (*entry).next_prefix_tree;
                while (*ref_entry).next_prefix_tree != entry {
                    ref_entry = (*ref_entry).next_prefix_tree;
                }
                (*new_entry).node_type_code = PFX_NTC_ROOT;
                (*ref_entry).next_prefix_tree = new_entry;
                (*new_entry).next_prefix_tree = (*entry).next_prefix_tree;
                (*entry).node_type_code = PFX_NTC_CHILD;
                (*entry).next_prefix_tree = null_mut();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::boxed::Box;
    use std::vec::Vec;

    // NUL-terminated (real UNICODE_STRING convention): Length excludes
    // the NUL; CompareUnicodeStrings may read Buffer[min_len].
    fn us(s: &mut Vec<u16>) -> Box<UnicodeString> {
        let chars = s.len() - 1;
        Box::new(UnicodeString {
            length: (chars * 2) as u16,
            maximum_length: (s.len() * 2) as u16,
            buffer: s.as_mut_ptr(),
        })
    }
    fn w(s: &str) -> Vec<u16> {
        let mut v: Vec<u16> = s.encode_utf16().collect();
        v.push(0);
        v
    }
    fn entry() -> Box<PrefixEntry> {
        Box::new(PrefixEntry {
            node_type_code: 0,
            name_length: 0,
            next_prefix_tree: null_mut(),
            case_match: null_mut(),
            links: Links {
                parent: null_mut(),
                left_child: null_mut(),
                right_child: null_mut(),
            },
            prefix: null_mut(),
        })
    }

    #[test]
    fn insert_find_remove_next() {
        unsafe {
            let mut tbl = Box::new(PrefixTable {
                node_type_code: 0,
                name_length: 0,
                next_prefix_tree: null_mut(),
                last_next_entry: null_mut(),
            });
            RtlInitializeUnicodePrefix(&mut *tbl);
            assert!(RtlNextUnicodePrefix(&mut *tbl, 1).is_null());

            // Same name-count (one backslash ⇒ NameLength==2) so all
            // three live in ONE splay tree — RtlNextUnicodePrefix
            // iterates that via real_successor (well-defined). (Genuine
            // cross-tree iteration has a known repoint quirk we
            // faithfully replicate but don't assert.)
            let mut s1 = w("\\alpha");
            let mut s2 = w("\\bravo");
            let mut s3 = w("\\charlie");
            let p1 = us(&mut s1);
            let p2 = us(&mut s2);
            let p3 = us(&mut s3);
            let mut e1 = entry();
            let mut e2 = entry();
            let mut e3 = entry();
            assert_eq!(
                RtlInsertUnicodePrefix(&mut *tbl, &*p1 as *const _ as *mut _, &mut *e1),
                1
            );
            assert_eq!(
                RtlInsertUnicodePrefix(&mut *tbl, &*p2 as *const _ as *mut _, &mut *e2),
                1
            );
            assert_eq!(
                RtlInsertUnicodePrefix(&mut *tbl, &*p3 as *const _ as *mut _, &mut *e3),
                1
            );
            // duplicate insert must fail
            let mut s1b = w("\\alpha");
            let p1b = us(&mut s1b);
            let mut e1b = entry();
            assert_eq!(
                RtlInsertUnicodePrefix(&mut *tbl, &*p1b as *const _ as *mut _, &mut *e1b),
                0
            );

            // a full path under \bravo finds that prefix
            let mut q = w("\\bravo\\file.txt");
            let qs = us(&mut q);
            let found = RtlFindUnicodePrefix(&mut *tbl, &*qs as *const _ as *mut _, 1);
            assert!(!found.is_null());
            assert_eq!((*found).prefix, &*p2 as *const _ as *mut _);

            // \charlie path
            let mut q2 = w("\\charlie\\x");
            let qs2 = us(&mut q2);
            let f2 = RtlFindUnicodePrefix(&mut *tbl, &*qs2 as *const _ as *mut _, 1);
            assert!(!f2.is_null());
            assert_eq!((*f2).prefix, &*p3 as *const _ as *mut _);

            // unrelated path: no match
            let mut q3 = w("\\zeta\\nope");
            let qs3 = us(&mut q3);
            assert!(RtlFindUnicodePrefix(&mut *tbl, &*qs3 as *const _ as *mut _, 1).is_null());

            // iterate the single tree (well-defined)
            let mut seen = 0;
            let mut e = RtlNextUnicodePrefix(&mut *tbl, 1);
            while !e.is_null() && seen <= 10 {
                seen += 1;
                e = RtlNextUnicodePrefix(&mut *tbl, 0);
            }
            assert_eq!(seen, 3);

            // remove \bravo: its path no longer matches anything
            RtlRemoveUnicodePrefix(&mut *tbl, &mut *e2);
            let f3 = RtlFindUnicodePrefix(&mut *tbl, &*qs as *const _ as *mut _, 1);
            assert!(f3.is_null());
            // \charlie still resolves
            let f4 = RtlFindUnicodePrefix(&mut *tbl, &*qs2 as *const _ as *mut _, 1);
            assert!(!f4.is_null());
            assert_eq!((*f4).prefix, &*p3 as *const _ as *mut _);
        }
    }

    // The core purpose of the table — LONGEST-prefix match across prefixes of
    // DIFFERENT depth (different NameLength ⇒ separate splay trees in the
    // NextPrefixTree chain). The existing test only uses same-depth prefixes
    // (one tree); this exercises the multi-tree chain walk in
    // RtlFindUnicodePrefix and cross-depth RtlRemoveUnicodePrefix (which routes
    // through the splay delete fixed in e5ca403e206).
    #[test]
    fn nested_prefix_longest_match() {
        unsafe {
            let mut tbl = Box::new(PrefixTable {
                node_type_code: 0,
                name_length: 0,
                next_prefix_tree: null_mut(),
                last_next_entry: null_mut(),
            });
            RtlInitializeUnicodePrefix(&mut *tbl);
            let tp: *mut PrefixTable = &mut *tbl;

            // Backing stores must outlive the table (it holds raw pointers into
            // them). The inner heap buffers are stable across outer-Vec growth.
            let mut backs: Vec<Vec<u16>> = Vec::new();
            let mut uss: Vec<Box<UnicodeString>> = Vec::new();
            let mut ents: Vec<Box<PrefixEntry>> = Vec::new();
            let mut ent: Vec<*mut PrefixEntry> = Vec::new();

            for p in ["\\a", "\\a\\b", "\\a\\b\\c"] {
                backs.push(w(p));
                let u = us(backs.last_mut().unwrap());
                let uptr = &*u as *const _ as *mut UnicodeString;
                uss.push(u);
                ents.push(entry());
                let eptr = &mut **ents.last_mut().unwrap() as *mut PrefixEntry;
                assert_eq!(RtlInsertUnicodePrefix(tp, uptr, eptr), 1, "insert {p}");
                ent.push(eptr);
            }

            let mut qbacks: Vec<Vec<u16>> = Vec::new();
            let mut quss: Vec<Box<UnicodeString>> = Vec::new();
            let mut find = |query: &str| -> *mut PrefixEntry {
                qbacks.push(w(query));
                let u = us(qbacks.last_mut().unwrap());
                let uptr = &*u as *const _ as *mut UnicodeString;
                quss.push(u);
                RtlFindUnicodePrefix(tp, uptr, 1)
            };

            // longest registered ancestor wins
            assert_eq!(find("\\a\\b\\c\\d"), ent[2], "deepest prefix");
            assert_eq!(find("\\a\\b\\x"), ent[1]);
            assert_eq!(find("\\a\\x"), ent[0]);
            assert_eq!(find("\\a\\b\\c"), ent[2], "exact deepest");
            assert!(find("\\q\\r\\s").is_null(), "unrelated path");

            // remove the deepest → the next-longest ancestor now wins
            RtlRemoveUnicodePrefix(tp, ent[2]);
            assert_eq!(find("\\a\\b\\c\\d"), ent[1], "after removing \\a\\b\\c");
            // and again
            RtlRemoveUnicodePrefix(tp, ent[1]);
            assert_eq!(find("\\a\\b\\x"), ent[0], "after removing \\a\\b");
            assert_eq!(find("\\a\\b\\c\\d"), ent[0]);
        }
    }
}
