//! wined3d-h — wine/wined3d.h's `static inline` helpers, ported once.
//!
//! No library exports them (they are inline), so every client of wined3d -- ddraw, d3d8,
//! d3d9, the d3d1x front ends -- needs its own copy, and `port headers` rightly refuses to
//! buy a platform header's inlines per module. The three below touch only integers and are
//! plain functions. The other eight read wined3d's structs (`wined3d_private_store`,
//! `wined3d_streaming_buffer`, `wined3d_box`), and each client crate's island declares
//! those types itself -- nominally distinct per crate -- so they are a macro instead:
//! `wined3d_h::wined3d_inlines!();` in a client's prelude expands them against THAT crate's
//! types, from this one source. The names they use (`list_init`, `heap_alloc` -- wine/heap.h's
//! `HeapAlloc(GetProcessHeap(), 0, n)` --, `IsEqualGUID!`,
//! the WINED3D* constants) resolve at the expansion site, where the island and prelude
//! already provide them.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

/// wined3d.h `wined3d_log2i`: the GCC path, `__builtin_clz(x) ^ 0x1f`. C leaves x == 0
/// undefined there (its table path answers ~0u); this returns ~0u for it, as the table does.
#[inline]
pub fn wined3d_log2i(x: u32) -> u32 {
    if x == 0 {
        return !0u32;
    }
    x.leading_zeros() ^ 0x1f
}

/// wined3d.h `wined3d_bit_scan`: the lowest set bit's index, cleared from `*x`. The GCC path
/// is `__builtin_ffs(*x) - 1` then `*x ^= 1u << bit_offset`, a shift by -1 (undefined) when
/// `*x == 0`; here that returns -1 and leaves `*x` alone.
///
/// # Safety
/// `x` must point to a valid, writable `unsigned int`.
#[inline]
pub unsafe fn wined3d_bit_scan(x: *mut u32) -> i32 {
    let v = unsafe { *x };
    if v == 0 {
        return -1;
    }
    let bit = v.trailing_zeros();
    unsafe { *x = v ^ (1u32 << bit) };
    bit as i32
}

/// wined3d.h `wined3d_bound_range`: `start + count` does not overflow and is <= `limit`.
#[inline]
pub fn wined3d_bound_range(start: u32, count: u32, limit: u32) -> bool {
    start.checked_add(count).is_some_and(|sum| sum <= limit)
}

/// The struct-bound helpers of wine/wined3d.h, expanded into the calling crate. Invoke once,
/// in the prelude, after the island and winelist-rust are in scope.
#[macro_export]
macro_rules! wined3d_inlines {
    () => {
        /// wined3d.h `wined3d_private_store_init`.
        pub unsafe fn wined3d_private_store_init(store: *mut wined3d_private_store) {
            unsafe { list_init(core::ptr::addr_of_mut!((*store).content)) };
        }

        /// wined3d.h `wined3d_private_store_get_private_data`: LIST_FOR_EACH over the
        /// store, the entry whose tag is `tag`, or NULL.
        pub unsafe fn wined3d_private_store_get_private_data(
            store: *const wined3d_private_store,
            tag: *const GUID,
        ) -> *mut wined3d_private_data {
            unsafe {
                let head = core::ptr::addr_of!((*store).content) as *mut list;
                let mut entry = (*head).next;
                while entry != head {
                    let data = (entry as *mut u8)
                        .sub(core::mem::offset_of!(wined3d_private_data, entry))
                        as *mut wined3d_private_data;
                    if IsEqualGUID!(core::ptr::addr_of!((*data).tag), tag) {
                        return data;
                    }
                    entry = (*entry).next;
                }
                core::ptr::null_mut()
            }
        }

        /// wined3d.h `wined3d_private_store_free_private_data`.
        pub unsafe fn wined3d_private_store_free_private_data(
            _store: *mut wined3d_private_store,
            entry: *mut wined3d_private_data,
        ) {
            unsafe {
                if (*entry).flags & WINED3DSPD_IUNKNOWN as DWORD != 0 {
                    // IUnknown_Release(entry->content.object)
                    let obj = (*entry).content.object;
                    ((*(*obj).lpVtbl).Release.unwrap_unchecked())(obj);
                }
                list_remove(core::ptr::addr_of_mut!((*entry).entry));
                heap_free(entry as *mut core::ffi::c_void);
            }
        }

        /// wined3d.h `wined3d_private_store_cleanup`: LIST_FOR_EACH_SAFE, freeing each.
        pub unsafe fn wined3d_private_store_cleanup(store: *mut wined3d_private_store) {
            unsafe {
                let head = core::ptr::addr_of_mut!((*store).content);
                let mut e1 = (*head).next;
                while e1 != head {
                    let e2 = (*e1).next;
                    let data = (e1 as *mut u8)
                        .sub(core::mem::offset_of!(wined3d_private_data, entry))
                        as *mut wined3d_private_data;
                    wined3d_private_store_free_private_data(store, data);
                    e1 = e2;
                }
            }
        }

        /// wined3d.h `wined3d_private_store_set_private_data`.
        pub unsafe fn wined3d_private_store_set_private_data(
            store: *mut wined3d_private_store,
            guid: *const GUID,
            data: *const core::ffi::c_void,
            data_size: DWORD,
            flags: u32,
        ) -> HRESULT {
            unsafe {
                let mut ptr = data;
                if flags & WINED3DSPD_IUNKNOWN as u32 != 0 {
                    if data_size as usize != core::mem::size_of::<*mut IUnknown>() {
                        return WINED3DERR_INVALIDCALL;
                    }
                    // `ptr = &data`: the pointer VALUE is what gets stored.
                    ptr = core::ptr::addr_of!(data) as *const core::ffi::c_void;
                }
                // FIELD_OFFSET(struct wined3d_private_data, content.data[data_size])
                let bytes = core::mem::offset_of!(wined3d_private_data, content) + data_size as usize;
                let d = heap_alloc(bytes as _) as *mut wined3d_private_data;
                if d.is_null() {
                    return E_OUTOFMEMORY;
                }
                (*d).tag = *guid;
                (*d).flags = flags as _;
                (*d).size = data_size;
                core::ptr::copy_nonoverlapping(
                    ptr as *const u8,
                    core::ptr::addr_of_mut!((*d).content) as *mut u8,
                    data_size as usize,
                );
                if flags & WINED3DSPD_IUNKNOWN as u32 != 0 {
                    // IUnknown_AddRef(d->content.object)
                    let obj = (*d).content.object;
                    ((*(*obj).lpVtbl).AddRef.unwrap_unchecked())(obj);
                }
                let old = wined3d_private_store_get_private_data(store, guid);
                if !old.is_null() {
                    wined3d_private_store_free_private_data(store, old);
                }
                list_add_tail(
                    core::ptr::addr_of_mut!((*store).content),
                    core::ptr::addr_of_mut!((*d).entry),
                );
                WINED3D_OK
            }
        }

        /// wined3d.h `wined3d_streaming_buffer_init`.
        pub unsafe fn wined3d_streaming_buffer_init(
            buffer: *mut wined3d_streaming_buffer,
            bind_flags: core::ffi::c_uint,
        ) {
            unsafe {
                core::ptr::write_bytes(buffer, 0, 1);
                (*buffer).bind_flags = bind_flags;
            }
        }

        /// wined3d.h `wined3d_streaming_buffer_cleanup`.
        pub unsafe fn wined3d_streaming_buffer_cleanup(buffer: *mut wined3d_streaming_buffer) {
            unsafe {
                if !(*buffer).buffer.is_null() {
                    wined3d_buffer_decref((*buffer).buffer);
                }
                (*buffer).buffer = core::ptr::null_mut();
                (*buffer).pos = 0;
            }
        }

        /// wined3d.h `wined3d_box_set`.
        pub unsafe fn wined3d_box_set(
            box_: *mut wined3d_box,
            left: core::ffi::c_uint,
            top: core::ffi::c_uint,
            right: core::ffi::c_uint,
            bottom: core::ffi::c_uint,
            front: core::ffi::c_uint,
            back: core::ffi::c_uint,
        ) {
            unsafe {
                (*box_).left = left as _;
                (*box_).top = top as _;
                (*box_).right = right as _;
                (*box_).bottom = bottom as _;
                (*box_).front = front as _;
                (*box_).back = back as _;
            }
        }
    };
}
