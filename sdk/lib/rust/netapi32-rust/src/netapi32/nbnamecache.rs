use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _NBNameCacheNode {
    pub expireTime: u32,
    pub entry: *mut NBNameCacheEntry,
    pub next: *mut _NBNameCacheNode,
}
pub type NBNameCacheNode = _NBNameCacheNode;

#[repr(C)]
pub struct NBNameCache {
    pub heap: PVOID,
    pub cs: _RTL_CRITICAL_SECTION,
    pub entryExpireTimeMS: u32,
    pub head: *mut NBNameCacheNode,
}
unsafe extern "C" fn NBNameCacheUnlinkNode(
    cache: *mut NBNameCache,
    prev: *mut *mut NBNameCacheNode,
) {
    if !cache.is_null() && !prev.is_null() && !(*prev).is_null() {
        let next: *mut NBNameCacheNode = (**prev).next;
        HeapFree((*cache).heap, 0, (**prev).entry as *mut core::ffi::c_void);
        HeapFree((*cache).heap, 0, *prev as *mut core::ffi::c_void);
        *prev = next;
    }
}
/* Walks the list beginning with cache->head looking for the node with name `name`.
 * If found, returns a pointer to the `next` pointer of the node _prior_ to the found
 * node (or &head if head points to it) — dereference twice for the node itself.
 * Deletes expired nodes along the way. Returns NULL if the node isn't found.
 * Assumes the caller owns cache's lock. (translated from static NBNameCacheWalk) */
unsafe extern "C" fn NBNameCacheWalk(
    cache: *mut NBNameCache,
    name: *const core::ffi::c_char,
) -> *mut *mut NBNameCacheNode {
    let mut ret: *mut *mut NBNameCacheNode = core::ptr::null_mut();
    if !cache.is_null() && !(*cache).head.is_null() {
        let mut ptr: *mut *mut NBNameCacheNode = core::ptr::addr_of_mut!((*cache).head);
        while !ptr.is_null() && !(*ptr).is_null() && !(**ptr).entry.is_null() {
            if memcmp(
                core::ptr::addr_of!((*(**ptr).entry).name) as *const core::ffi::c_void,
                name as *const core::ffi::c_void,
                (NCBNAMSZ - 1) as usize,
            ) == 0
            {
                ret = ptr;
            } else if GetTickCount() > (**ptr).expireTime {
                NBNameCacheUnlinkNode(cache, ptr);
            }
            if !(*ptr).is_null() {
                ptr = core::ptr::addr_of_mut!((**ptr).next);
            }
        }
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn NBNameCacheCreate(mut heap: HANDLE, entryExpireTimeMS: DWORD) -> *mut NBNameCache {
    if heap.is_null() {
        heap = GetProcessHeap();
    }
    let cache = HeapAlloc(
        heap,
        0,
        core::mem::size_of::<NBNameCache>() as SIZE_T,
    ) as *mut NBNameCache;
    if !cache.is_null() {
        (*cache).heap = heap;
        InitializeCriticalSection(core::ptr::addr_of_mut!((*cache).cs));
        (*(*cache).cs.DebugInfo).__bindgen_anon_1.Spare[0] = b"nbnamecache.c: NBNameCache.cs\0".as_ptr() as DWORD_PTR;
        (*cache).entryExpireTimeMS = entryExpireTimeMS;
        (*cache).head = core::ptr::null_mut::<NBNameCacheNode>();
    }
    cache
}
#[no_mangle]
pub unsafe extern "C" fn NBNameCacheAddEntry(
    cache: *mut NBNameCache,
    entry: *mut NBNameCacheEntry,
) -> BOOL {
    let mut ret: BOOL;
    if cache.is_null() || entry.is_null() {
        ret = 0;
    } else {
        EnterCriticalSection(&raw mut (*cache).cs);
        let name_ptr =
            core::ptr::addr_of_mut!((*entry).name) as *mut u8 as *const ::core::ffi::c_char;
        let node_ptr = NBNameCacheWalk(cache, name_ptr);
        if !node_ptr.is_null() {
            let node = &mut *(*node_ptr);
            (*node).expireTime = GetTickCount().wrapping_add((*cache).entryExpireTimeMS);
            HeapFree((*cache).heap, 0u32, (*node).entry as *mut core::ffi::c_void);
            (*node).entry = entry;
            ret = 1;
        } else {
            let new_node = HeapAlloc(
                (*cache).heap,
                0u32,
                core::mem::size_of::<NBNameCacheNode>() as SIZE_T,
            ) as *mut NBNameCacheNode;
            if !new_node.is_null() {
                (*new_node).expireTime = GetTickCount().wrapping_add((*cache).entryExpireTimeMS);
                (*new_node).entry = entry;
                (*new_node).next = (*cache).head;
                (*cache).head = new_node;
                ret = 1;
            } else {
                ret = 0;
            }
        }
        LeaveCriticalSection(&raw mut (*cache).cs);
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn NBNameCacheFindEntry(
    cache: *mut NBNameCache,
    name: *const u8,
) -> *const NBNameCacheEntry {
    let mut printName: [u8; NCBNAMSZ as usize] = [0u8; NCBNAMSZ as usize];
    core::ptr::copy_nonoverlapping(name, printName.as_mut_ptr(), (NCBNAMSZ - 1) as usize);
    printName[(NCBNAMSZ - 1) as usize] = 0u8;

    let mut ret: *const NBNameCacheEntry = core::ptr::null();
    if !cache.is_null() {
        EnterCriticalSection(&raw mut (*cache).cs);
        let node: *mut *mut NBNameCacheNode = NBNameCacheWalk(cache, name as *const core::ffi::c_char);
        if !node.is_null() {
            ret = (**node).entry as *const NBNameCacheEntry;
        } else {
            ret = core::ptr::null();
        }
        LeaveCriticalSection(&raw mut (*cache).cs);
    } else {
        ret = core::ptr::null();
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn NBNameCacheDestroy(cache: *mut NBNameCache) {
    if !cache.is_null() {
        (*(*cache).cs.DebugInfo).__bindgen_anon_1.Spare[0] = 0;
        DeleteCriticalSection(&raw mut (*cache).cs);
        while !(*cache).head.is_null() {
            NBNameCacheUnlinkNode(cache, &raw mut (*cache).head);
        }
        HeapFree((*cache).heap, 0u32, cache as *mut core::ffi::c_void);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Copyright (c) 2003 Juan Lang
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  * This implementation uses a linked list, because I don't have a decent
//  * hash table implementation handy.  This is somewhat inefficient, but it's
//  * rather more efficient than not having a name cache at all.
//  */
//
// #include "netapi32.h"
//
// typedef struct _NBNameCacheNode
// {
//     DWORD expireTime;
//     NBNameCacheEntry *entry;
//     struct _NBNameCacheNode *next;
// } NBNameCacheNode;
//
// struct NBNameCache
// {
//     HANDLE heap;
//     CRITICAL_SECTION cs;
//     DWORD entryExpireTimeMS;
//     NBNameCacheNode *head;
// };
//
// /* Unlinks the node pointed to by *prev, and frees any associated memory.
//  * If that node's next pointed to another node, *prev now points to it.
//  * Assumes the caller owns cache's lock.
//  */
// static void NBNameCacheUnlinkNode(struct NBNameCache *cache,
//  NBNameCacheNode **prev)
// {
//     if (cache && prev && *prev)
//     {
//         NBNameCacheNode *next = (*prev)->next;
//
//         HeapFree(cache->heap, 0, (*prev)->entry);
//         HeapFree(cache->heap, 0, *prev);
//         *prev = next;
//     }
// }
//
// /* Walks the list beginning with cache->head looking for the node with name
//  * name.  If the node is found, returns a pointer to the next pointer of the
//  * node _prior_ to the found node (or head if head points to it).  Thus, if the
//  * node's all you want, dereference the return value twice.  If you want to
//  * modify the list, modify the referent of the return value.
//  * While it's at it, deletes nodes whose time has expired (except the node
//  * you're looking for, of course).
//  * Returns NULL if the node isn't found.
//  * Assumes the caller owns cache's lock.
//  */
// static NBNameCacheNode **NBNameCacheWalk(struct NBNameCache *cache,
//  const char name[NCBNAMSZ])
// {
//     NBNameCacheNode **ret = NULL;
//
//     if (cache && cache->head)
//     {
//         NBNameCacheNode **ptr;
//
//         ptr = &cache->head;
//         while (ptr && *ptr && (*ptr)->entry)
//         {
//             if (!memcmp((*ptr)->entry->name, name, NCBNAMSZ - 1))
//                 ret = ptr;
//             else
//             {
//                 if (GetTickCount() > (*ptr)->expireTime)
//                     NBNameCacheUnlinkNode(cache, ptr);
//             }
//             if (*ptr)
//                 ptr = &(*ptr)->next;
//         }
//     }
//     return ret;
// }
//
// struct NBNameCache *NBNameCacheCreate(HANDLE heap, DWORD entryExpireTimeMS)
// {
//     struct NBNameCache *cache;
//     
//     
//     if (!heap)
//         heap = GetProcessHeap();
//     cache = HeapAlloc(heap, 0, sizeof(struct NBNameCache));
//     if (cache)
//     {
//         cache->heap = heap;
//         InitializeCriticalSection(&cache->cs);
//         cache->cs.DebugInfo->Spare[0] = (DWORD_PTR)(__FILE__ ": NBNameCache.cs");
//         cache->entryExpireTimeMS = entryExpireTimeMS;
//         cache->head = NULL;
//     }
//     return cache;
// }
//
// BOOL NBNameCacheAddEntry(struct NBNameCache *cache, NBNameCacheEntry *entry)
// {
//     BOOL ret;
//
//     if (cache && entry)
//     {
//         NBNameCacheNode **node;
//
//         EnterCriticalSection(&cache->cs);
//         node = NBNameCacheWalk(cache, (char*)entry->name);
//         if (node)
//         {
//             (*node)->expireTime = GetTickCount() +
//              cache->entryExpireTimeMS;
//             HeapFree(cache->heap, 0, (*node)->entry);
//             (*node)->entry = entry;
//             ret = TRUE;
//         }
//         else
//         {
//             NBNameCacheNode *newNode = HeapAlloc(cache->heap, 0, sizeof(NBNameCacheNode));
//             if (newNode)
//             {
//                 newNode->expireTime = GetTickCount() +
//                  cache->entryExpireTimeMS;
//                 newNode->entry = entry;
//                 newNode->next = cache->head;
//                 cache->head = newNode;
//                 ret = TRUE;
//             }
//             else
//                 ret = FALSE;
//         }
//         LeaveCriticalSection(&cache->cs);
//     }
//     else
//         ret = FALSE;
//     return ret;
// }
//
// const NBNameCacheEntry *NBNameCacheFindEntry(struct NBNameCache *cache,
//  const UCHAR name[NCBNAMSZ])
// {
//     const NBNameCacheEntry *ret;
//     UCHAR printName[NCBNAMSZ];
//
//     memcpy(printName, name, NCBNAMSZ - 1);
//     printName[NCBNAMSZ - 1] = '\0';
//     if (cache)
//     {
//         NBNameCacheNode **node;
//
//         EnterCriticalSection(&cache->cs);
//         node = NBNameCacheWalk(cache, (const char *)name);
//         if (node)
//             ret = (*node)->entry;
//         else
//             ret = NULL;
//         LeaveCriticalSection(&cache->cs);
//     }
//     else
//         ret = NULL;
//     return ret;
// }
//
// void NBNameCacheDestroy(struct NBNameCache *cache)
// {
//     if (cache)
//     {
//         cache->cs.DebugInfo->Spare[0] = 0;
//         DeleteCriticalSection(&cache->cs);
//         while (cache->head)
//             NBNameCacheUnlinkNode(cache, &cache->head);
//         HeapFree(cache->heap, 0, cache);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
