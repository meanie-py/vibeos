use crate::d3dwine::context::context_invalidate_compute_state;
use crate::d3dwine::context::context_invalidate_state;
use crate::d3dwine::buffer::wined3d_buffer_acquire_bo_for_write;
use crate::d3dwine::buffer::wined3d_buffer_invalidate_location;
use crate::d3dwine::buffer::wined3d_buffer_load;
use crate::d3dwine::buffer::wined3d_buffer_load_location;
use crate::d3dwine::buffer::wined3d_buffer_vk_barrier;
use crate::d3dwine::buffer::wined3d_buffer_vk_get_buffer_info;
use crate::d3dwine::context::wined3d_context_cleanup;
use crate::d3dwine::context::wined3d_context_init;
use crate::d3dwine::context::wined3d_stream_info_from_declaration;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const WINED3D_PERIODIC_SUBMIT_WORK_COUNT: u32 = 512;

pub const WINED3D_PERIODIC_SUBMIT_MAX_BUFFERS: u32 = 3;
#[no_mangle]
pub unsafe extern "C" fn vk_compare_op_from_wined3d(mut op: wined3d_cmp_func) -> VkCompareOp {
    match op as u32 {
        WINED3D_CMP_NEVER => {
            return VK_COMPARE_OP_NEVER;
        }
        WINED3D_CMP_LESS => {
            return VK_COMPARE_OP_LESS;
        }
        WINED3D_CMP_EQUAL => {
            return VK_COMPARE_OP_EQUAL;
        }
        WINED3D_CMP_LESSEQUAL => {
            return VK_COMPARE_OP_LESS_OR_EQUAL;
        }
        WINED3D_CMP_GREATER => {
            return VK_COMPARE_OP_GREATER;
        }
        WINED3D_CMP_NOTEQUAL => {
            return VK_COMPARE_OP_NOT_EQUAL;
        }
        WINED3D_CMP_GREATEREQUAL => {
            return VK_COMPARE_OP_GREATER_OR_EQUAL;
        }
        WINED3D_CMP_ALWAYS => {
            return VK_COMPARE_OP_ALWAYS;
        }
        _ => {
            if op as u32 == 0 {
                WARN!("Unhandled compare operation %#x.\n", op as u32);
            } else {
                FIXME!("Unhandled compare operation %#x.\n", op as u32);
            }
            return VK_COMPARE_OP_NEVER;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn vk_shader_stage_from_wined3d(shader_type: wined3d_shader_type) -> VkShaderStageFlagBits {
    match shader_type {
        WINED3D_SHADER_TYPE_VERTEX => {
            return VK_SHADER_STAGE_VERTEX_BIT;
        }
        WINED3D_SHADER_TYPE_HULL => {
            return VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT;
        }
        WINED3D_SHADER_TYPE_DOMAIN => {
            return VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT;
        }
        WINED3D_SHADER_TYPE_GEOMETRY => {
            return VK_SHADER_STAGE_GEOMETRY_BIT;
        }
        WINED3D_SHADER_TYPE_PIXEL => {
            return VK_SHADER_STAGE_FRAGMENT_BIT;
        }
        WINED3D_SHADER_TYPE_COMPUTE => {
            return VK_SHADER_STAGE_COMPUTE_BIT;
        }
        _ => {
            ERR!("Unhandled shader type %s.\n", debug_shader_type(shader_type));
            return 0;
        }
    }
}

unsafe extern "C" fn vk_blend_factor_from_wined3d(
    blend: wined3d_blend,
    dst_format: *const wined3d_format,
    alpha: bool,
) -> VkBlendFactor {
    match blend as ::core::ffi::c_uint {
        WINED3D_BLEND_ZERO => {
            return VK_BLEND_FACTOR_ZERO;
        }
        WINED3D_BLEND_ONE => {
            return VK_BLEND_FACTOR_ONE;
        }
        WINED3D_BLEND_SRCCOLOR => {
            return VK_BLEND_FACTOR_SRC_COLOR;
        }
        WINED3D_BLEND_INVSRCCOLOR => {
            return VK_BLEND_FACTOR_ONE_MINUS_SRC_COLOR;
        }
        WINED3D_BLEND_SRCALPHA => {
            return VK_BLEND_FACTOR_SRC_ALPHA;
        }
        WINED3D_BLEND_INVSRCALPHA => {
            return VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA;
        }
        WINED3D_BLEND_DESTALPHA => {
            if (*dst_format).alpha_size != 0 {
                return VK_BLEND_FACTOR_DST_ALPHA;
            }
            return VK_BLEND_FACTOR_ONE;
        }
        WINED3D_BLEND_INVDESTALPHA => {
            if (*dst_format).alpha_size != 0 {
                return VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA;
            }
            return VK_BLEND_FACTOR_ZERO;
        }
        WINED3D_BLEND_DESTCOLOR => {
            return VK_BLEND_FACTOR_DST_COLOR;
        }
        WINED3D_BLEND_INVDESTCOLOR => {
            return VK_BLEND_FACTOR_ONE_MINUS_DST_COLOR;
        }
        WINED3D_BLEND_SRCALPHASAT => {
            return VK_BLEND_FACTOR_SRC_ALPHA_SATURATE;
        }
        WINED3D_BLEND_BLENDFACTOR => {
            if alpha {
                return VK_BLEND_FACTOR_CONSTANT_ALPHA;
            }
            return VK_BLEND_FACTOR_CONSTANT_COLOR;
        }
        WINED3D_BLEND_INVBLENDFACTOR => {
            if alpha {
                return VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_ALPHA;
            }
            return VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR;
        }
        WINED3D_BLEND_SRC1COLOR => {
            return VK_BLEND_FACTOR_SRC1_COLOR;
        }
        WINED3D_BLEND_INVSRC1COLOR => {
            return VK_BLEND_FACTOR_ONE_MINUS_SRC1_COLOR;
        }
        WINED3D_BLEND_SRC1ALPHA => {
            return VK_BLEND_FACTOR_SRC1_ALPHA;
        }
        WINED3D_BLEND_INVSRC1ALPHA => {
            return VK_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA;
        }
        _ => {
            FIXME!("Unhandled blend %#x.\n", blend);
            return VK_BLEND_FACTOR_ZERO;
        }
    }
}

unsafe extern "C" fn vk_blend_op_from_wined3d(mut op: wined3d_blend_op) -> VkBlendOp {
    match op {
        WINED3D_BLEND_OP_ADD => {
            return VK_BLEND_OP_ADD;
        }
        WINED3D_BLEND_OP_SUBTRACT => {
            return VK_BLEND_OP_SUBTRACT;
        }
        WINED3D_BLEND_OP_REVSUBTRACT => {
            return VK_BLEND_OP_REVERSE_SUBTRACT;
        }
        WINED3D_BLEND_OP_MIN => {
            return VK_BLEND_OP_MIN;
        }
        WINED3D_BLEND_OP_MAX => {
            return VK_BLEND_OP_MAX;
        }
        _ => {
            FIXME!("Unhandled blend op %#x.\n", op);
            return VK_BLEND_OP_ADD;
        }
    }
}

unsafe extern "C" fn vk_colour_write_mask_from_wined3d(wined3d_mask: uint32_t) -> VkColorComponentFlags {
    let mut vk_mask: VkColorComponentFlags = 0 as VkColorComponentFlags;

    if wined3d_mask & WINED3DCOLORWRITEENABLE_RED as uint32_t != 0 {
        vk_mask |= VK_COLOR_COMPONENT_R_BIT as VkColorComponentFlags;
    }
    if wined3d_mask & WINED3DCOLORWRITEENABLE_GREEN as uint32_t != 0 {
        vk_mask |= VK_COLOR_COMPONENT_G_BIT as VkColorComponentFlags;
    }
    if wined3d_mask & WINED3DCOLORWRITEENABLE_BLUE as uint32_t != 0 {
        vk_mask |= VK_COLOR_COMPONENT_B_BIT as VkColorComponentFlags;
    }
    if wined3d_mask & WINED3DCOLORWRITEENABLE_ALPHA as uint32_t != 0 {
        vk_mask |= VK_COLOR_COMPONENT_A_BIT as VkColorComponentFlags;
    }

    return vk_mask;
}

unsafe extern "C" fn vk_cull_mode_from_wined3d(mode: wined3d_cull) -> VkCullModeFlags {
    match mode as u32 {
        WINED3D_CULL_NONE => {
            return VK_CULL_MODE_NONE;
        }
        WINED3D_CULL_FRONT => {
            return VK_CULL_MODE_FRONT_BIT;
        }
        WINED3D_CULL_BACK => {
            return VK_CULL_MODE_BACK_BIT;
        }
        _ => {
            FIXME!("Unhandled cull mode %#x.\n", mode as u32);
            return VK_CULL_MODE_NONE;
        }
    }
}

unsafe extern "C" fn vk_topology_from_wined3d(t: wined3d_primitive_type) -> VkPrimitiveTopology {
    match t as ::core::ffi::c_uint {
        WINED3D_PT_POINTLIST => return VK_PRIMITIVE_TOPOLOGY_POINT_LIST,
        WINED3D_PT_LINELIST => return VK_PRIMITIVE_TOPOLOGY_LINE_LIST,
        WINED3D_PT_LINESTRIP => return VK_PRIMITIVE_TOPOLOGY_LINE_STRIP,
        WINED3D_PT_TRIANGLELIST => return VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
        WINED3D_PT_TRIANGLESTRIP => return VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP,
        WINED3D_PT_TRIANGLEFAN => return VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN,
        WINED3D_PT_LINELIST_ADJ => return VK_PRIMITIVE_TOPOLOGY_LINE_LIST_WITH_ADJACENCY,
        WINED3D_PT_LINESTRIP_ADJ => return VK_PRIMITIVE_TOPOLOGY_LINE_STRIP_WITH_ADJACENCY,
        WINED3D_PT_TRIANGLELIST_ADJ => return VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST_WITH_ADJACENCY,
        WINED3D_PT_TRIANGLESTRIP_ADJ => return VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP_WITH_ADJACENCY,
        WINED3D_PT_PATCH => return VK_PRIMITIVE_TOPOLOGY_PATCH_LIST,
        WINED3D_PT_UNDEFINED => {}
        _ => {
            FIXME!(
                "Unhandled primitive type %s.\n",
                debug_d3dprimitivetype(t)
            );
        }
    }
    return (!(0 as ::core::ffi::c_uint)) as VkPrimitiveTopology;
}

unsafe extern "C" fn vk_topology_class_from_wined3d(t: wined3d_primitive_type) -> VkPrimitiveTopology {
    match t as ::core::ffi::c_uint {
        WINED3D_PT_POINTLIST => return VK_PRIMITIVE_TOPOLOGY_POINT_LIST,
        WINED3D_PT_LINELIST | WINED3D_PT_LINELIST_ADJ | WINED3D_PT_LINESTRIP | WINED3D_PT_LINESTRIP_ADJ => {
            return VK_PRIMITIVE_TOPOLOGY_LINE_LIST;
        }
        WINED3D_PT_TRIANGLEFAN | WINED3D_PT_TRIANGLELIST | WINED3D_PT_TRIANGLELIST_ADJ | WINED3D_PT_TRIANGLESTRIP | WINED3D_PT_TRIANGLESTRIP_ADJ => {
            return VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;
        }
        WINED3D_PT_PATCH => return VK_PRIMITIVE_TOPOLOGY_PATCH_LIST,
        WINED3D_PT_UNDEFINED => return !0 as VkPrimitiveTopology,
        _ => {
            FIXME!("Unhandled primitive type %s.\n", debug_d3dprimitivetype(t));
            return !0 as VkPrimitiveTopology;
        }
    }
}

unsafe extern "C" fn vk_stencil_op_from_wined3d(mut op: wined3d_stencil_op) -> VkStencilOp {
    match op {
        WINED3D_STENCIL_OP_KEEP => {
            return VK_STENCIL_OP_KEEP;
        }
        WINED3D_STENCIL_OP_ZERO => {
            return VK_STENCIL_OP_ZERO;
        }
        WINED3D_STENCIL_OP_REPLACE => {
            return VK_STENCIL_OP_REPLACE;
        }
        WINED3D_STENCIL_OP_INCR_SAT => {
            return VK_STENCIL_OP_INCREMENT_AND_CLAMP;
        }
        WINED3D_STENCIL_OP_DECR_SAT => {
            return VK_STENCIL_OP_DECREMENT_AND_CLAMP;
        }
        WINED3D_STENCIL_OP_INVERT => {
            return VK_STENCIL_OP_INVERT;
        }
        WINED3D_STENCIL_OP_INCR => {
            return VK_STENCIL_OP_INCREMENT_AND_WRAP;
        }
        WINED3D_STENCIL_OP_DECR => {
            return VK_STENCIL_OP_DECREMENT_AND_WRAP;
        }
        _ => {
            if op as u64 == 0 {
                WARN!("Unhandled stencil operation %#x.\n", op as u32);
            } else {
                FIXME!("Unhandled stencil operation %#x.\n", op as u32);
            }
            return VK_STENCIL_OP_KEEP;
        }
    }
}

unsafe extern "C" fn wined3d_get_unused_stream_index(state: *const wined3d_state, index: *mut u32) -> bool {
    let mut i: u32 = 0;
    while (i as usize) < ARRAY_SIZE!((*state).streams) as usize {
        if (*state).streams[i as usize].buffer.is_null() {
            *index = i;
            return true;
        }
        i += 1;
    }

    return false;
}

unsafe extern "C" fn wined3d_allocator_chunk_vk_lock(chunk_vk: *mut wined3d_allocator_chunk_vk) {
    wined3d_device_vk_allocator_lock(wined3d_device_vk_from_allocator((*chunk_vk).c.allocator as *mut wined3d_allocator));
}

unsafe extern "C" fn wined3d_allocator_chunk_vk_unlock(mut chunk_vk: *mut wined3d_allocator_chunk_vk) {
    wined3d_device_vk_allocator_unlock(wined3d_device_vk_from_allocator(
        (*chunk_vk).c.allocator as *mut wined3d_allocator,
    ));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_allocator_chunk_vk_map(chunk_vk: *mut wined3d_allocator_chunk_vk, context_vk: *mut wined3d_context_vk) -> *mut ::core::ffi::c_void {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut map_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut vr: VkResult;

    TRACE!(
        "chunk %p, memory 0x%s, map_ptr %p.\n",
        chunk_vk,
        wine_dbgstr_longlong((*chunk_vk).vk_memory as ULONGLONG),
        (*chunk_vk).c.map_ptr
    );

    wined3d_allocator_chunk_vk_lock(chunk_vk);

    if (*chunk_vk).c.map_ptr.is_null() {
        vr = ((*vk_info).vk_ops.vkMapMemory.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*chunk_vk).vk_memory,
            0,
            (VK_WHOLE_SIZE) as u64,
            0,
            core::ptr::addr_of_mut!((*chunk_vk).c.map_ptr),
        );
        if vr < 0 {
            ERR!(
                "Failed to map chunk memory, vr %s.\n",
                wined3d_debug_vkresult(vr)
            );
            wined3d_allocator_chunk_vk_unlock(chunk_vk);
            return core::ptr::null_mut();
        }

        adapter_adjust_mapped_memory((*device_vk).d.adapter, (WINED3D_ALLOCATOR_CHUNK_SIZE) as ssize_t);
    }

    (*chunk_vk).c.map_count = (*chunk_vk).c.map_count.wrapping_add(1);
    map_ptr = (*chunk_vk).c.map_ptr;

    wined3d_allocator_chunk_vk_unlock(chunk_vk);

    return map_ptr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_allocator_chunk_vk_unmap(chunk_vk: *mut wined3d_allocator_chunk_vk, context_vk: *mut wined3d_context_vk) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;

    wined3d_allocator_chunk_vk_lock(chunk_vk);

    (*chunk_vk).c.map_count = (*chunk_vk).c.map_count.wrapping_sub(1);
    if (*chunk_vk).c.map_count != 0 {
        wined3d_allocator_chunk_vk_unlock(chunk_vk);
        return;
    }

    (*vk_info)
        .vk_ops
        .vkUnmapMemory
        .unwrap_unchecked()((*device_vk).vk_device, (*chunk_vk).vk_memory);
    (*chunk_vk).c.map_ptr = core::ptr::null_mut();

    wined3d_allocator_chunk_vk_unlock(chunk_vk);

    adapter_adjust_mapped_memory((*device_vk).d.adapter, (-(WINED3D_ALLOCATOR_CHUNK_SIZE as i32)) as isize);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_allocate_vram_chunk_memory(context_vk: *mut wined3d_context_vk, pool: ::core::ffi::c_uint, size: size_t) -> VkDeviceMemory {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut allocate_info: VkMemoryAllocateInfo = core::mem::zeroed();
    let mut vk_memory: VkDeviceMemory = core::mem::zeroed();
    let mut vr: VkResult;

    allocate_info.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    allocate_info.pNext = core::ptr::null();
    allocate_info.allocationSize = size as VkDeviceSize;
    allocate_info.memoryTypeIndex = pool;
    vr = ((*vk_info).vk_ops.vkAllocateMemory.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(allocate_info),
        core::ptr::null(),
        core::ptr::addr_of_mut!(vk_memory),
    );
    if vr < 0 {
        ERR!(
            "Failed to allocate memory, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return (VK_NULL_HANDLE) as VkDeviceMemory;
    }

    return vk_memory;
}

unsafe extern "C" fn wined3d_context_vk_allocate_memory(
    context_vk: *mut wined3d_context_vk,
    memory_type: ::core::ffi::c_uint,
    size: VkDeviceSize,
    vk_memory: *mut VkDeviceMemory,
) -> *mut wined3d_allocator_block {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let allocator: *mut wined3d_allocator = core::ptr::addr_of_mut!((*device_vk).allocator);
    let mut block: *mut wined3d_allocator_block = core::ptr::null_mut::<wined3d_allocator_block>();

    wined3d_device_vk_allocator_lock(device_vk);

    if size > (WINED3D_ALLOCATOR_CHUNK_SIZE / 2) as VkDeviceSize {
        *vk_memory = wined3d_context_vk_allocate_vram_chunk_memory(context_vk, memory_type, size as size_t);
        wined3d_device_vk_allocator_unlock(device_vk);
        return core::ptr::null_mut::<wined3d_allocator_block>();
    }

    block = wined3d_allocator_allocate(
        allocator,
        core::ptr::addr_of_mut!((*context_vk).c),
        memory_type,
        size as size_t,
    );
    if block.is_null() {
        wined3d_device_vk_allocator_unlock(device_vk);
        *vk_memory = VK_NULL_HANDLE as VkDeviceMemory;
        return core::ptr::null_mut::<wined3d_allocator_block>();
    }

    *vk_memory = (*wined3d_allocator_chunk_vk((*block).chunk)).vk_memory;

    wined3d_device_vk_allocator_unlock(device_vk);
    return block;
}

unsafe extern "C" fn wined3d_context_vk_free_memory(
    mut context_vk: *mut wined3d_context_vk,
    mut block: *mut wined3d_allocator_block,
) {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);

    assert!((*(*block).chunk).allocator == core::ptr::addr_of_mut!((*device_vk).allocator));
    wined3d_device_vk_allocator_lock(device_vk);
    wined3d_allocator_block_free(block);
    wined3d_device_vk_allocator_unlock(device_vk);
}

unsafe extern "C" fn wined3d_context_vk_create_slab_bo(
    context_vk: *mut wined3d_context_vk,
    size: VkDeviceSize,
    usage: VkBufferUsageFlags,
    memory_type: VkMemoryPropertyFlags,
    bo: *mut wined3d_bo_vk,
) -> bool {
    let adapter_vk: *const wined3d_adapter_vk = wined3d_adapter_vk((*(*context_vk).c.device).adapter);
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let limits: *const VkPhysicalDeviceLimits = core::ptr::addr_of!((*adapter_vk).device_limits);
    let mut key: wined3d_bo_slab_vk_key = wined3d_bo_slab_vk_key {
        memory_type: 0,
        usage: 0,
        size: 0,
    };
    let mut slab: *mut wined3d_bo_slab_vk = core::ptr::null_mut::<wined3d_bo_slab_vk>();
    let mut entry: *mut wine_rb_entry = core::ptr::null_mut::<wine_rb_entry>();
    let mut object_size: usize;
    let mut idx: usize;
    let mut alignment: usize;

    if size > (WINED3D_ALLOCATOR_MIN_BLOCK_SIZE / 2) as VkDeviceSize {
        return false;
    }

    alignment = WINED3D_SLAB_BO_MIN_OBJECT_ALIGN as usize;
    if (usage & (VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT | VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT)) != 0
        && (*limits).minTexelBufferOffsetAlignment > alignment as VkDeviceSize
    {
        alignment = (*limits).minTexelBufferOffsetAlignment as usize;
    }
    if (usage & VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT) != 0 && (*limits).minUniformBufferOffsetAlignment != 0 {
        alignment = (*limits).minUniformBufferOffsetAlignment as usize;
    }
    if (usage & VK_BUFFER_USAGE_STORAGE_BUFFER_BIT) != 0 && (*limits).minStorageBufferOffsetAlignment != 0 {
        alignment = (*limits).minStorageBufferOffsetAlignment as usize;
    }

    object_size = (size.wrapping_add((alignment as VkDeviceSize).wrapping_sub(1))
        & !((alignment as VkDeviceSize).wrapping_sub(1))) as usize;
    if object_size < (WINED3D_ALLOCATOR_MIN_BLOCK_SIZE / 32) as usize {
        object_size = (WINED3D_ALLOCATOR_MIN_BLOCK_SIZE / 32) as usize;
    }
    key.memory_type = memory_type;
    key.usage = usage;
    key.size = (32 as usize).wrapping_mul(object_size) as VkDeviceSize;

    wined3d_device_vk_allocator_lock(device_vk);

    entry = wine_rb_get(
        core::ptr::addr_of_mut!((*context_vk).bo_slab_available),
        core::ptr::addr_of_mut!(key) as *const core::ffi::c_void,
    );
    if !entry.is_null() {
        slab = WINE_RB_ENTRY_VALUE!(entry, wined3d_bo_slab_vk, entry);
    } else {
        slab = calloc(
            (1 as usize) as _,
            (core::mem::size_of::<wined3d_bo_slab_vk>()) as _,
        ) as *mut wined3d_bo_slab_vk;
        if slab.is_null() {
            wined3d_device_vk_allocator_unlock(device_vk);
            ERR!("Failed to allocate bo slab.\n");
            return false;
        }

        if wine_rb_put(
            core::ptr::addr_of_mut!((*context_vk).bo_slab_available),
            core::ptr::addr_of_mut!(key) as *const core::ffi::c_void,
            core::ptr::addr_of_mut!((*slab).entry),
        ) < 0
        {
            wined3d_device_vk_allocator_unlock(device_vk);
            ERR!("Failed to add slab to available tree.\n");
            free(slab as *mut core::ffi::c_void);
            return false;
        }

        (*slab).requested_memory_type = memory_type;
        if wined3d_context_vk_create_bo(
            context_vk,
            key.size,
            usage,
            memory_type,
            core::ptr::addr_of_mut!((*slab).bo),
        ) == 0
        {
            wined3d_device_vk_allocator_unlock(device_vk);
            ERR!("Failed to create slab bo.\n");
            wine_rb_remove(
                core::ptr::addr_of_mut!((*context_vk).bo_slab_available),
                core::ptr::addr_of_mut!((*slab).entry),
            );
            free(slab as *mut core::ffi::c_void);
            return false;
        }
        (*slab).map = !(0 as uint32_t);
    }

    idx = wined3d_bit_scan(core::ptr::addr_of_mut!((*slab).map)) as usize;
    if (*slab).map == 0 {
        if !(*slab).next.is_null() {
            wine_rb_replace(
                core::ptr::addr_of_mut!((*context_vk).bo_slab_available),
                core::ptr::addr_of_mut!((*slab).entry),
                core::ptr::addr_of_mut!((*(*slab).next).entry),
            );
            (*slab).next = core::ptr::null_mut::<wined3d_bo_slab_vk>();
        } else {
            wine_rb_remove(
                core::ptr::addr_of_mut!((*context_vk).bo_slab_available),
                core::ptr::addr_of_mut!((*slab).entry),
            );
        }
    }

    wined3d_device_vk_allocator_unlock(device_vk);

    *bo = (*slab).bo;
    (*bo).memory = core::ptr::null_mut::<wined3d_allocator_block>();
    (*bo).slab = slab;
    (*bo).b.refcount = 1 as uint8_t;
    (*bo).b.client_map_count = 0 as core::ffi::c_uint;
    (*bo).b.map_ptr = core::ptr::null_mut();
    (*bo).b.buffer_offset = idx.wrapping_mul(object_size) as _;
    (*bo).b.memory_offset = (*slab)
        .bo
        .b
        .memory_offset
        .wrapping_add((*bo).b.buffer_offset);
    (*bo).size = size;
    list_init(core::ptr::addr_of_mut!((*bo).b.users));
    (*bo).command_buffer_id = 0 as uint64_t;
    (*bo).host_synced = false;

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_create_bo(context_vk: *mut wined3d_context_vk, size: VkDeviceSize, usage: VkBufferUsageFlags, memory_type: VkMemoryPropertyFlags, bo: *mut wined3d_bo_vk) -> BOOL {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut memory_requirements: VkMemoryRequirements = core::mem::zeroed();
    let mut adapter_vk: *mut wined3d_adapter_vk = core::ptr::null_mut();
    let mut create_info: VkBufferCreateInfo = core::mem::zeroed();
    let mut memory_type_idx: ::core::ffi::c_uint = 0;
    let mut vr: VkResult = 0;

    if wined3d_context_vk_create_slab_bo(context_vk, size, usage, memory_type, bo) {
        return (TRUE) as BOOL;
    }

    adapter_vk = wined3d_adapter_vk((*device_vk).d.adapter);

    create_info.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
    create_info.pNext = core::ptr::null();
    create_info.flags = 0;
    create_info.size = size;
    create_info.usage = usage;
    create_info.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    create_info.queueFamilyIndexCount = 0;
    create_info.pQueueFamilyIndices = core::ptr::null();

    vr = ((*vk_info).vk_ops.vkCreateBuffer.unwrap_unchecked())((*device_vk).vk_device, &raw mut create_info, core::ptr::null(), &raw mut (*bo).vk_buffer);
    if (vr as i32) < 0 {
        ERR!("Failed to create Vulkan buffer, vr %s.\n", wined3d_debug_vkresult(vr));
        return (FALSE) as BOOL;
    }

    ((*vk_info).vk_ops.vkGetBufferMemoryRequirements.unwrap_unchecked())((*device_vk).vk_device, (*bo).vk_buffer, &raw mut memory_requirements);

    memory_type_idx = wined3d_adapter_vk_get_memory_type_index(adapter_vk,
            memory_requirements.memoryTypeBits, memory_type);
    if memory_type_idx == !(0 as ::core::ffi::c_uint) {
        ERR!("Failed to find suitable memory type.\n");
        ((*vk_info).vk_ops.vkDestroyBuffer.unwrap_unchecked())((*device_vk).vk_device, (*bo).vk_buffer, core::ptr::null());
        return (FALSE) as BOOL;
    }
    (*bo).memory = wined3d_context_vk_allocate_memory(context_vk,
            memory_type_idx, memory_requirements.size, &raw mut (*bo).vk_memory);
    if (*bo).vk_memory == 0 {
        ERR!("Failed to allocate buffer memory.\n");
        ((*vk_info).vk_ops.vkDestroyBuffer.unwrap_unchecked())((*device_vk).vk_device, (*bo).vk_buffer, core::ptr::null());
        return (FALSE) as BOOL;
    }
    (*bo).b.memory_offset = if !(*bo).memory.is_null() {
        (*(*bo).memory).offset
    } else {
        0 as size_t
    };

    vr = ((*vk_info).vk_ops.vkBindBufferMemory.unwrap_unchecked())((*device_vk).vk_device, (*bo).vk_buffer,
            (*bo).vk_memory, (*bo).b.memory_offset as VkDeviceSize);
    if (vr as i32) < 0 {
        ERR!("Failed to bind buffer memory, vr %s.\n", wined3d_debug_vkresult(vr));
        if !(*bo).memory.is_null() {
            wined3d_context_vk_free_memory(context_vk, (*bo).memory);
        } else {
            ((*vk_info).vk_ops.vkFreeMemory.unwrap_unchecked())((*device_vk).vk_device, (*bo).vk_memory, core::ptr::null());
        }
        ((*vk_info).vk_ops.vkDestroyBuffer.unwrap_unchecked())((*device_vk).vk_device, (*bo).vk_buffer, core::ptr::null());
        return (FALSE) as BOOL;
    }

    (*bo).b.refcount = 1 as uint8_t;
    (*bo).b.client_map_count = 0 as ::core::ffi::c_uint;
    (*bo).b.map_ptr = core::ptr::null_mut();
    (*bo).b.buffer_offset = 0 as size_t;
    (*bo).size = size;
    (*bo).usage = usage;
    (*bo).memory_type =
        (*adapter_vk).memory_properties.memoryTypes[memory_type_idx as usize].propertyFlags;
    (*bo).b.coherent = (((*bo).memory_type & VK_MEMORY_PROPERTY_HOST_COHERENT_BIT) != 0);
    list_init(&raw mut (*bo).b.users);
    (*bo).command_buffer_id = 0 as uint64_t;
    (*bo).slab = core::ptr::null_mut();
    (*bo).host_synced = false;

    TRACE!(
        "Created buffer 0x%s, memory 0x%s for bo %p.\n",
        wine_dbgstr_longlong((*bo).vk_buffer as ULONGLONG),
        wine_dbgstr_longlong((*bo).vk_memory as ULONGLONG),
        bo,
    );

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_create_image(context_vk: *mut wined3d_context_vk, vk_image_type: VkImageType, usage: VkImageUsageFlags, vk_format: VkFormat, width: ::core::ffi::c_uint, height: ::core::ffi::c_uint, depth: ::core::ffi::c_uint, sample_count: ::core::ffi::c_uint, mip_levels: ::core::ffi::c_uint, layer_count: ::core::ffi::c_uint, flags: ::core::ffi::c_uint, image: *mut wined3d_image_vk) -> BOOL {
    let adapter_vk: *mut wined3d_adapter_vk = wined3d_adapter_vk((*(*context_vk).c.device).adapter);
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut memory_requirements: VkMemoryRequirements = core::mem::zeroed();
    let mut create_info: VkImageCreateInfo = core::mem::zeroed();
    let mut memory_type_idx: ::core::ffi::c_uint = 0;
    let mut vr: VkResult = core::mem::zeroed();

    create_info.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
    create_info.pNext = core::ptr::null();
    create_info.flags = flags;
    create_info.imageType = vk_image_type;
    create_info.format = vk_format;
    create_info.extent.width = width;
    create_info.extent.height = height;
    create_info.extent.depth = depth;
    create_info.mipLevels = mip_levels;
    create_info.arrayLayers = layer_count;
    create_info.samples = sample_count;
    create_info.tiling = VK_IMAGE_TILING_OPTIMAL;
    create_info.usage = usage;
    create_info.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    create_info.queueFamilyIndexCount = 0;
    create_info.pQueueFamilyIndices = core::ptr::null();
    create_info.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;

    (*image).command_buffer_id = 0;

    vr = ((*vk_info).vk_ops.vkCreateImage.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(create_info),
        core::ptr::null(),
        core::ptr::addr_of_mut!((*image).vk_image),
    );
    if vr != VK_SUCCESS {
        ERR!("Failed to create image, vr %s.\n", wined3d_debug_vkresult(vr));
        (*image).vk_image = (VK_NULL_HANDLE as _);
        return (FALSE) as BOOL;
    }

    ((*vk_info).vk_ops.vkGetImageMemoryRequirements.unwrap_unchecked())(
        (*device_vk).vk_device,
        (*image).vk_image,
        core::ptr::addr_of_mut!(memory_requirements),
    );

    memory_type_idx = wined3d_adapter_vk_get_memory_type_index(
        adapter_vk,
        memory_requirements.memoryTypeBits,
        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
    );
    if memory_type_idx == !0 {
        ERR!("Failed to find suitable image memory type.\n");
        ((*vk_info).vk_ops.vkDestroyImage.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*image).vk_image,
            core::ptr::null(),
        );
        (*image).vk_image = (VK_NULL_HANDLE as _);
        return (FALSE) as BOOL;
    }

    (*image).memory = wined3d_context_vk_allocate_memory(
        context_vk,
        memory_type_idx,
        memory_requirements.size,
        core::ptr::addr_of_mut!((*image).vk_memory),
    );
    if (*image).vk_memory == 0 {
        ERR!("Failed to allocate image memory.\n");
        ((*vk_info).vk_ops.vkDestroyImage.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*image).vk_image,
            core::ptr::null(),
        );
        (*image).vk_image = (VK_NULL_HANDLE as _);
        return (FALSE) as BOOL;
    }

    vr = ((*vk_info).vk_ops.vkBindImageMemory.unwrap_unchecked())(
        (*device_vk).vk_device,
        (*image).vk_image,
        (*image).vk_memory,
        if !(*image).memory.is_null() {
            ((*(*image).memory).offset) as _
        } else {
            0
        },
    );
    if vr != VK_SUCCESS {
        ((*vk_info).vk_ops.vkDestroyImage.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*image).vk_image,
            core::ptr::null(),
        );
        if !(*image).memory.is_null() {
            wined3d_context_vk_free_memory(context_vk, (*image).memory);
        } else {
            ((*vk_info).vk_ops.vkFreeMemory.unwrap_unchecked())(
                (*device_vk).vk_device,
                (*image).vk_memory,
                core::ptr::null(),
            );
        }
        ERR!("Failed to bind image memory, vr %s.\n", wined3d_debug_vkresult(vr));
        (*image).memory = core::ptr::null_mut();
        (*image).vk_memory = (VK_NULL_HANDLE as _);
        (*image).vk_image = (VK_NULL_HANDLE as _);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_context_vk_get_retired_object_vk(
    mut context_vk: *mut wined3d_context_vk,
) -> *mut wined3d_retired_object_vk {
    let retired: *mut wined3d_retired_objects_vk = core::ptr::addr_of_mut!((*context_vk).retired);
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if !(*retired).free.is_null() {
        o = (*retired).free;
        (*retired).free = (*o).u.next;
        return o;
    }

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*retired).objects) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*retired).size),
        (*retired).count.wrapping_add(1 as _),
        core::mem::size_of::<wined3d_retired_object_vk>() as SIZE_T,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    let fresh22 = (*retired).count;
    (*retired).count = (*retired).count.wrapping_add(1);
    return (*retired).objects.offset(fresh22 as isize) as *mut wined3d_retired_object_vk;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_vk_framebuffer(context_vk: *mut wined3d_context_vk, vk_framebuffer: VkFramebuffer, command_buffer_id: uint64_t) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        ((*vk_info).vk_ops.vkDestroyFramebuffer.unwrap_unchecked())(
            (*device_vk).vk_device,
            vk_framebuffer,
            core::ptr::null(),
        );
        return;
    }

    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!(
            "Leaking framebuffer 0x%s.\n",
            wine_dbgstr_longlong(vk_framebuffer)
        );
        return;
    }

    (*o).type_ = WINED3D_RETIRED_FRAMEBUFFER_VK;
    (*o).u.vk_framebuffer = vk_framebuffer;
    (*o).command_buffer_id = command_buffer_id;
}

unsafe extern "C" fn wined3d_context_vk_return_vk_descriptor_pool(
    context_vk: *mut wined3d_context_vk,
    vk_descriptor_pool: VkDescriptorPool,
) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*context_vk).vk_descriptor_pools) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*context_vk).vk_descriptor_pools_size),
        (*context_vk).vk_descriptor_pool_count + 1,
        (core::mem::size_of::<VkDescriptorPool>()) as SIZE_T,
    ) == 0
    {
        ((*vk_info).vk_ops.vkDestroyDescriptorPool.unwrap_unchecked())(
            (*device_vk).vk_device,
            vk_descriptor_pool,
            core::ptr::null(),
        );

        return;
    }

    ((*vk_info).vk_ops.vkResetDescriptorPool.unwrap_unchecked())(
        (*device_vk).vk_device,
        vk_descriptor_pool,
        0,
    );
    let fresh = (*context_vk).vk_descriptor_pool_count;
    (*context_vk).vk_descriptor_pool_count += 1;
    *(*context_vk).vk_descriptor_pools.add(fresh as usize) = vk_descriptor_pool;
}

unsafe extern "C" fn wined3d_context_vk_reset_vk_descriptor_pool(
    mut context_vk: *mut wined3d_context_vk,
    mut vk_descriptor_pool: VkDescriptorPool,
    mut command_buffer_id: uint64_t,
) {
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut::<wined3d_retired_object_vk>();
    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        wined3d_context_vk_return_vk_descriptor_pool(context_vk, vk_descriptor_pool);
        return;
    }
    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!(
            "Leaking descriptor pool 0x%s.\n",
            wine_dbgstr_longlong(vk_descriptor_pool as u64)
        );
        return;
    }
    (*o).type_ = WINED3D_RETIRED_DESCRIPTOR_POOL_VK;
    (*o).u.vk_descriptor_pool = vk_descriptor_pool;
    (*o).command_buffer_id = command_buffer_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_vk_memory(
    mut context_vk: *mut wined3d_context_vk,
    mut vk_memory: VkDeviceMemory,
    mut command_buffer_id: uint64_t,
) {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut::<wined3d_retired_object_vk>();
    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        ((*vk_info).vk_ops.vkFreeMemory.unwrap_unchecked())(
            (*device_vk).vk_device,
            vk_memory,
            core::ptr::null::<VkAllocationCallbacks>(),
        );
        return;
    }
    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!(
            "Leaking memory 0x%s.\n",
            wine_dbgstr_longlong(vk_memory as ULONGLONG)
        );
        return;
    }
    (*o).type_ = WINED3D_RETIRED_MEMORY_VK;
    (*o).u.vk_memory = vk_memory;
    (*o).command_buffer_id = command_buffer_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_allocator_block(
    mut context_vk: *mut wined3d_context_vk,
    mut block: *mut wined3d_allocator_block,
    mut command_buffer_id: uint64_t,
) {
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        wined3d_context_vk_free_memory(context_vk, block);
        return;
    }

    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!("Leaking block %p.\n", block);
        return;
    }

    (*o).type_ = WINED3D_RETIRED_ALLOCATOR_BLOCK_VK;
    (*o).u.block = block;
    (*o).command_buffer_id = command_buffer_id;
}

unsafe extern "C" fn wined3d_bo_slab_vk_free_slice(
    slab: *mut wined3d_bo_slab_vk,
    idx: SIZE_T,
    context_vk: *mut wined3d_context_vk,
) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let mut key: wined3d_bo_slab_vk_key = core::mem::zeroed();
    let mut entry: *mut wine_rb_entry = core::ptr::null_mut();

    TRACE!(
        "slab %p, idx %Iu, context_vk %p.\n",
        slab,
        idx,
        context_vk
    );

    wined3d_device_vk_allocator_lock(device_vk);

    if (*slab).map == 0 {
        key.memory_type = (*slab).requested_memory_type;
        key.usage = (*slab).bo.usage;
        key.size = (*slab).bo.size;

        entry = wine_rb_get(
            core::ptr::addr_of_mut!((*context_vk).bo_slab_available),
            core::ptr::addr_of_mut!(key) as *const core::ffi::c_void,
        );
        if !entry.is_null() {
            (*slab).next = (entry as *mut u8)
                .offset(-(core::mem::offset_of!(wined3d_bo_slab_vk, entry) as isize))
                as *mut wined3d_bo_slab_vk;
            wine_rb_replace(
                core::ptr::addr_of_mut!((*context_vk).bo_slab_available),
                entry,
                core::ptr::addr_of_mut!((*slab).entry),
            );
        } else if (wine_rb_put(
            core::ptr::addr_of_mut!((*context_vk).bo_slab_available),
            core::ptr::addr_of_mut!(key) as *const core::ffi::c_void,
            core::ptr::addr_of_mut!((*slab).entry),
        ) < (0 as i32))
        {
            ERR!(
                "Unable to return slab %p (map 0x%08x) to available tree.\n",
                slab,
                (*slab).map
            );
        }
    }
    (*slab).map |= (1 as u32) << idx;

    wined3d_device_vk_allocator_unlock(device_vk);
}

unsafe extern "C" fn wined3d_context_vk_destroy_bo_slab_slice(
    mut context_vk: *mut wined3d_context_vk,
    mut slab: *mut wined3d_bo_slab_vk,
    mut idx: SIZE_T,
    mut command_buffer_id: uint64_t,
) {
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();
    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        wined3d_bo_slab_vk_free_slice(slab, idx, context_vk);
        return;
    }
    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!("Leaking slab %p, slice %#Ix.\n", slab, idx);
        return;
    }
    (*o).type_ = WINED3D_RETIRED_BO_SLAB_SLICE_VK;
    (*o).u.slice.slab = slab;
    (*o).u.slice.idx = idx as size_t;
    (*o).command_buffer_id = command_buffer_id;
}

unsafe extern "C" fn wined3d_context_vk_destroy_vk_buffer(
    mut context_vk: *mut wined3d_context_vk,
    mut vk_buffer: VkBuffer,
    mut command_buffer_id: uint64_t,
) {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        ((*vk_info).vk_ops.vkDestroyBuffer.unwrap_unchecked())((*device_vk).vk_device, vk_buffer, core::ptr::null());
        return;
    }

    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!("Leaking buffer 0x%s.\n", wine_dbgstr_longlong(vk_buffer as u64));
        return;
    }

    (*o).type_ = WINED3D_RETIRED_BUFFER_VK;
    (*o).u.vk_buffer = vk_buffer;
    (*o).command_buffer_id = command_buffer_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_vk_image(
    context_vk: *mut wined3d_context_vk,
    vk_image: VkImage,
    command_buffer_id: uint64_t,
) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        ((*vk_info).vk_ops.vkDestroyImage.unwrap_unchecked())((*device_vk).vk_device, vk_image, core::ptr::null());
        return;
    }

    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!("Leaking image 0x%s.\n", wine_dbgstr_longlong(vk_image as ULONGLONG));
        return;
    }

    (*o).type_ = WINED3D_RETIRED_IMAGE_VK;
    (*o).u.vk_image = vk_image;
    (*o).command_buffer_id = command_buffer_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_vk_buffer_view(context_vk: *mut wined3d_context_vk, vk_view: VkBufferView, command_buffer_id: uint64_t) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if (*context_vk).completed_command_buffer_id >= command_buffer_id
    {
        ((*vk_info).vk_ops.vkDestroyBufferView.unwrap_unchecked())((*device_vk).vk_device, vk_view, core::ptr::null());

        return;
    }

    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null()
    {
        ERR!("Leaking buffer view 0x%s.\n", wine_dbgstr_longlong(vk_view as ULONGLONG));
        return;
    }

    (*o).type_ = WINED3D_RETIRED_BUFFER_VIEW_VK;
    (*o).u.vk_buffer_view = vk_view;
    (*o).command_buffer_id = command_buffer_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_vk_image_view(context_vk: *mut wined3d_context_vk, vk_view: VkImageView, command_buffer_id: uint64_t) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())((*device_vk).vk_device, vk_view, core::ptr::null());
        return;
    }

    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!("Leaking image view 0x%s.\n", wine_dbgstr_longlong(vk_view));
        return;
    }

    (*o).type_ = WINED3D_RETIRED_IMAGE_VIEW_VK;
    (*o).u.vk_image_view = vk_view;
    (*o).command_buffer_id = command_buffer_id;
}

unsafe extern "C" fn wined3d_context_vk_reset_completed_queries(
    context_vk: *mut wined3d_context_vk,
    pool_vk: *mut wined3d_query_pool_vk,
    buffer: *mut wined3d_command_buffer_vk,
) {
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut::<wined3d_retired_object_vk>();
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };
    let mut start: c_uint = 0;

    loop {
        if !(wined3d_bitmap_get_range(
            core::ptr::addr_of_mut!((*pool_vk).completed) as *mut uint32_t,
            WINED3D_QUERY_POOL_SIZE,
            start,
            core::ptr::addr_of_mut!(range),
        ) != 0)
        {
            break;
        }

        ((*vk_info).vk_ops.vkCmdResetQueryPool.unwrap_unchecked())(
            (*buffer).vk_command_buffer,
            (*pool_vk).vk_query_pool,
            range.offset as uint32_t,
            range.size as uint32_t,
        );

        o = wined3d_context_vk_get_retired_object_vk(context_vk);
        if o.is_null() {
            ERR!(
                "Freeing query range %u+%u in pool %p.\n",
                range.offset,
                range.size,
                pool_vk
            );
            wined3d_query_pool_vk_mark_free(
                context_vk,
                pool_vk,
                range.offset as uint32_t,
                range.size as uint32_t,
            );
        } else {
            (*o).type_ = WINED3D_RETIRED_QUERY_POOL_VK;
            (*o).u.queries.pool_vk = pool_vk;
            (*o).u.queries.start = range.offset as uint32_t;
            (*o).u.queries.count = range.size as uint32_t;
            (*o).command_buffer_id = (*buffer).id;
        }

        start = range.offset.wrapping_add(range.size);
    }

    memset(
        core::ptr::addr_of_mut!((*pool_vk).completed) as *mut uint32_t as *mut c_void,
        0,
        core::mem::size_of::<[uint32_t; 8]>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_vk_pipeline(context_vk: *mut wined3d_context_vk, vk_pipeline: VkPipeline, command_buffer_id: uint64_t) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, vk_pipeline, core::ptr::null());
        return;
    }

    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!("Leaking pipeline 0x%s.\n", wine_dbgstr_longlong(vk_pipeline));
        return;
    }

    (*o).type_ = WINED3D_RETIRED_PIPELINE_VK;
    (*o).u.vk_pipeline = vk_pipeline;
    (*o).command_buffer_id = command_buffer_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_vk_sampler(context_vk: *mut wined3d_context_vk, vk_sampler: VkSampler, command_buffer_id: uint64_t) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        ((*vk_info).vk_ops.vkDestroySampler)
            .unwrap_unchecked()(
            (*device_vk).vk_device,
            vk_sampler,
            core::ptr::null::<VkAllocationCallbacks>(),
        );
        return;
    }

    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!(
            "Leaking sampler 0x%s.\n",
            wine_dbgstr_longlong(vk_sampler)
        );
        return;
    }

    (*o).type_ = WINED3D_RETIRED_SAMPLER_VK;
    (*o).u.vk_sampler = vk_sampler;
    (*o).command_buffer_id = command_buffer_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_vk_event(context_vk: *mut wined3d_context_vk, vk_event: VkEvent, command_buffer_id: uint64_t) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut();

    if (*context_vk).completed_command_buffer_id >= command_buffer_id {
        ((*vk_info).vk_ops.vkDestroyEvent.unwrap_unchecked())((*device_vk).vk_device, vk_event, core::ptr::null());
        return;
    }

    o = wined3d_context_vk_get_retired_object_vk(context_vk);
    if o.is_null() {
        ERR!("Leaking event 0x%s.\n", wine_dbgstr_longlong(vk_event));
        return;
    }

    (*o).type_ = WINED3D_RETIRED_EVENT_VK;
    (*o).u.vk_event = vk_event;
    (*o).command_buffer_id = command_buffer_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_image(context_vk: *mut wined3d_context_vk, image: *mut wined3d_image_vk) {
    wined3d_context_vk_destroy_vk_image(context_vk, (*image).vk_image, (*image).command_buffer_id);
    if !(*image).memory.is_null() {
        wined3d_context_vk_destroy_allocator_block(
            context_vk,
            (*image).memory,
            (*image).command_buffer_id,
        );
    } else {
        wined3d_context_vk_destroy_vk_memory(
            context_vk,
            (*image).vk_memory,
            (*image).command_buffer_id,
        );
    }

    (*image).vk_image = VK_NULL_HANDLE as VkImage;
    (*image).vk_memory = VK_NULL_HANDLE as VkDeviceMemory;
    (*image).memory = ::core::ptr::null_mut::<wined3d_allocator_block>();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_destroy_bo(context_vk: *mut wined3d_context_vk, bo: *const wined3d_bo_vk) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut slab_vk: *mut wined3d_bo_slab_vk = core::ptr::null_mut();
    let mut object_size: size_t;
    let mut idx: size_t;

    assert!(list_empty(core::ptr::addr_of!((*bo).b.users)) != 0);

    if (*bo).command_buffer_id == (*context_vk).current_command_buffer.id {
        (*context_vk).retired_bo_size = (*context_vk).retired_bo_size.wrapping_add((*bo).size);
    }

    slab_vk = (*bo).slab as *mut wined3d_bo_slab_vk;
    if !slab_vk.is_null() {
        if !(*bo).b.map_ptr.is_null() {
            wined3d_bo_slab_vk_unmap(slab_vk, context_vk);
        }
        object_size = ((*slab_vk).bo.size / (32 as VkDeviceSize)) as size_t;
        idx = (*bo).b.buffer_offset / object_size;
        wined3d_context_vk_destroy_bo_slab_slice(context_vk, slab_vk, idx as SIZE_T, (*bo).command_buffer_id);
        return;
    }

    wined3d_context_vk_destroy_vk_buffer(context_vk, (*bo).vk_buffer, (*bo).command_buffer_id);
    if !(*bo).memory.is_null() {
        if !(*bo).b.map_ptr.is_null() {
            wined3d_allocator_chunk_vk_unmap(
                wined3d_allocator_chunk_vk((*(*bo).memory).chunk),
                context_vk,
            );
        }
        wined3d_context_vk_destroy_allocator_block(context_vk, (*bo).memory, (*bo).command_buffer_id);
        return;
    }

    if !(*bo).b.map_ptr.is_null() {
        ((*vk_info).vk_ops.vkUnmapMemory.unwrap_unchecked())((*device_vk).vk_device, (*bo).vk_memory);
        adapter_adjust_mapped_memory((*device_vk).d.adapter, (*bo).size.wrapping_neg() as ssize_t);
    }
    wined3d_context_vk_destroy_vk_memory(context_vk, (*bo).vk_memory, (*bo).command_buffer_id);
}

unsafe extern "C" fn free_command_buffer(context_vk: *mut wined3d_context_vk, buffer: *mut wined3d_command_buffer_vk) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;

    ((*vk_info).vk_ops.vkDestroyFence.unwrap_unchecked())((*device_vk).vk_device, (*buffer).vk_fence, core::ptr::null_mut());
    ((*vk_info).vk_ops.vkFreeCommandBuffers.unwrap_unchecked())(
        (*device_vk).vk_device,
        (*context_vk).vk_command_pool,
        1,
        core::ptr::addr_of_mut!((*buffer).vk_command_buffer),
    );
}

unsafe extern "C" fn wined3d_context_vk_remove_command_buffer(
    context_vk: *mut wined3d_context_vk,
    submit_index: ::core::ffi::c_uint,
) {
    let buffer: *mut wined3d_command_buffer_vk =
        (*context_vk).submitted.buffers.offset(submit_index as isize) as *mut wined3d_command_buffer_vk;

    if (*buffer).id > (*context_vk).completed_command_buffer_id {
        (*context_vk).completed_command_buffer_id = (*buffer).id;
    }

    if wined3d_array_reserve(
        &raw mut (*context_vk).completed.buffers as *mut *mut ::core::ffi::c_void,
        &raw mut (*context_vk).completed.buffers_size,
        (*context_vk).completed.buffer_count.wrapping_add(1 as _) as SIZE_T,
        ::core::mem::size_of::<wined3d_command_buffer_vk>() as SIZE_T,
    ) != 0
    {
        let fresh21 = (*context_vk).completed.buffer_count;
        (*context_vk).completed.buffer_count = (*context_vk).completed.buffer_count.wrapping_add(1);
        *(*context_vk).completed.buffers.offset(fresh21 as isize) = *buffer;
    } else {
        free_command_buffer(context_vk, buffer);
    }

    (*context_vk).submitted.buffer_count = (*context_vk).submitted.buffer_count.wrapping_sub(1);
    *buffer = *(*context_vk)
        .submitted
        .buffers
        .offset((*context_vk).submitted.buffer_count as isize);
}

unsafe extern "C" fn wined3d_context_vk_cleanup_resources(context_vk: *mut wined3d_context_vk, vk_fence: VkFence) {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let mut retired: *mut wined3d_retired_objects_vk = &raw mut (*context_vk).retired;
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut buffer: *mut wined3d_command_buffer_vk = core::ptr::null_mut::<wined3d_command_buffer_vk>();
    let mut o: *mut wined3d_retired_object_vk = core::ptr::null_mut::<wined3d_retired_object_vk>();
    let mut command_buffer_id: uint64_t = 0;
    let mut i: SIZE_T = 0;

    while i < (*context_vk).submitted.buffer_count {
        buffer = (*context_vk).submitted.buffers.offset(i as isize) as *mut wined3d_command_buffer_vk;
        if ((*vk_info).vk_ops.vkGetFenceStatus.unwrap_unchecked())((*device_vk).vk_device, (*buffer).vk_fence) == VK_NOT_READY {
            i = i.wrapping_add(1);
            continue;
        }

        if (*buffer).vk_fence == vk_fence {
            return;
        }
        wined3d_context_vk_remove_command_buffer(context_vk, i as ::core::ffi::c_uint);
    }

    command_buffer_id = (*context_vk).completed_command_buffer_id;

    (*retired).free = core::ptr::null_mut::<wined3d_retired_object_vk>();
    i = (*retired).count;
    'retired_loop: while i != 0 {
        o = (*retired).objects.offset(i.wrapping_sub(1) as isize) as *mut wined3d_retired_object_vk;

        'body: {
            if (*o).type_ as ::core::ffi::c_uint != WINED3D_RETIRED_FREE_VK as ::core::ffi::c_uint
                && (*o).command_buffer_id > command_buffer_id
            {
                break 'body;
            }

            match (*o).type_ as ::core::ffi::c_uint {
                WINED3D_RETIRED_FREE_VK => {
                    /* Nothing to do. */
                }
                WINED3D_RETIRED_FRAMEBUFFER_VK => {
                    ((*vk_info).vk_ops.vkDestroyFramebuffer.unwrap_unchecked())((*device_vk).vk_device, (*o).u.vk_framebuffer, core::ptr::null_mut());
                    TRACE!("Destroyed framebuffer 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_framebuffer as ULONGLONG));
                }
                WINED3D_RETIRED_DESCRIPTOR_POOL_VK => {
                    wined3d_context_vk_return_vk_descriptor_pool(context_vk, (*o).u.vk_descriptor_pool);
                    TRACE!("Reset descriptor pool 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_descriptor_pool as ULONGLONG));
                }
                WINED3D_RETIRED_MEMORY_VK => {
                    ((*vk_info).vk_ops.vkFreeMemory.unwrap_unchecked())((*device_vk).vk_device, (*o).u.vk_memory, core::ptr::null_mut());
                    TRACE!("Freed memory 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_memory as ULONGLONG));
                }
                WINED3D_RETIRED_ALLOCATOR_BLOCK_VK => {
                    TRACE!("Destroying block %p.\n", (*o).u.block);
                    wined3d_context_vk_free_memory(context_vk, (*o).u.block);
                }
                WINED3D_RETIRED_BO_SLAB_SLICE_VK => {
                    wined3d_bo_slab_vk_free_slice((*o).u.slice.slab, (*o).u.slice.idx as SIZE_T, context_vk);
                }
                WINED3D_RETIRED_BUFFER_VK => {
                    ((*vk_info).vk_ops.vkDestroyBuffer.unwrap_unchecked())((*device_vk).vk_device, (*o).u.vk_buffer, core::ptr::null_mut());
                    TRACE!("Destroyed buffer 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_buffer as ULONGLONG));
                }
                WINED3D_RETIRED_IMAGE_VK => {
                    ((*vk_info).vk_ops.vkDestroyImage.unwrap_unchecked())((*device_vk).vk_device, (*o).u.vk_image, core::ptr::null_mut());
                    TRACE!("Destroyed image 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_image as ULONGLONG));
                }
                WINED3D_RETIRED_BUFFER_VIEW_VK => {
                    ((*vk_info).vk_ops.vkDestroyBufferView.unwrap_unchecked())((*device_vk).vk_device, (*o).u.vk_buffer_view, core::ptr::null_mut());
                    TRACE!("Destroyed buffer view 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_buffer_view as ULONGLONG));
                }
                WINED3D_RETIRED_IMAGE_VIEW_VK => {
                    ((*vk_info).vk_ops.vkDestroyImageView.unwrap_unchecked())((*device_vk).vk_device, (*o).u.vk_image_view, core::ptr::null_mut());
                    TRACE!("Destroyed image view 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_image_view as ULONGLONG));
                }
                WINED3D_RETIRED_SAMPLER_VK => {
                    ((*vk_info).vk_ops.vkDestroySampler.unwrap_unchecked())((*device_vk).vk_device, (*o).u.vk_sampler, core::ptr::null_mut());
                    TRACE!("Destroyed sampler 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_sampler as ULONGLONG));
                }
                WINED3D_RETIRED_QUERY_POOL_VK => {
                    wined3d_query_pool_vk_mark_free(context_vk, (*o).u.queries.pool_vk, (*o).u.queries.start, (*o).u.queries.count);
                    TRACE!("Freed query range %u+%u in pool %p.\n", (*o).u.queries.start, (*o).u.queries.count, (*o).u.queries.pool_vk);
                }
                WINED3D_RETIRED_EVENT_VK => {
                    ((*vk_info).vk_ops.vkDestroyEvent.unwrap_unchecked())((*device_vk).vk_device, (*o).u.vk_event, core::ptr::null_mut());
                    TRACE!("Destroyed event 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_event as ULONGLONG));
                }
                WINED3D_RETIRED_PIPELINE_VK => {
                    ((*vk_info).vk_ops.vkDestroyPipeline.unwrap_unchecked())((*device_vk).vk_device, (*o).u.vk_pipeline, core::ptr::null_mut());
                    TRACE!("Destroyed pipeline 0x%s.\n", wine_dbgstr_longlong((*o).u.vk_pipeline as ULONGLONG));
                }
                _ => {
                    ERR!("Unhandled object type %#x.\n", (*o).type_ as ::core::ffi::c_uint);
                }
            }

            if i == (*retired).count {
                (*retired).count = (*retired).count.wrapping_sub(1);
                break 'body;
            }

            (*o).type_ = WINED3D_RETIRED_FREE_VK;
            (*o).u.next = (*retired).free;
            (*retired).free = o;

            if vk_fence != 0 && ((*vk_info).vk_ops.vkGetFenceStatus.unwrap_unchecked())((*device_vk).vk_device, vk_fence) != VK_NOT_READY {
                break 'retired_loop;
            }
        }
        i = i.wrapping_sub(1);
    }
}

unsafe extern "C" fn wined3d_context_vk_destroy_bo_slab(entry: *mut rb_entry, ctx: *mut core::ffi::c_void) {
    let mut context_vk: *mut wined3d_context_vk = ctx as *mut wined3d_context_vk;
    let mut slab: *mut wined3d_bo_slab_vk = core::ptr::null_mut::<wined3d_bo_slab_vk>();
    let mut next: *mut wined3d_bo_slab_vk = core::ptr::null_mut::<wined3d_bo_slab_vk>();
    slab = WINE_RB_ENTRY_VALUE!(entry, wined3d_bo_slab_vk, entry);
    while !slab.is_null() {
        next = (*slab).next;
        wined3d_context_vk_destroy_bo(context_vk, core::ptr::addr_of_mut!((*slab).bo));
        free(slab as *mut core::ffi::c_void);
        slab = next;
    }
}

unsafe extern "C" fn wined3d_context_vk_destroy_graphics_pipeline(
    mut entry: *mut wine_rb_entry,
    mut ctx: *mut core::ffi::c_void,
) {
    let mut pipeline_vk: *mut wined3d_graphics_pipeline_vk = (entry as *mut u8)
        .sub(core::mem::offset_of!(wined3d_graphics_pipeline_vk, entry))
        as *mut wined3d_graphics_pipeline_vk;
    let mut context_vk: *mut wined3d_context_vk = ctx as *mut wined3d_context_vk;
    let mut vk_info: *const wined3d_vk_info = core::ptr::null::<wined3d_vk_info>();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut::<wined3d_device_vk>();
    vk_info = (*context_vk).vk_info;
    device_vk = wined3d_device_vk((*context_vk).c.device);
    (*vk_info)
        .vk_ops
        .vkDestroyPipeline
        .unwrap_unchecked()(
        (*device_vk).vk_device,
        (*pipeline_vk).vk_pipeline,
        core::ptr::null::<VkAllocationCallbacks>(),
    );
    free(pipeline_vk as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_context_vk_destroy_pipeline_layout(entry: *mut rb_entry, ctx: *mut core::ffi::c_void) {
    let layout: *mut wined3d_pipeline_layout_vk = (entry as *mut u8)
        .sub(core::mem::offset_of!(wined3d_pipeline_layout_vk, entry))
        as *mut wined3d_pipeline_layout_vk;
    let context_vk: *mut wined3d_context_vk = ctx as *mut wined3d_context_vk;
    let vk_info: *const wined3d_vk_info;
    let device_vk: *mut wined3d_device_vk;

    vk_info = (*context_vk).vk_info;
    device_vk = wined3d_device_vk((*context_vk).c.device);

    (*vk_info)
        .vk_ops
        .vkDestroyPipelineLayout
        .unwrap_unchecked()(
        (*device_vk).vk_device,
        (*layout).vk_pipeline_layout,
        core::ptr::null::<VkAllocationCallbacks>(),
    );
    (*vk_info)
        .vk_ops
        .vkDestroyDescriptorSetLayout
        .unwrap_unchecked()(
        (*device_vk).vk_device,
        (*layout).vk_set_layout,
        core::ptr::null::<VkAllocationCallbacks>(),
    );
    free((*layout).key.bindings as *mut core::ffi::c_void);
    free(layout as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_render_pass_key_vk_init(
    mut key: *mut wined3d_render_pass_key_vk,
    mut fb: *const wined3d_fb_state,
    mut rt_count: ::core::ffi::c_uint,
    mut depth_stencil: bool,
    mut clear_flags: uint32_t,
) {
    let mut a: *mut wined3d_render_pass_attachment_vk =
        ::core::ptr::null_mut::<wined3d_render_pass_attachment_vk>();
    let mut view: *mut wined3d_rendertarget_view =
        ::core::ptr::null_mut::<wined3d_rendertarget_view>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut location: DWORD = 0;
    memset(
        key as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<wined3d_render_pass_key_vk>() as size_t,
    );
    i = 0 as ::core::ffi::c_uint;
    while i < rt_count {
        'body: {
            view = (*fb).render_targets[i as usize];
            if view.is_null()
                || (*(*view).format).id as ::core::ffi::c_uint
                    == WINED3DFMT_NULL as ::core::ffi::c_uint
            {
                break 'body;
            }
            a = (&raw mut (*key).rt as *mut wined3d_render_pass_attachment_vk).offset(i as isize)
                as *mut wined3d_render_pass_attachment_vk;
            (*a).vk_format = (*wined3d_format_vk((*view).format)).vk_format;
            (*a).vk_samples = max!(
                1 as ::core::ffi::c_uint,
                wined3d_resource_get_sample_count((*view).resource),
            ) as VkSampleCountFlagBits;
            (*a).vk_layout = (*wined3d_texture_vk(
                wined3d_texture_from_resource((*view).resource),
            ))
            .layout as VkImageLayout;
            location = wined3d_rendertarget_view_get_locations(view);
            if clear_flags & WINED3DCLEAR_TARGET as uint32_t != 0 {
                (*a).flags = WINED3D_FB_ATTACHMENT_FLAG_CLEAR_C as uint32_t;
            } else if location & WINED3D_LOCATION_DISCARDED as DWORD != 0 {
                (*a).flags = WINED3D_FB_ATTACHMENT_FLAG_DISCARDED as uint32_t;
            } else if location & WINED3D_LOCATION_CLEARED as DWORD != 0 {
                (*a).flags = WINED3D_FB_ATTACHMENT_FLAG_CLEAR_C as uint32_t;
            }
            (*key).rt_mask = ((*key).rt_mask as ::core::ffi::c_uint
                | (1 as ::core::ffi::c_uint) << i)
                as uint32_t;
        }
        i = i.wrapping_add(1);
    }
    if depth_stencil as ::core::ffi::c_int != 0 && {
        view = (*fb).depth_stencil;
        !view.is_null()
    } {
        a = &raw mut (*key).ds;
        (*a).vk_format = (*wined3d_format_vk((*view).format)).vk_format;
        (*a).vk_samples = max!(
            1 as ::core::ffi::c_uint,
            wined3d_resource_get_sample_count((*view).resource),
        ) as VkSampleCountFlagBits;
        (*a).vk_layout = (*wined3d_texture_vk(
            wined3d_texture_from_resource((*view).resource),
        ))
        .layout as VkImageLayout;
        location = wined3d_rendertarget_view_get_locations(view);
        (*key).rt_mask = ((*key).rt_mask as ::core::ffi::c_uint
            | (1 as ::core::ffi::c_uint)
                << WINED3D_MAX_RENDER_TARGETS as ::core::ffi::c_uint)
            as uint32_t;
        if clear_flags & WINED3DCLEAR_STENCIL as uint32_t != 0 {
            (*a).flags = WINED3D_FB_ATTACHMENT_FLAG_CLEAR_S as uint32_t;
        }
        if clear_flags & WINED3DCLEAR_ZBUFFER as uint32_t != 0 {
            (*a).flags |= WINED3D_FB_ATTACHMENT_FLAG_CLEAR_Z as uint32_t;
        }
        if (*a).flags == 0 && location & WINED3D_LOCATION_DISCARDED as DWORD != 0 {
            (*a).flags = WINED3D_FB_ATTACHMENT_FLAG_DISCARDED as uint32_t;
        } else if location & WINED3D_LOCATION_CLEARED as DWORD != 0 {
            (*a).flags = (WINED3D_FB_ATTACHMENT_FLAG_CLEAR_S as uint32_t
                | WINED3D_FB_ATTACHMENT_FLAG_CLEAR_Z as uint32_t) as uint32_t;
        }
    }
}

unsafe extern "C" fn wined3d_render_pass_vk_cleanup(
    pass: *mut wined3d_render_pass_vk,
    context_vk: *mut wined3d_context_vk,
) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;

    ((*vk_info).vk_ops.vkDestroyRenderPass.unwrap_unchecked())((*device_vk).vk_device, (*pass).vk_render_pass, core::ptr::null_mut());
}

unsafe extern "C" fn wined3d_render_pass_vk_init(
    pass: *mut wined3d_render_pass_vk,
    context_vk: *mut wined3d_context_vk,
    key: *const wined3d_render_pass_key_vk,
) -> bool {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let mut attachment_references: [VkAttachmentReference; (WINED3D_MAX_RENDER_TARGETS as _)] = core::mem::zeroed();
    let mut attachments: [VkAttachmentDescription; (WINED3D_MAX_RENDER_TARGETS + 1) as _] = core::mem::zeroed();
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut a: *const wined3d_render_pass_attachment_vk = core::ptr::null();
    let mut ds_attachment_reference: VkAttachmentReference = core::mem::zeroed();
    let mut ds_reference: *mut VkAttachmentReference = core::ptr::null_mut();
    let mut attachment_count: u32 = 0;
    let mut rt_count: u32 = 0;
    let mut i: u32 = 0;
    let mut attachment: *mut VkAttachmentDescription = core::ptr::null_mut();
    let mut sub_pass_desc: VkSubpassDescription = core::mem::zeroed();
    let mut pass_desc: VkRenderPassCreateInfo = core::mem::zeroed();
    let mut mask: u32 = 0;
    let mut vr: VkResult;

    rt_count = 0;
    attachment_count = 0;
    mask = (*key).rt_mask & wined3d_mask_from_size(WINED3D_MAX_RENDER_TARGETS);
    while mask != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(mask)) as u32;
        a = (core::ptr::addr_of!((*key).rt) as *const wined3d_render_pass_attachment_vk).add(i as usize);

        attachment = (core::ptr::addr_of_mut!(attachments) as *mut VkAttachmentDescription).add(attachment_count as usize);
        (*attachment).flags = 0;
        (*attachment).format = (*a).vk_format;
        (*attachment).samples = (*a).vk_samples;

        if ((*a).flags & WINED3D_FB_ATTACHMENT_FLAG_DISCARDED) != 0 {
            (*attachment).loadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
        } else if ((*a).flags & WINED3D_FB_ATTACHMENT_FLAG_CLEAR_C) != 0 {
            (*attachment).loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
        } else {
            (*attachment).loadOp = VK_ATTACHMENT_LOAD_OP_LOAD;
        }

        (*attachment).storeOp = VK_ATTACHMENT_STORE_OP_STORE;
        (*attachment).stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
        (*attachment).stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
        (*attachment).initialLayout = (*a).vk_layout;
        (*attachment).finalLayout = (*a).vk_layout;

        attachment_references[i as usize].attachment = attachment_count;
        attachment_references[i as usize].layout = (*a).vk_layout;

        attachment_count = attachment_count.wrapping_add(1);
        rt_count = i.wrapping_add(1);
    }

    mask = !(*key).rt_mask & wined3d_mask_from_size(rt_count);
    while mask != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(mask)) as u32;
        attachment_references[i as usize].attachment = (VK_ATTACHMENT_UNUSED) as u32;
        attachment_references[i as usize].layout = VK_IMAGE_LAYOUT_UNDEFINED;
    }

    if ((*key).rt_mask & (1 << WINED3D_MAX_RENDER_TARGETS)) != 0 {
        a = core::ptr::addr_of!((*key).ds);

        attachment = (core::ptr::addr_of_mut!(attachments) as *mut VkAttachmentDescription).add(attachment_count as usize);
        (*attachment).flags = 0;
        (*attachment).format = (*a).vk_format;
        (*attachment).samples = (*a).vk_samples;

        if ((*a).flags & WINED3D_FB_ATTACHMENT_FLAG_DISCARDED) != 0 {
            (*attachment).loadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
        } else if ((*a).flags & WINED3D_FB_ATTACHMENT_FLAG_CLEAR_Z) != 0 {
            (*attachment).loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
        } else {
            (*attachment).loadOp = VK_ATTACHMENT_LOAD_OP_LOAD;
        }

        if ((*a).flags & WINED3D_FB_ATTACHMENT_FLAG_DISCARDED) != 0 {
            (*attachment).stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
        } else if ((*a).flags & WINED3D_FB_ATTACHMENT_FLAG_CLEAR_S) != 0 {
            (*attachment).stencilLoadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
        } else {
            (*attachment).stencilLoadOp = VK_ATTACHMENT_LOAD_OP_LOAD;
        }

        (*attachment).storeOp = VK_ATTACHMENT_STORE_OP_STORE;
        (*attachment).stencilStoreOp = VK_ATTACHMENT_STORE_OP_STORE;
        (*attachment).initialLayout = (*a).vk_layout;
        (*attachment).finalLayout = (*a).vk_layout;

        ds_reference = core::ptr::addr_of_mut!(ds_attachment_reference);
        (*ds_reference).attachment = attachment_count;
        (*ds_reference).layout = (*a).vk_layout;

        attachment_count = attachment_count.wrapping_add(1);
    }

    sub_pass_desc.flags = 0;
    sub_pass_desc.pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS;
    sub_pass_desc.inputAttachmentCount = 0;
    sub_pass_desc.pInputAttachments = core::ptr::null();
    sub_pass_desc.colorAttachmentCount = rt_count;
    sub_pass_desc.pColorAttachments = core::ptr::addr_of_mut!(attachment_references) as *mut VkAttachmentReference;
    sub_pass_desc.pResolveAttachments = core::ptr::null();
    sub_pass_desc.pDepthStencilAttachment = ds_reference;
    sub_pass_desc.preserveAttachmentCount = 0;
    sub_pass_desc.pPreserveAttachments = core::ptr::null();

    pass_desc.sType = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO;
    pass_desc.pNext = core::ptr::null();
    pass_desc.flags = 0;
    pass_desc.attachmentCount = attachment_count;
    pass_desc.pAttachments = core::ptr::addr_of_mut!(attachments) as *mut VkAttachmentDescription;
    pass_desc.subpassCount = 1;
    pass_desc.pSubpasses = core::ptr::addr_of_mut!(sub_pass_desc);
    pass_desc.dependencyCount = 0;
    pass_desc.pDependencies = core::ptr::null();

    (*pass).key = *key;
    vr = ((*vk_info).vk_ops.vkCreateRenderPass.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(pass_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*pass).vk_render_pass));
    if (vr as i32) < 0 {
        WARN!("Failed to create Vulkan render pass, vr %d.\n", vr);
        return false;
    }

    let _ = vk_info;
    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_get_render_pass(context_vk: *mut wined3d_context_vk, fb: *const wined3d_fb_state, rt_count: ::core::ffi::c_uint, depth_stencil: bool, clear_flags: uint32_t) -> VkRenderPass {
    let mut key: wined3d_render_pass_key_vk = core::mem::zeroed();
    let mut pass: *mut wined3d_render_pass_vk = core::ptr::null_mut();
    let mut entry: *mut wine_rb_entry = core::ptr::null_mut();

    wined3d_render_pass_key_vk_init(core::ptr::addr_of_mut!(key), fb, rt_count, depth_stencil, clear_flags);
    entry = wine_rb_get(
        core::ptr::addr_of_mut!((*context_vk).render_passes),
        core::ptr::addr_of_mut!(key) as *const ::core::ffi::c_void,
    ) as *mut wine_rb_entry;
    if !entry.is_null() {
        return (*WINE_RB_ENTRY_VALUE!(entry, wined3d_render_pass_vk, entry)).vk_render_pass;
    }

    pass = malloc(core::mem::size_of::<wined3d_render_pass_vk>() as size_t) as *mut wined3d_render_pass_vk;
    if pass.is_null() {
        return (VK_NULL_HANDLE) as VkRenderPass;
    }

    if !wined3d_render_pass_vk_init(pass, context_vk, core::ptr::addr_of_mut!(key)) {
        free(pass as *mut ::core::ffi::c_void);
        return (VK_NULL_HANDLE) as VkRenderPass;
    }

    if wine_rb_put(
        core::ptr::addr_of_mut!((*context_vk).render_passes),
        core::ptr::addr_of_mut!((*pass).key) as *const ::core::ffi::c_void,
        core::ptr::addr_of_mut!((*pass).entry),
    ) == -1
    {
        ERR!("Failed to insert render pass.\n");
        wined3d_render_pass_vk_cleanup(pass, context_vk);
        free(pass as *mut ::core::ffi::c_void);
        return (VK_NULL_HANDLE) as VkRenderPass;
    }

    return (*pass).vk_render_pass;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_end_current_render_pass(mut context_vk: *mut wined3d_context_vk) {
    let mut vk_command_buffer: VkCommandBuffer =
        (*context_vk).current_command_buffer.vk_command_buffer;
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut query_vk: *mut wined3d_query_vk = core::ptr::null_mut::<wined3d_query_vk>();

    if (*context_vk).vk_render_pass != 0 {
        query_vk = ((*context_vk).render_pass_queries.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_query_vk, entry)) as *mut wined3d_query_vk;
        while core::ptr::addr_of_mut!((*query_vk).entry)
            != core::ptr::addr_of_mut!((*context_vk).render_pass_queries)
        {
            wined3d_query_vk_suspend(query_vk, context_vk);
            query_vk = ((*query_vk).entry.next as *mut u8)
                .sub(core::mem::offset_of!(wined3d_query_vk, entry)) as *mut wined3d_query_vk;
        }

        ((*vk_info).vk_ops.vkCmdEndRenderPass.unwrap_unchecked())(vk_command_buffer);
        (*context_vk).vk_render_pass = (VK_NULL_HANDLE as _);
        ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
            vk_command_buffer,
            VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT,
            VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT,
            0,
            0,
            core::ptr::null(),
            0,
            core::ptr::null(),
            0,
            core::ptr::null()
        );

        query_vk = ((*context_vk).render_pass_queries.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_query_vk, entry)) as *mut wined3d_query_vk;
        while core::ptr::addr_of_mut!((*query_vk).entry)
            != core::ptr::addr_of_mut!((*context_vk).render_pass_queries)
        {
            if !wined3d_context_vk_allocate_query(
                context_vk,
                (*query_vk).q.type_,
                core::ptr::addr_of_mut!((*query_vk).pool_idx),
            ) {
                ERR!("Failed to allocate new query.\n");
                break;
            } else {
                query_vk = ((*query_vk).entry.next as *mut u8)
                    .sub(core::mem::offset_of!(wined3d_query_vk, entry))
                    as *mut wined3d_query_vk;
            }
        }
    }

    if (*context_vk).vk_framebuffer != 0 {
        wined3d_context_vk_destroy_vk_framebuffer(
            context_vk,
            (*context_vk).vk_framebuffer,
            (*context_vk).current_command_buffer.id,
        );
        (*context_vk).vk_framebuffer = (VK_NULL_HANDLE as _);
    }
}

unsafe extern "C" fn wined3d_context_vk_destroy_render_pass(entry: *mut rb_entry, ctx: *mut core::ffi::c_void) {
    let pass: *mut wined3d_render_pass_vk = (entry as *mut u8)
        .sub(core::mem::offset_of!(wined3d_render_pass_vk, entry))
        as *mut wined3d_render_pass_vk;
    wined3d_render_pass_vk_cleanup(pass, ctx as *mut wined3d_context_vk);
    free(pass as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_shader_descriptor_writes_vk_cleanup(
    writes: *mut wined3d_shader_descriptor_writes_vk,
) {
    free((*writes).writes as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn wined3d_context_vk_destroy_query_pools(
    mut context_vk: *mut wined3d_context_vk,
    mut free_pools: *mut list,
) {
    let mut pool_vk: *mut wined3d_query_pool_vk = core::ptr::null_mut::<wined3d_query_pool_vk>();
    let mut entry: *mut wined3d_query_pool_vk = core::ptr::null_mut::<wined3d_query_pool_vk>();
    pool_vk = ((*free_pools).next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_query_pool_vk, entry)) as *mut wined3d_query_pool_vk;
    entry = (((*pool_vk).entry.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_query_pool_vk, entry)))
        as *mut wined3d_query_pool_vk;
    while core::ptr::addr_of_mut!((*pool_vk).entry) != free_pools {
        wined3d_query_pool_vk_cleanup(pool_vk, context_vk);
        free(pool_vk as *mut core::ffi::c_void);
        pool_vk = entry;
        entry = (((*pool_vk).entry.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_query_pool_vk, entry)))
            as *mut wined3d_query_pool_vk;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_allocate_query(context_vk: *mut wined3d_context_vk, type_: wined3d_query_type, pool_idx: *mut wined3d_query_pool_idx_vk) -> bool {
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut pool_vk: *mut wined3d_query_pool_vk = core::ptr::null_mut();
    let mut entry: *mut wined3d_query_pool_vk = core::ptr::null_mut();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut();
    let free_pools: *mut list;
    let mut vr: VkResult = VK_SUCCESS;
    let mut idx: size_t = 0;

    match type_ as u32 {
        WINED3D_QUERY_TYPE_OCCLUSION => {
            free_pools = core::ptr::addr_of_mut!((*context_vk).free_occlusion_query_pools);
        }
        WINED3D_QUERY_TYPE_TIMESTAMP => {
            free_pools = core::ptr::addr_of_mut!((*context_vk).free_timestamp_query_pools);
        }
        WINED3D_QUERY_TYPE_PIPELINE_STATISTICS => {
            free_pools = core::ptr::addr_of_mut!((*context_vk).free_pipeline_statistics_query_pools);
        }
        WINED3D_QUERY_TYPE_SO_STATISTICS
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM1
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM2
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3 => {
            free_pools = core::ptr::addr_of_mut!((*context_vk).free_stream_output_statistics_query_pools);
        }
        _ => {
            FIXME!("Unhandled query type %#x.\n", type_ as u32);
            return false;
        }
    }

    'done: {
        pool_vk = ((*free_pools).next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_query_pool_vk, entry))
            as *mut wined3d_query_pool_vk;
        entry = ((*pool_vk).entry.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_query_pool_vk, entry))
            as *mut wined3d_query_pool_vk;
        loop {
            if !(core::ptr::addr_of_mut!((*pool_vk).entry) != free_pools) {
                break;
            }
            if wined3d_query_pool_vk_allocate_query(pool_vk, core::ptr::addr_of_mut!(idx)) {
                break 'done;
            }
            list_remove(core::ptr::addr_of_mut!((*pool_vk).entry));
            list_init(core::ptr::addr_of_mut!((*pool_vk).entry));
            pool_vk = entry;
            entry = ((*pool_vk).entry.next as *mut u8)
                .sub(core::mem::offset_of!(wined3d_query_pool_vk, entry))
                as *mut wined3d_query_pool_vk;
        }

        pool_vk = calloc(
            (1 as size_t) as _,
            (core::mem::size_of::<wined3d_query_pool_vk>() as size_t) as _,
        ) as *mut wined3d_query_pool_vk;
        if pool_vk.is_null() {
            return false;
        }
        if !wined3d_query_pool_vk_init(pool_vk, context_vk, type_, free_pools) {
            free(pool_vk as *mut core::ffi::c_void);
            return false;
        }

        device_vk = wined3d_device_vk((*context_vk).c.device);

        if (*vk_info).supported[WINED3D_VK_EXT_HOST_QUERY_RESET as usize] != 0 {
            ((*vk_info).vk_ops.vkResetQueryPoolEXT.unwrap_unchecked())(
                (*device_vk).vk_device,
                (*pool_vk).vk_query_pool,
                0,
                WINED3D_QUERY_POOL_SIZE,
            );
        } else {
            let mut event_create_info: VkEventCreateInfo = VkEventCreateInfo {
                sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
                pNext: core::ptr::null(),
                flags: 0,
            };

            wined3d_context_vk_end_current_render_pass(context_vk);
            ((*vk_info).vk_ops.vkCmdResetQueryPool.unwrap_unchecked())(
                wined3d_context_vk_get_command_buffer(context_vk),
                (*pool_vk).vk_query_pool,
                0,
                WINED3D_QUERY_POOL_SIZE,
            );

            event_create_info.sType = VK_STRUCTURE_TYPE_EVENT_CREATE_INFO;
            event_create_info.pNext = core::ptr::null();
            event_create_info.flags = 0;

            /* We probably shouldn't call vkGetQueryPoolResults() without synchronizing with vkCmdResetQueryPool()
             * even if the query pool is freshly allocated. wined3d_query_vk_accumulate_data() will check this event
             * before returning results. */
            vr = ((*vk_info).vk_ops.vkCreateEvent.unwrap_unchecked())(
                (*device_vk).vk_device,
                core::ptr::addr_of_mut!(event_create_info),
                core::ptr::null(),
                core::ptr::addr_of_mut!((*pool_vk).vk_event),
            );
            if vr == VK_SUCCESS {
                /* At which stage vkCmdResetQueryPool() executes? */
                ((*vk_info).vk_ops.vkCmdSetEvent.unwrap_unchecked())(
                    wined3d_context_vk_get_command_buffer(context_vk),
                    (*pool_vk).vk_event,
                    VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                );
            } else {
                ERR!("Failed to create event, vr %s.\n", wined3d_debug_vkresult(vr));
            }
        }

        if !wined3d_query_pool_vk_allocate_query(pool_vk, core::ptr::addr_of_mut!(idx)) {
            wined3d_query_pool_vk_cleanup(pool_vk, context_vk);
            free(pool_vk as *mut core::ffi::c_void);
            return false;
        }
    }

    (*pool_idx).pool_vk = pool_vk;
    (*pool_idx).idx = idx;

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_cleanup(context_vk: *mut wined3d_context_vk) {
    let buffer: *mut wined3d_command_buffer_vk = core::ptr::addr_of_mut!((*context_vk).current_command_buffer);
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut i: core::ffi::c_uint;

    if !(*buffer).vk_command_buffer.is_null() {
        free_command_buffer(context_vk, buffer);
        (*buffer).vk_command_buffer = core::ptr::null_mut();
    }

    wined3d_context_vk_wait_command_buffer(context_vk, (*buffer).id.wrapping_sub(1));
    (*context_vk).completed_command_buffer_id = (*buffer).id;
    i = 0;
    while (i as SIZE_T) < (*context_vk).completed.buffer_count {
        free_command_buffer(
            context_vk,
            (*context_vk).completed.buffers.offset(i as isize) as *mut wined3d_command_buffer_vk,
        );
        i = i.wrapping_add(1);
    }

    free((*context_vk).compute.bindings.bindings as *mut core::ffi::c_void);
    free((*context_vk).graphics.bindings.bindings as *mut core::ffi::c_void);
    i = 0;
    while (i as SIZE_T) < (*context_vk).vk_descriptor_pool_count {
        ((*vk_info).vk_ops.vkDestroyDescriptorPool.unwrap_unchecked())(
            (*device_vk).vk_device,
            *(*context_vk).vk_descriptor_pools.offset(i as isize),
            core::ptr::null(),
        );
        i = i.wrapping_add(1);
    }
    free((*context_vk).vk_descriptor_pools as *mut core::ffi::c_void);
    if !((*context_vk).vk_framebuffer == 0) {
        ((*vk_info).vk_ops.vkDestroyFramebuffer.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*context_vk).vk_framebuffer,
            core::ptr::null(),
        );
    }
    if !((*context_vk).vk_so_counter_bo.vk_buffer == 0) {
        wined3d_context_vk_destroy_bo(context_vk, core::ptr::addr_of!((*context_vk).vk_so_counter_bo));
    }
    wined3d_context_vk_cleanup_resources(context_vk, (VK_NULL_HANDLE) as VkFence);
    /* Destroy the command pool after cleaning up resources. In particular,
     * this needs to happen after all command buffers are freed, because
     * vkFreeCommandBuffers() requires a valid pool handle. */
    ((*vk_info).vk_ops.vkDestroyCommandPool.unwrap_unchecked())(
        (*device_vk).vk_device,
        (*context_vk).vk_command_pool,
        core::ptr::null(),
    );
    wined3d_context_vk_destroy_query_pools(context_vk, core::ptr::addr_of_mut!((*context_vk).free_occlusion_query_pools));
    wined3d_context_vk_destroy_query_pools(context_vk, core::ptr::addr_of_mut!((*context_vk).free_timestamp_query_pools));
    wined3d_context_vk_destroy_query_pools(context_vk, core::ptr::addr_of_mut!((*context_vk).free_pipeline_statistics_query_pools));
    wined3d_context_vk_destroy_query_pools(context_vk, core::ptr::addr_of_mut!((*context_vk).free_stream_output_statistics_query_pools));
    wine_rb_destroy(
        core::ptr::addr_of_mut!((*context_vk).bo_slab_available),
        Some(wined3d_context_vk_destroy_bo_slab as unsafe extern "C" fn(*mut rb_entry, *mut core::ffi::c_void) -> ()),
        context_vk as *mut core::ffi::c_void,
    );
    free((*context_vk).submitted.buffers as *mut core::ffi::c_void);
    free((*context_vk).completed.buffers as *mut core::ffi::c_void);
    free((*context_vk).retired.objects as *mut core::ffi::c_void);

    wined3d_shader_descriptor_writes_vk_cleanup(core::ptr::addr_of_mut!((*context_vk).descriptor_writes));
    wine_rb_destroy(
        core::ptr::addr_of_mut!((*context_vk).graphics_pipelines),
        Some(wined3d_context_vk_destroy_graphics_pipeline as unsafe extern "C" fn(*mut rb_entry, *mut core::ffi::c_void) -> ()),
        context_vk as *mut core::ffi::c_void,
    );
    wine_rb_destroy(
        core::ptr::addr_of_mut!((*context_vk).pipeline_layouts),
        Some(wined3d_context_vk_destroy_pipeline_layout as unsafe extern "C" fn(*mut rb_entry, *mut core::ffi::c_void) -> ()),
        context_vk as *mut core::ffi::c_void,
    );
    wine_rb_destroy(
        core::ptr::addr_of_mut!((*context_vk).render_passes),
        Some(wined3d_context_vk_destroy_render_pass as unsafe extern "C" fn(*mut rb_entry, *mut core::ffi::c_void) -> ()),
        context_vk as *mut core::ffi::c_void,
    );

    wined3d_context_cleanup(core::ptr::addr_of_mut!((*context_vk).c));
}

unsafe extern "C" fn should_periodic_submit(context_vk: *mut wined3d_context_vk) -> bool {
    let mut busy_count: u64;

    if (*context_vk).command_buffer_work_count < WINED3D_PERIODIC_SUBMIT_WORK_COUNT {
        return false;
    }

    /* The point of periodic submit is to keep the GPU busy, so if it's already
     * busy with 4 or more command buffers, don't submit another one now. */
    busy_count = (*context_vk)
        .current_command_buffer
        .id
        .wrapping_sub((*context_vk).completed_command_buffer_id)
        .wrapping_sub(1);
    if busy_count > (WINED3D_PERIODIC_SUBMIT_MAX_BUFFERS as _) {
        return false;
    }

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_get_command_buffer(context_vk: *mut wined3d_context_vk) -> VkCommandBuffer {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut buffer: *mut wined3d_command_buffer_vk = core::ptr::null_mut();
    let mut begin_info: VkCommandBufferBeginInfo = core::mem::zeroed();
    let mut query_vk: *mut wined3d_query_vk = core::ptr::null_mut();
    let mut vr: VkResult = core::mem::zeroed();

    buffer = core::ptr::addr_of_mut!((*context_vk).current_command_buffer);
    if !(*buffer).vk_command_buffer.is_null() {
        if (*context_vk).retired_bo_size > (WINED3D_RETIRED_BO_SIZE_THRESHOLD as _) || should_periodic_submit(context_vk) {
            wined3d_context_vk_submit_command_buffer(
                context_vk,
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
        } else {
            return (*buffer).vk_command_buffer;
        }
    }

    if (*context_vk).completed.buffer_count != 0 {
        let old_buffer: *const wined3d_command_buffer_vk;

        (*context_vk).completed.buffer_count -= 1;
        old_buffer = (*context_vk).completed.buffers.add((*context_vk).completed.buffer_count as usize);
        (*buffer).vk_command_buffer = (*old_buffer).vk_command_buffer;
        (*buffer).vk_fence = (*old_buffer).vk_fence;
    } else {
        let mut command_buffer_info: VkCommandBufferAllocateInfo = core::mem::zeroed();
        let mut fence_desc: VkFenceCreateInfo = core::mem::zeroed();

        fence_desc.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
        fence_desc.pNext = core::ptr::null();
        fence_desc.flags = 0;
        vr = ((*vk_info).vk_ops.vkCreateFence.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(fence_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*buffer).vk_fence),
        );
        if (vr as i32) < 0 {
            ERR!("Failed to create fence, vr %s.\n", wined3d_debug_vkresult(vr));
            return core::ptr::null_mut();
        }

        command_buffer_info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
        command_buffer_info.pNext = core::ptr::null();
        command_buffer_info.commandPool = (*context_vk).vk_command_pool;
        command_buffer_info.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
        command_buffer_info.commandBufferCount = 1;
        vr = ((*vk_info).vk_ops.vkAllocateCommandBuffers.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(command_buffer_info),
            core::ptr::addr_of_mut!((*buffer).vk_command_buffer),
        );
        if (vr as i32) < 0 {
            WARN!("Failed to allocate Vulkan command buffer, vr %s.\n", wined3d_debug_vkresult(vr));
            ((*vk_info).vk_ops.vkDestroyFence.unwrap_unchecked())((*device_vk).vk_device, (*buffer).vk_fence, core::ptr::null());
            return core::ptr::null_mut();
        }
    }

    begin_info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    begin_info.pNext = core::ptr::null();
    begin_info.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
    begin_info.pInheritanceInfo = core::ptr::null();
    vr = ((*vk_info).vk_ops.vkBeginCommandBuffer.unwrap_unchecked())((*buffer).vk_command_buffer, core::ptr::addr_of_mut!(begin_info));
    if (vr as i32) < 0 {
        WARN!("Failed to begin command buffer, vr %s.\n", wined3d_debug_vkresult(vr));
        ((*vk_info).vk_ops.vkFreeCommandBuffers.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*context_vk).vk_command_pool,
            1,
            core::ptr::addr_of_mut!((*buffer).vk_command_buffer),
        );
        (*buffer).vk_command_buffer = core::ptr::null_mut();
        return (*buffer).vk_command_buffer;
    }

    query_vk = ((*context_vk).active_queries.next as *mut u8)
        .offset(-(core::mem::offset_of!(wined3d_query_vk, entry) as isize)) as *mut wined3d_query_vk;
    while core::ptr::addr_of_mut!((*query_vk).entry) != core::ptr::addr_of_mut!((*context_vk).active_queries) {
        if !wined3d_context_vk_allocate_query(
            context_vk,
            (*query_vk).q.type_,
            core::ptr::addr_of_mut!((*query_vk).pool_idx),
        ) {
            ERR!("Failed to allocate new query.\n");
            break;
        }

        wined3d_query_vk_resume(query_vk, context_vk);
        query_vk = ((*query_vk).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(wined3d_query_vk, entry) as isize)) as *mut wined3d_query_vk;
    }

    (*context_vk).command_buffer_work_count = 0;

    return (*buffer).vk_command_buffer;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_submit_command_buffer(context_vk: *mut wined3d_context_vk, wait_semaphore_count: ::core::ffi::c_uint, wait_semaphores: *const VkSemaphore, wait_stages: *const VkPipelineStageFlags, signal_semaphore_count: ::core::ffi::c_uint, signal_semaphores: *const VkSemaphore) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut pool_vk: *mut wined3d_query_pool_vk = core::ptr::null_mut();
    let mut pool_vk_next: *mut wined3d_query_pool_vk = core::ptr::null_mut();
    let mut buffer: *mut wined3d_command_buffer_vk = core::ptr::null_mut();
    let mut query_vk: *mut wined3d_query_vk = core::ptr::null_mut();
    let mut submit_info: VkSubmitInfo = core::mem::zeroed();
    let mut vr: VkResult;

    buffer = core::ptr::addr_of_mut!((*context_vk).current_command_buffer);
    if (*buffer).vk_command_buffer.is_null() {
        return;
    }

    wined3d_context_vk_end_current_render_pass(context_vk);

    pool_vk = ((*context_vk).completed_query_pools.next as *mut u8)
        .offset(-(core::mem::offset_of!(wined3d_query_pool_vk, completed_entry) as isize))
        as *mut wined3d_query_pool_vk;
    pool_vk_next = ((*pool_vk).completed_entry.next as *mut u8)
        .offset(-(core::mem::offset_of!(wined3d_query_pool_vk, completed_entry) as isize))
        as *mut wined3d_query_pool_vk;
    while core::ptr::addr_of_mut!((*pool_vk).completed_entry)
        != core::ptr::addr_of_mut!((*context_vk).completed_query_pools)
    {
        list_remove(core::ptr::addr_of_mut!((*pool_vk).completed_entry));
        list_init(core::ptr::addr_of_mut!((*pool_vk).completed_entry));

        wined3d_context_vk_reset_completed_queries(context_vk, pool_vk, buffer);

        pool_vk = pool_vk_next;
        pool_vk_next = ((*pool_vk).completed_entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(wined3d_query_pool_vk, completed_entry) as isize))
            as *mut wined3d_query_pool_vk;
    }

    query_vk = ((*context_vk).active_queries.next as *mut u8)
        .offset(-(core::mem::offset_of!(wined3d_query_vk, entry) as isize))
        as *mut wined3d_query_vk;
    while core::ptr::addr_of_mut!((*query_vk).entry)
        != core::ptr::addr_of_mut!((*context_vk).active_queries)
    {
        wined3d_query_vk_suspend(query_vk, context_vk);

        query_vk = ((*query_vk).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(wined3d_query_vk, entry) as isize))
            as *mut wined3d_query_vk;
    }

    (*context_vk).graphics.vk_pipeline = (VK_NULL_HANDLE as _);
    (*context_vk).set_update_compute_pipeline((1) as _);
    (*context_vk).set_update_stream_output((1) as _);
    (*context_vk).c.set_update_shader_resource_bindings((1) as _);
    (*context_vk).c.set_update_compute_shader_resource_bindings((1) as _);
    (*context_vk).c.set_update_unordered_access_view_bindings((1) as _);
    (*context_vk).c.set_update_compute_unordered_access_view_bindings((1) as _);
    (*context_vk).c.set_update_primitive_type((1) as _);
    (*context_vk).c.set_update_patch_vertex_count((1) as _);
    (*context_vk).c.set_update_multisample_state((1) as _);
    context_invalidate_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_STREAMSRC as u32));
    context_invalidate_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_INDEXBUFFER as u32));
    context_invalidate_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_BLEND as u32));
    context_invalidate_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_BLEND_FACTOR as u32));
    context_invalidate_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_DEPTH_STENCIL as u32));
    context_invalidate_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_RASTERIZER as u32));
    context_invalidate_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_STENCIL_REF as u32));
    context_invalidate_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_VIEWPORT as u32));
    context_invalidate_state(core::ptr::addr_of_mut!((*context_vk).c), (STATE_SCISSORRECT as u32));

    ((*vk_info).vk_ops.vkEndCommandBuffer.unwrap_unchecked())((*buffer).vk_command_buffer);

    ((*vk_info).vk_ops.vkResetFences.unwrap_unchecked())(
        (*device_vk).vk_device,
        1,
        core::ptr::addr_of_mut!((*buffer).vk_fence),
    );

    submit_info.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
    submit_info.pNext = core::ptr::null();
    submit_info.waitSemaphoreCount = wait_semaphore_count;
    submit_info.pWaitSemaphores = wait_semaphores;
    submit_info.pWaitDstStageMask = wait_stages;
    submit_info.commandBufferCount = 1;
    submit_info.pCommandBuffers = core::ptr::addr_of_mut!((*buffer).vk_command_buffer);
    submit_info.signalSemaphoreCount = signal_semaphore_count;
    submit_info.pSignalSemaphores = signal_semaphores;

    vr = ((*vk_info).vk_ops.vkQueueSubmit.unwrap_unchecked())(
        (*device_vk).vk_queue,
        1,
        core::ptr::addr_of_mut!(submit_info),
        (*buffer).vk_fence,
    );
    if (vr as i32) < 0 {
        ERR!(
            "Failed to submit command buffer %p, vr %s.\n",
            (*buffer).vk_command_buffer,
            wined3d_debug_vkresult(vr)
        );
    }

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*context_vk).submitted.buffers) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*context_vk).submitted.buffers_size),
        (*context_vk).submitted.buffer_count.wrapping_add(1),
        core::mem::size_of::<wined3d_command_buffer_vk>() as SIZE_T,
    ) == 0
    {
        ERR!("Failed to grow submitted command buffer array.\n");
    }

    let fresh19 = (*context_vk).submitted.buffer_count;
    (*context_vk).submitted.buffer_count = (*context_vk).submitted.buffer_count.wrapping_add(1);
    *(*context_vk).submitted.buffers.offset(fresh19 as isize) = *buffer;

    (*buffer).vk_command_buffer = core::ptr::null_mut();
    /* We don't expect this to ever happen, but handle it anyway. */
    (*buffer).id = (*buffer).id.wrapping_add(1);
    if (*buffer).id == 0 {
        wined3d_context_vk_wait_command_buffer(context_vk, (*buffer).id.wrapping_sub(1));
        (*context_vk).completed_command_buffer_id = 0;
        (*buffer).id = 1;
    }
    (*context_vk).retired_bo_size = 0;
    wined3d_context_vk_cleanup_resources(context_vk, (VK_NULL_HANDLE) as VkFence);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_wait_command_buffer(context_vk: *mut wined3d_context_vk, id: uint64_t) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut vk_fence: VkFence = 0;
    let mut i: SIZE_T;

    if id <= (*context_vk).completed_command_buffer_id
        || id > (*context_vk).current_command_buffer.id
    {
        return;
    }

    i = 0;
    while i < (*context_vk).submitted.buffer_count {
        if (*(*context_vk).submitted.buffers.add(i as usize)).id != id {
            i = i.wrapping_add(1);
            continue;
        }

        vk_fence = (*(*context_vk).submitted.buffers.add(i as usize)).vk_fence;
        wined3d_context_vk_cleanup_resources(context_vk, vk_fence);
        i = 0;
        while i < (*context_vk).submitted.buffer_count {
            if (*(*context_vk).submitted.buffers.add(i as usize)).id != id {
                i = i.wrapping_add(1);
                continue;
            }

            ((*vk_info).vk_ops.vkWaitForFences.unwrap_unchecked())(
                (*device_vk).vk_device,
                1,
                core::ptr::addr_of_mut!(vk_fence),
                VK_TRUE,
                UINT64_MAX,
            );
            wined3d_context_vk_remove_command_buffer(context_vk, i as u32);
            return;
        }
        i = i.wrapping_add(1);
    }

    ERR!("Failed to find fence for command buffer with id 0x%s.\n", wine_dbgstr_longlong(id));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_image_barrier(mut context_vk: *mut wined3d_context_vk, mut vk_command_buffer: VkCommandBuffer, mut src_stage_mask: VkPipelineStageFlags, mut dst_stage_mask: VkPipelineStageFlags, mut src_access_mask: VkAccessFlags, mut dst_access_mask: VkAccessFlags, mut old_layout: VkImageLayout, mut new_layout: VkImageLayout, mut image: VkImage, mut range: *const VkImageSubresourceRange) {
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut barrier: VkImageMemoryBarrier = VkImageMemoryBarrier {
        sType: VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER as _,
        pNext: ::core::ptr::null::<::core::ffi::c_void>(),
        srcAccessMask: 0,
        dstAccessMask: 0,
        oldLayout: VK_IMAGE_LAYOUT_UNDEFINED,
        newLayout: VK_IMAGE_LAYOUT_UNDEFINED,
        srcQueueFamilyIndex: 0,
        dstQueueFamilyIndex: 0,
        image: 0,
        subresourceRange: VkImageSubresourceRange {
            aspectMask: 0,
            baseMipLevel: 0,
            levelCount: 0,
            baseArrayLayer: 0,
            layerCount: 0,
        },
    };
    wined3d_context_vk_end_current_render_pass(context_vk);
    barrier.sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER as _;
    barrier.pNext = ::core::ptr::null::<::core::ffi::c_void>();
    barrier.srcAccessMask = src_access_mask;
    barrier.dstAccessMask = dst_access_mask;
    barrier.oldLayout = old_layout;
    barrier.newLayout = new_layout;
    barrier.srcQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED as uint32_t;
    barrier.dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED as uint32_t;
    barrier.image = image;
    barrier.subresourceRange = *range;
    (*vk_info)
        .vk_ops
        .vkCmdPipelineBarrier
        .unwrap_unchecked()(
        vk_command_buffer,
        src_stage_mask,
        dst_stage_mask,
        0 as VkDependencyFlags,
        0 as uint32_t,
        ::core::ptr::null::<VkMemoryBarrier>(),
        0 as uint32_t,
        ::core::ptr::null::<VkBufferMemoryBarrier>(),
        1 as uint32_t,
        &raw mut barrier,
    );
}

unsafe extern "C" fn wined3d_render_pass_vk_compare(key: *const core::ffi::c_void, entry: *const wine_rb_entry) -> core::ffi::c_int {
    let k: *const wined3d_render_pass_key_vk = key as *const wined3d_render_pass_key_vk;
    let pass: *const wined3d_render_pass_vk = (entry as *mut u8)
        .offset(-(core::mem::offset_of!(wined3d_render_pass_vk, entry) as isize))
        as *const wined3d_render_pass_vk;
    return memcmp(
        k as *const core::ffi::c_void,
        core::ptr::addr_of!((*pass).key) as *const core::ffi::c_void,
        core::mem::size_of::<wined3d_render_pass_key_vk>(),
    );
}

unsafe extern "C" fn wined3d_pipeline_layout_vk_compare(key: *const core::ffi::c_void, entry: *const rb_entry) -> core::ffi::c_int {
    let a: *const wined3d_pipeline_layout_key_vk = key as *const wined3d_pipeline_layout_key_vk;
    let b: *const wined3d_pipeline_layout_key_vk = &raw const (*((entry as *mut core::ffi::c_char).offset(-(core::mem::offset_of!(wined3d_pipeline_layout_vk, entry) as isize)) as *const wined3d_pipeline_layout_vk)).key;
    let mut ret: core::ffi::c_int;

    ret = wined3d_uint32_compare(((*a).binding_count) as uint32_t, ((*b).binding_count) as uint32_t);
    if ret != 0 {
        return ret;
    }
    return memcmp((*a).bindings as *const core::ffi::c_void, (*b).bindings as *const core::ffi::c_void, ((*a).binding_count as usize).wrapping_mul(core::mem::size_of::<VkDescriptorSetLayoutBinding>()));
}

unsafe extern "C" fn wined3d_graphics_pipeline_vk_compare(
    key: *const core::ffi::c_void,
    entry: *const rb_entry,
) -> core::ffi::c_int {
    let a: *const wined3d_graphics_pipeline_key_vk = key as *const wined3d_graphics_pipeline_key_vk;
    let b: *const wined3d_graphics_pipeline_key_vk = core::ptr::addr_of!(
        (*((entry as *mut core::ffi::c_char).offset(-(0 as isize))
            as *const wined3d_graphics_pipeline_vk))
            .key
    );
    let mut i: core::ffi::c_uint;
    let mut ret: core::ffi::c_int;

    ret = wined3d_uint32_compare((*a).pipeline_desc.stageCount, (*b).pipeline_desc.stageCount);
    if ret != 0 {
        return ret;
    }
    i = 0 as core::ffi::c_uint;
    while (i as uint32_t) < (*a).pipeline_desc.stageCount {
        ret = wined3d_uint64_compare(
            (*a).stages[i as usize].module as uint64_t,
            (*b).stages[i as usize].module as uint64_t,
        );
        if ret != 0 {
            return ret;
        }
        i = i.wrapping_add(1);
    }

    ret = wined3d_uint32_compare(
        (*a).divisor_desc.vertexBindingDivisorCount,
        (*b).divisor_desc.vertexBindingDivisorCount,
    );
    if ret != 0 {
        return ret;
    }
    ret = memcmp(
        core::ptr::addr_of!((*a).divisors) as *const core::ffi::c_void,
        core::ptr::addr_of!((*b).divisors) as *const core::ffi::c_void,
        ((*a).divisor_desc.vertexBindingDivisorCount as size_t).wrapping_mul(
            core::mem::size_of::<VkVertexInputBindingDivisorDescriptionEXT>() as size_t,
        ),
    );
    if ret != 0 {
        return ret;
    }

    ret = wined3d_uint32_compare(
        (*a).input_desc.vertexAttributeDescriptionCount,
        (*b).input_desc.vertexAttributeDescriptionCount,
    );
    if ret != 0 {
        return ret;
    }
    ret = memcmp(
        core::ptr::addr_of!((*a).attributes) as *const core::ffi::c_void,
        core::ptr::addr_of!((*b).attributes) as *const core::ffi::c_void,
        ((*a).input_desc.vertexAttributeDescriptionCount as size_t)
            .wrapping_mul(core::mem::size_of::<VkVertexInputAttributeDescription>() as size_t),
    );
    if ret != 0 {
        return ret;
    }
    ret = wined3d_uint32_compare(
        (*a).input_desc.vertexBindingDescriptionCount,
        (*b).input_desc.vertexBindingDescriptionCount,
    );
    if ret != 0 {
        return ret;
    }
    ret = memcmp(
        core::ptr::addr_of!((*a).bindings) as *const core::ffi::c_void,
        core::ptr::addr_of!((*b).bindings) as *const core::ffi::c_void,
        ((*a).input_desc.vertexBindingDescriptionCount as size_t)
            .wrapping_mul(core::mem::size_of::<VkVertexInputBindingDescription>() as size_t),
    );
    if ret != 0 {
        return ret;
    }

    ret = wined3d_uint32_compare(
        (*a).ia_desc.topology as uint32_t,
        (*b).ia_desc.topology as uint32_t,
    );
    if ret != 0 {
        return ret;
    }
    ret = wined3d_uint32_compare(
        (*a).ia_desc.primitiveRestartEnable as uint32_t,
        (*b).ia_desc.primitiveRestartEnable as uint32_t,
    );
    if ret != 0 {
        return ret;
    }

    ret = wined3d_uint32_compare(
        (*a).ts_desc.patchControlPoints,
        (*b).ts_desc.patchControlPoints,
    );
    if ret != 0 {
        return ret;
    }

    ret = memcmp(
        core::ptr::addr_of!((*a).rs_desc) as *const core::ffi::c_void,
        core::ptr::addr_of!((*b).rs_desc) as *const core::ffi::c_void,
        core::mem::size_of::<VkPipelineRasterizationStateCreateInfo>() as size_t,
    );
    if ret != 0 {
        return ret;
    }

    ret = wined3d_uint32_compare(
        (*a).ms_desc.rasterizationSamples as uint32_t,
        (*b).ms_desc.rasterizationSamples as uint32_t,
    );
    if ret != 0 {
        return ret;
    }
    ret = wined3d_uint32_compare(
        (*a).ms_desc.alphaToCoverageEnable as uint32_t,
        (*b).ms_desc.alphaToCoverageEnable as uint32_t,
    );
    if ret != 0 {
        return ret;
    }
    ret = wined3d_uint32_compare((*a).sample_mask as uint32_t, (*b).sample_mask as uint32_t);
    if ret != 0 {
        return ret;
    }

    ret = memcmp(
        core::ptr::addr_of!((*a).ds_desc) as *const core::ffi::c_void,
        core::ptr::addr_of!((*b).ds_desc) as *const core::ffi::c_void,
        core::mem::size_of::<VkPipelineDepthStencilStateCreateInfo>() as size_t,
    );
    if ret != 0 {
        return ret;
    }

    ret = wined3d_uint32_compare(
        (*a).blend_desc.attachmentCount,
        (*b).blend_desc.attachmentCount,
    );
    if ret != 0 {
        return ret;
    }
    ret = memcmp(
        core::ptr::addr_of!((*a).blend_attachments) as *const core::ffi::c_void,
        core::ptr::addr_of!((*b).blend_attachments) as *const core::ffi::c_void,
        ((*a).blend_desc.attachmentCount as size_t)
            .wrapping_mul(core::mem::size_of::<VkPipelineColorBlendAttachmentState>() as size_t),
    );
    if ret != 0 {
        return ret;
    }

    ret = wined3d_uint64_compare(
        (*a).pipeline_desc.layout as uint64_t,
        (*b).pipeline_desc.layout as uint64_t,
    );
    if ret != 0 {
        return ret;
    }

    ret = wined3d_uint64_compare(
        (*a).pipeline_desc.renderPass as uint64_t,
        (*b).pipeline_desc.renderPass as uint64_t,
    );
    if ret != 0 {
        return ret;
    }

    return 0 as core::ffi::c_int;
}

unsafe extern "C" fn wined3d_bo_slab_vk_compare(
    mut key: *const ::core::ffi::c_void,
    mut entry: *const rb_entry,
) -> ::core::ffi::c_int {
    let mut slab: *const wined3d_bo_slab_vk = (entry as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(wined3d_bo_slab_vk, entry) as isize))
        as *const wined3d_bo_slab_vk;
    let mut k: *const wined3d_bo_slab_vk_key = key as *const wined3d_bo_slab_vk_key;
    let mut ret: ::core::ffi::c_int = 0;
    ret = wined3d_uint32_compare(
        (*k).memory_type as u32,
        (*slab).requested_memory_type as u32,
    );
    if ret != 0 {
        return ret;
    }
    ret = wined3d_uint32_compare((*k).usage as u32, (*slab).bo.usage as u32);
    if ret != 0 {
        return ret;
    }
    return wined3d_uint64_compare((*k).size as u64, (*slab).bo.size as u64);
}

unsafe extern "C" fn wined3d_context_vk_init_graphics_pipeline_key(
    mut context_vk: *mut wined3d_context_vk,
) {
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut dynamic_states: *mut VkDynamicState =
        &raw mut (*context_vk).dynamic_states as *mut VkDynamicState;
    let mut key: *mut wined3d_graphics_pipeline_key_vk =
        ::core::ptr::null_mut::<wined3d_graphics_pipeline_key_vk>();
    let mut stage: *mut VkPipelineShaderStageCreateInfo =
        ::core::ptr::null_mut::<VkPipelineShaderStageCreateInfo>();
    let mut dynamic_state_count: uint32_t = 0 as uint32_t;
    let mut i: ::core::ffi::c_uint = 0;

    *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_BLEND_CONSTANTS;
    dynamic_state_count += 1;
    *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_STENCIL_REFERENCE;
    dynamic_state_count += 1;
    *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_VIEWPORT;
    dynamic_state_count += 1;
    *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_SCISSOR;
    dynamic_state_count += 1;

    if (*vk_info).supported[WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE as usize] != 0 {
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_DEPTH_TEST_ENABLE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_DEPTH_WRITE_ENABLE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_DEPTH_COMPARE_OP_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_STENCIL_TEST_ENABLE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_STENCIL_OP_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_STENCIL_COMPARE_MASK;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_STENCIL_WRITE_MASK;
        dynamic_state_count += 1;
    }
    if (*vk_info).dynamic_state2 {
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_PRIMITIVE_TOPOLOGY_EXT;
        dynamic_state_count += 1;
        if ((*(*context_vk).c.d3d_info).wined3d_creation_flags
            & (WINED3D_NO_PRIMITIVE_RESTART as uint32_t))
            == 0
        {
            *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_PRIMITIVE_RESTART_ENABLE_EXT;
            dynamic_state_count += 1;
        }
    }
    if (*vk_info).dynamic_patch_vertex_count {
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_PATCH_CONTROL_POINTS_EXT;
        dynamic_state_count += 1;
    }
    if (*vk_info).dynamic_multisample_state {
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_ALPHA_TO_COVERAGE_ENABLE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_RASTERIZATION_SAMPLES_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_SAMPLE_MASK_EXT;
        dynamic_state_count += 1;
    }
    if (*vk_info).dynamic_blend_state {
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_COLOR_BLEND_ENABLE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_COLOR_BLEND_EQUATION_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_COLOR_WRITE_MASK_EXT;
        dynamic_state_count += 1;
    }
    if (*vk_info).dynamic_rasterizer_state {
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_DEPTH_CLAMP_ENABLE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_RASTERIZER_DISCARD_ENABLE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_CULL_MODE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_FRONT_FACE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_DEPTH_BIAS_ENABLE_EXT;
        dynamic_state_count += 1;
        *dynamic_states.add(dynamic_state_count as usize) = VK_DYNAMIC_STATE_DEPTH_BIAS;
        dynamic_state_count += 1;
    }

    key = &raw mut (*context_vk).graphics.pipeline_key_vk;
    memset(
        key as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<wined3d_graphics_pipeline_key_vk>() as size_t,
    );

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < (ARRAY_SIZE!((*context_vk).graphics.vk_modules) as usize) {
        stage = (&raw mut (*key).stages as *mut VkPipelineShaderStageCreateInfo).add(i as usize)
            as *mut VkPipelineShaderStageCreateInfo;
        (*stage).sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
        (*stage).pName = b"main\0".as_ptr() as *const ::core::ffi::c_char;
        i = i.wrapping_add(1);
    }

    (*key).input_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
    (*key).input_desc.pVertexBindingDescriptions =
        &raw mut (*key).bindings as *mut VkVertexInputBindingDescription;
    (*key).input_desc.pVertexAttributeDescriptions =
        &raw mut (*key).attributes as *mut VkVertexInputAttributeDescription;

    (*key).divisor_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT;
    (*key).divisor_desc.pVertexBindingDivisors =
        &raw mut (*key).divisors as *mut VkVertexInputBindingDivisorDescriptionEXT;

    (*key).ia_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;

    (*key).ts_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO;

    (*key).vp_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO;
    (*key).vp_desc.viewportCount =
        (if (*(*context_vk).vk_info).multiple_viewports as ::core::ffi::c_int != 0 {
            WINED3D_MAX_VIEWPORTS
        } else {
            1
        }) as uint32_t;
    (*key).vp_desc.scissorCount = (*key).vp_desc.viewportCount;

    (*key).rs_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
    (*key).rs_desc.lineWidth = 1.0f32;

    (*key).ms_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
    (*key).ms_desc.pSampleMask = &raw mut (*key).sample_mask;
    /* This has to be initialized to a nonzero value even if it's dynamic. */
    (*key).ms_desc.rasterizationSamples = 1;

    (*key).ds_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
    (*key).ds_desc.maxDepthBounds = 1.0f32;

    (*key).blend_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
    (*key).blend_desc.logicOp = VK_LOGIC_OP_COPY;
    (*key).blend_desc.pAttachments =
        &raw mut (*key).blend_attachments as *mut VkPipelineColorBlendAttachmentState;
    (*key).blend_desc.blendConstants[0 as usize] = 1.0f32;
    (*key).blend_desc.blendConstants[1 as usize] = 1.0f32;
    (*key).blend_desc.blendConstants[2 as usize] = 1.0f32;
    (*key).blend_desc.blendConstants[3 as usize] = 1.0f32;

    (*key).dynamic_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO;
    (*key).dynamic_desc.dynamicStateCount = dynamic_state_count;
    (*key).dynamic_desc.pDynamicStates = dynamic_states;

    (*key).pipeline_desc.sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
    (*key).pipeline_desc.pStages = &raw mut (*key).stages as *mut VkPipelineShaderStageCreateInfo;
    (*key).pipeline_desc.pVertexInputState = &raw mut (*key).input_desc;
    (*key).pipeline_desc.pInputAssemblyState = &raw mut (*key).ia_desc;
    (*key).pipeline_desc.pTessellationState = &raw mut (*key).ts_desc;
    (*key).pipeline_desc.pViewportState = &raw mut (*key).vp_desc;
    (*key).pipeline_desc.pRasterizationState = &raw mut (*key).rs_desc;
    (*key).pipeline_desc.pMultisampleState = &raw mut (*key).ms_desc;
    (*key).pipeline_desc.pDepthStencilState = &raw mut (*key).ds_desc;
    (*key).pipeline_desc.pColorBlendState = &raw mut (*key).blend_desc;
    (*key).pipeline_desc.pDynamicState = &raw mut (*key).dynamic_desc;
    (*key).pipeline_desc.basePipelineIndex = -(1 as ::core::ffi::c_int) as int32_t;
}

unsafe extern "C" fn rasterizer_state_from_wined3d(
    mut desc: *mut VkPipelineRasterizationStateCreateInfo,
    mut state: *const wined3d_state,
    mut d3d_info: *const wined3d_d3d_info,
) {
    let mut r: *const wined3d_rasterizer_state_desc =
        ::core::ptr::null::<wined3d_rasterizer_state_desc>();
    let mut scale_bias: f32;
    let mut const_bias: u32;
    let mut const_bias_f32: f32;
    if (*state).rasterizer_state.is_null() {
        (*desc).depthClampEnable = VK_FALSE as VkBool32;
        (*desc).rasterizerDiscardEnable = is_rasterization_disabled(
            (*state).shader[WINED3D_SHADER_TYPE_GEOMETRY as usize],
        ) as VkBool32;
        (*desc).cullMode = VK_CULL_MODE_BACK_BIT as VkCullModeFlags;
        (*desc).frontFace = VK_FRONT_FACE_CLOCKWISE as VkFrontFace;
        (*desc).depthBiasEnable = VK_FALSE as VkBool32;
        (*desc).depthBiasConstantFactor = 0.0f32;
        (*desc).depthBiasClamp = 0.0f32;
        (*desc).depthBiasSlopeFactor = 0.0f32;
        return;
    }
    r = &raw const (*(*state).rasterizer_state).desc;
    (*desc).depthClampEnable = ((*r).depth_clip == 0) as ::core::ffi::c_int as VkBool32;
    (*desc).rasterizerDiscardEnable = is_rasterization_disabled(
        (*state).shader[WINED3D_SHADER_TYPE_GEOMETRY as usize],
    ) as VkBool32;
    (*desc).cullMode = vk_cull_mode_from_wined3d((*r).cull_mode);
    (*desc).frontFace = (if (*r).front_ccw != 0 {
        VK_FRONT_FACE_COUNTER_CLOCKWISE as ::core::ffi::c_int
    } else {
        VK_FRONT_FACE_CLOCKWISE as ::core::ffi::c_int
    }) as VkFrontFace;

    scale_bias = (*r).scale_bias;
    const_bias = (*r).depth_bias.to_bits();
    const_bias_f32 = f32::from_bits(const_bias);
    if scale_bias == 0.0 && const_bias_f32 == 0.0 {
        (*desc).depthBiasEnable = VK_FALSE as VkBool32;
        (*desc).depthBiasConstantFactor = 0.0f32;
        (*desc).depthBiasClamp = 0.0f32;
        (*desc).depthBiasSlopeFactor = 0.0f32;
        return;
    }

    (*desc).depthBiasEnable = VK_TRUE as VkBool32;
    if (*d3d_info).wined3d_creation_flags & WINED3D_LEGACY_DEPTH_BIAS as uint32_t != 0 {
        let mut dsv: *const wined3d_rendertarget_view =
            ::core::ptr::null::<wined3d_rendertarget_view>();

        dsv = (*state).fb.depth_stencil;
        if !dsv.is_null() {
            (*desc).depthBiasConstantFactor =
                -(const_bias as ::core::ffi::c_float) / (*(*dsv).format).depth_bias_scale;
            (*desc).depthBiasSlopeFactor = -(const_bias as ::core::ffi::c_float);
        } else {
            (*desc).depthBiasConstantFactor = 0.0f32;
            (*desc).depthBiasSlopeFactor = 0.0f32;
        }
    } else {
        (*desc).depthBiasConstantFactor = const_bias_f32;
        (*desc).depthBiasSlopeFactor = scale_bias;
    }
    (*desc).depthBiasClamp = (*r).depth_bias_clamp;
}

unsafe extern "C" fn wined3d_context_vk_set_dynamic_rasterizer_state(
    context_vk: *const wined3d_context_vk,
    vk_command_buffer: VkCommandBuffer,
    vk_info: *const wined3d_vk_info,
    state: *const wined3d_state,
) {
    let mut desc: VkPipelineRasterizationStateCreateInfo = core::mem::zeroed();

    rasterizer_state_from_wined3d(&raw mut desc, state, (*context_vk).c.d3d_info);

    (*vk_info)
        .vk_ops
        .vkCmdSetRasterizerDiscardEnableEXT
        .unwrap_unchecked()(vk_command_buffer, desc.rasterizerDiscardEnable);
    (*vk_info)
        .vk_ops
        .vkCmdSetDepthClampEnableEXT
        .unwrap_unchecked()(vk_command_buffer, desc.depthClampEnable);
    (*vk_info)
        .vk_ops
        .vkCmdSetCullModeEXT
        .unwrap_unchecked()(vk_command_buffer, desc.cullMode);
    (*vk_info)
        .vk_ops
        .vkCmdSetFrontFaceEXT
        .unwrap_unchecked()(vk_command_buffer, desc.frontFace);
    (*vk_info)
        .vk_ops
        .vkCmdSetDepthBiasEnableEXT
        .unwrap_unchecked()(vk_command_buffer, desc.depthBiasEnable);

    if desc.depthBiasEnable != 0 {
        (*vk_info)
            .vk_ops
            .vkCmdSetDepthBias
            .unwrap_unchecked()(
            vk_command_buffer,
            desc.depthBiasConstantFactor,
            desc.depthBiasClamp,
            desc.depthBiasSlopeFactor,
        );
    }
}

unsafe extern "C" fn blend_equation_from_wined3d(
    context_vk: *const wined3d_context_vk,
    eq: *mut VkColorBlendEquationEXT,
    rt: *const wined3d_rendertarget_blend_state_desc,
    rtv: *const wined3d_rendertarget_view,
) {
    let mut src_blend: wined3d_blend = 0;
    let mut dst_blend: wined3d_blend = 0;
    let mut rt_format: *const wined3d_format = core::ptr::null::<wined3d_format>();
    if !rtv.is_null() {
        rt_format = (*rtv).format;
    } else {
        rt_format = wined3d_get_format(
            (*(*context_vk).c.device).adapter,
            WINED3DFMT_NULL,
            0,
        );
    }
    src_blend = (*rt).src;
    dst_blend = (*rt).dst;
    if src_blend == WINED3D_BLEND_BOTHSRCALPHA {
        src_blend = WINED3D_BLEND_SRCALPHA;
        dst_blend = WINED3D_BLEND_INVSRCALPHA;
    } else if src_blend == WINED3D_BLEND_BOTHINVSRCALPHA {
        src_blend = WINED3D_BLEND_INVSRCALPHA;
        dst_blend = WINED3D_BLEND_SRCALPHA;
    }
    (*eq).srcColorBlendFactor = vk_blend_factor_from_wined3d(src_blend, rt_format, false);
    (*eq).dstColorBlendFactor = vk_blend_factor_from_wined3d(dst_blend, rt_format, false);
    (*eq).colorBlendOp = vk_blend_op_from_wined3d((*rt).op);

    src_blend = (*rt).src_alpha;
    dst_blend = (*rt).dst_alpha;
    (*eq).srcAlphaBlendFactor = vk_blend_factor_from_wined3d(src_blend, rt_format, true);
    (*eq).dstAlphaBlendFactor = vk_blend_factor_from_wined3d(dst_blend, rt_format, true);
    (*eq).alphaBlendOp = vk_blend_op_from_wined3d((*rt).op_alpha);
}

unsafe extern "C" fn wined3d_context_vk_update_blend_state(
    context_vk: *const wined3d_context_vk,
    state: *const wined3d_state,
    key: *mut wined3d_graphics_pipeline_key_vk,
) {
    let desc: *mut VkPipelineColorBlendStateCreateInfo = core::ptr::addr_of_mut!((*key).blend_desc);
    let mut b: *const wined3d_blend_state_desc = core::ptr::null::<wined3d_blend_state_desc>();
    let mut i: ::core::ffi::c_uint = 0;

    (*desc).attachmentCount = (*context_vk).rt_count as uint32_t;

    memset(
        core::ptr::addr_of_mut!((*key).blend_attachments) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[VkPipelineColorBlendAttachmentState; 8]>() as size_t,
    );
    if (*state).blend_state.is_null() {
        i = 0 as ::core::ffi::c_uint;
        while i < (*context_vk).rt_count {
            (*key).blend_attachments[i as usize].colorWriteMask = (VK_COLOR_COMPONENT_R_BIT as ::core::ffi::c_int
                | VK_COLOR_COMPONENT_G_BIT as ::core::ffi::c_int
                | VK_COLOR_COMPONENT_B_BIT as ::core::ffi::c_int
                | VK_COLOR_COMPONENT_A_BIT as ::core::ffi::c_int)
                as VkColorComponentFlags;
            i = i.wrapping_add(1);
        }

        return;
    }

    b = core::ptr::addr_of!((*(*state).blend_state).desc);
    i = 0 as ::core::ffi::c_uint;
    while i < (*context_vk).rt_count {
        let rt: *const wined3d_rendertarget_blend_state_desc =
            (core::ptr::addr_of!((*b).rt) as *const wined3d_rendertarget_blend_state_desc).offset(
                (if (*b).independent != 0 {
                    i
                } else {
                    0 as ::core::ffi::c_uint
                }) as isize,
            ) as *const wined3d_rendertarget_blend_state_desc;
        let a: *mut VkPipelineColorBlendAttachmentState =
            (core::ptr::addr_of_mut!((*key).blend_attachments) as *mut VkPipelineColorBlendAttachmentState)
                .offset(i as isize) as *mut VkPipelineColorBlendAttachmentState;

        (*a).colorWriteMask = vk_colour_write_mask_from_wined3d((*rt).writemask as uint32_t);
        if (*rt).enable != 0 {
            (*a).blendEnable = VK_TRUE as VkBool32;

            blend_equation_from_wined3d(
                context_vk,
                core::ptr::addr_of_mut!((*a).srcColorBlendFactor) as *mut VkColorBlendEquationEXT,
                rt,
                (*state).fb.render_targets[i as usize],
            );
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_context_vk_set_dynamic_blend_state(
    context_vk: *const wined3d_context_vk,
    vk_command_buffer: VkCommandBuffer,
    vk_info: *const wined3d_vk_info,
    state: *const wined3d_state,
) {
    let mut equations: [VkColorBlendEquationEXT; (WINED3D_MAX_RENDER_TARGETS as _)] = core::mem::zeroed();
    equations[0].srcColorBlendFactor = 0;
    let mut write_mask: [VkColorComponentFlags; (WINED3D_MAX_RENDER_TARGETS as _)] = [0; (WINED3D_MAX_RENDER_TARGETS as _)];
    let mut rt_count: ::core::ffi::c_uint = (*context_vk).rt_count;
    let mut enable: [VkBool32; (WINED3D_MAX_RENDER_TARGETS as _)] = [0; (WINED3D_MAX_RENDER_TARGETS as _)];
    let mut b: *const wined3d_blend_state_desc = ::core::ptr::null::<wined3d_blend_state_desc>();

    if (*state).blend_state.is_null() {
        static mut default_enable: [VkBool32; (WINED3D_MAX_RENDER_TARGETS as _)] = [0; (WINED3D_MAX_RENDER_TARGETS as _)];
        const X: VkColorComponentFlags = (VK_COLOR_COMPONENT_R_BIT as ::core::ffi::c_int
            | VK_COLOR_COMPONENT_G_BIT as ::core::ffi::c_int
            | VK_COLOR_COMPONENT_B_BIT as ::core::ffi::c_int
            | VK_COLOR_COMPONENT_A_BIT as ::core::ffi::c_int) as VkColorComponentFlags;
        static mut default_write_mask: [VkColorComponentFlags; (WINED3D_MAX_RENDER_TARGETS as _)] =
            [X; (WINED3D_MAX_RENDER_TARGETS as _)];

        (*vk_info)
            .vk_ops
            .vkCmdSetColorBlendEnableEXT
            .unwrap_unchecked()(
            vk_command_buffer,
            0 as uint32_t,
            rt_count as uint32_t,
            &raw const default_enable as *const VkBool32,
        );
        (*vk_info)
            .vk_ops
            .vkCmdSetColorWriteMaskEXT
            .unwrap_unchecked()(
            vk_command_buffer,
            0 as uint32_t,
            rt_count as uint32_t,
            &raw const default_write_mask as *const VkColorComponentFlags,
        );
        return;
    }

    b = &raw mut (*(*state).blend_state).desc;
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i < rt_count {
        let mut rt: *const wined3d_rendertarget_blend_state_desc =
            (&raw const (*b).rt as *const wined3d_rendertarget_blend_state_desc).offset(
                (if (*b).independent != 0 {
                    i
                } else {
                    0 as ::core::ffi::c_uint
                }) as isize,
            ) as *const wined3d_rendertarget_blend_state_desc;

        write_mask[i as usize] =
            vk_colour_write_mask_from_wined3d((*rt).writemask as uint32_t);
        enable[i as usize] = (*rt).enable as VkBool32;

        if !((*rt).enable == 0) {
            blend_equation_from_wined3d(
                context_vk,
                (&raw mut equations as *mut VkColorBlendEquationEXT).offset(i as isize)
                    as *mut VkColorBlendEquationEXT,
                rt,
                (*state).fb.render_targets[i as usize],
            );
        }
        i = i.wrapping_add(1);
    }

    (*vk_info)
        .vk_ops
        .vkCmdSetColorBlendEnableEXT
        .unwrap_unchecked()(
        vk_command_buffer,
        0 as uint32_t,
        rt_count as uint32_t,
        &raw mut enable as *mut VkBool32,
    );
    (*vk_info)
        .vk_ops
        .vkCmdSetColorWriteMaskEXT
        .unwrap_unchecked()(
        vk_command_buffer,
        0 as uint32_t,
        rt_count as uint32_t,
        &raw mut write_mask as *mut VkColorComponentFlags,
    );
    (*vk_info)
        .vk_ops
        .vkCmdSetColorBlendEquationEXT
        .unwrap_unchecked()(
        vk_command_buffer,
        0 as uint32_t,
        rt_count as uint32_t,
        &raw mut equations as *mut VkColorBlendEquationEXT,
    );
}

unsafe extern "C" fn vk_format_from_component_type(component_type: wined3d_component_type) -> VkFormat {
    let ct = component_type as ::core::ffi::c_uint;
    if ct == WINED3D_TYPE_UINT {
        return VK_FORMAT_R32G32B32A32_UINT;
    }
    if ct == WINED3D_TYPE_INT {
        return VK_FORMAT_R32G32B32A32_SINT;
    }
    if ct == WINED3D_TYPE_UNKNOWN || ct == WINED3D_TYPE_FLOAT {
        return VK_FORMAT_R32G32B32A32_SFLOAT;
    }
    return VK_FORMAT_UNDEFINED;
}

unsafe extern "C" fn wined3d_context_vk_update_graphics_pipeline_key(
    context_vk: *mut wined3d_context_vk,
    state: *const wined3d_state,
    vk_pipeline_layout: VkPipelineLayout,
    null_buffer_binding: *mut uint32_t,
) -> bool {
    let mut i: ::core::ffi::c_uint = 0;
    let mut attribute_count: ::core::ffi::c_uint = 0;
    let mut binding_count: ::core::ffi::c_uint = 0;
    let mut divisor_count: ::core::ffi::c_uint = 0;
    let mut stage_count: ::core::ffi::c_uint = 0;
    let d3d_info: *const wined3d_d3d_info = (*context_vk).c.d3d_info;
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut key: *mut wined3d_graphics_pipeline_key_vk = core::ptr::null_mut();
    let mut stage: *mut VkPipelineShaderStageCreateInfo = core::ptr::null_mut();
    let mut stream_info: wined3d_stream_info = core::mem::zeroed();
    let mut vertex_shader: *mut wined3d_shader = core::ptr::null_mut();
    let mut vk_topology: VkPrimitiveTopology = 0;
    let mut module: VkShaderModule = 0;
    let mut update: bool = false;
    let mut mask: uint32_t = 0;

    *null_buffer_binding = !0;
    key = &raw mut (*context_vk).graphics.pipeline_key_vk;

    if (*context_vk).c.shader_update_mask() & !(1 << WINED3D_SHADER_TYPE_COMPUTE) != 0
    {
        stage_count = 0;
        i = 0;
        while i < ARRAY_SIZE!((*context_vk).graphics.vk_modules) {
            'body: {
                module = (*context_vk).graphics.vk_modules[i as usize];
                if module == 0 {
                    break 'body;
                }

                let fresh = stage_count;
                stage_count = stage_count.wrapping_add(1);
                stage = (&raw mut (*key).stages as *mut VkPipelineShaderStageCreateInfo)
                    .offset(fresh as isize);
                (*stage).stage = vk_shader_stage_from_wined3d(i as wined3d_shader_type);
                (*stage).module = module;
            }
            i = i.wrapping_add(1);
        }

        (*key).pipeline_desc.stageCount = stage_count as uint32_t;

        update = true;
    }

    if wined3d_context_is_graphics_state_dirty(&raw mut (*context_vk).c, (STATE_VDECL as u32)) as ::core::ffi::c_int != 0
        || wined3d_context_is_graphics_state_dirty(&raw mut (*context_vk).c, (STATE_STREAMSRC as u32)) as ::core::ffi::c_int != 0
        || wined3d_context_is_graphics_state_dirty(&raw mut (*context_vk).c, STATE_SHADER!((WINED3D_SHADER_TYPE_VERTEX) as i32)) as ::core::ffi::c_int != 0
    {
        let mut a: *mut VkVertexInputAttributeDescription = core::ptr::null_mut();
        let mut b: *mut VkVertexInputBindingDescription = core::ptr::null_mut();

        wined3d_stream_info_from_declaration(&raw mut stream_info, state, d3d_info);
        divisor_count = 0;
        i = 0;
        mask = 0;
        attribute_count = 0;
        binding_count = 0;
        while i < ARRAY_SIZE!(stream_info.elements) {
            let mut d: *mut VkVertexInputBindingDivisorDescriptionEXT = core::ptr::null_mut();
            let mut e: *mut wined3d_stream_info_element = core::ptr::null_mut();
            let mut binding: uint32_t = 0;

            'body: {
                if (stream_info.use_map & (1 << i)) == 0 {
                    break 'body;
                }

                let fresh = attribute_count;
                attribute_count = attribute_count.wrapping_add(1);
                a = (&raw mut (*key).attributes as *mut VkVertexInputAttributeDescription)
                    .offset(fresh as isize);
                e = (&raw mut stream_info.elements as *mut wined3d_stream_info_element)
                    .offset(i as isize);
                binding = (*e).stream_idx as uint32_t;

                (*a).location = i as uint32_t;
                (*a).binding = binding;
                (*a).format = (*wined3d_format_vk((*e).format)).vk_format;
                (*a).offset = ((*e).data.addr as UINT_PTR)
                    .wrapping_sub((*state).streams[binding as usize].offset as UINT_PTR)
                    as uint32_t;

                if (mask & (1 << binding)) != 0 {
                    break 'body;
                }
                mask |= 1 << binding;

                let fresh = binding_count;
                binding_count = binding_count.wrapping_add(1);
                b = (&raw mut (*key).bindings as *mut VkVertexInputBindingDescription)
                    .offset(fresh as isize);
                (*b).binding = binding;
                (*b).stride = (*e).stride as uint32_t;
                (*b).inputRate = (if (*e).instanced {
                    VK_VERTEX_INPUT_RATE_INSTANCE as ::core::ffi::c_int
                } else {
                    VK_VERTEX_INPUT_RATE_VERTEX as ::core::ffi::c_int
                }) as VkVertexInputRate;

                if (*e).instanced {
                    let fresh = divisor_count;
                    divisor_count = divisor_count.wrapping_add(1);
                    d = (&raw mut (*key).divisors as *mut VkVertexInputBindingDivisorDescriptionEXT)
                        .offset(fresh as isize);
                    (*d).binding = binding;
                    (*d).divisor = (*e).divisor as uint32_t;
                }
            }
            i = i.wrapping_add(1);
        }

        vertex_shader = (*state).shader[WINED3D_SHADER_TYPE_VERTEX as usize];
        if !vertex_shader.is_null() && {
            mask = (!stream_info.use_map & (*vertex_shader).reg_maps.input_registers) as uint32_t;
            mask != 0
        } {
            let mut element: *mut wined3d_shader_signature_element = core::ptr::null_mut();
            let mut signature: *mut wined3d_shader_signature = core::ptr::null_mut();
            let mut null_binding: uint32_t = 0;
            let mut location: uint32_t = 0;

            if !wined3d_get_unused_stream_index(state, &raw mut null_binding) {
                ERR!("No streams left for a null buffer binding.\n");
            } else {
                signature = &raw mut (*vertex_shader).input_signature;
                i = 0;
                while i < (*signature).element_count {
                    'body: {
                        element = (*signature).elements.offset(i as isize)
                            as *mut wined3d_shader_signature_element;
                        location = (*element).register_idx as uint32_t;

                        if (mask & (1 << location)) == 0
                            || (*element).sysval_semantic as ::core::ffi::c_uint != 0
                        {
                            break 'body;
                        }
                        mask &= !(1 << location);

                        let fresh = attribute_count;
                        attribute_count = attribute_count.wrapping_add(1);
                        a = (&raw mut (*key).attributes as *mut VkVertexInputAttributeDescription)
                            .offset(fresh as isize);
                        (*a).location = location;
                        (*a).binding = null_binding;
                        (*a).format = vk_format_from_component_type((*element).component_type);
                        (*a).offset = 0;
                    }
                    i = i.wrapping_add(1);
                }

                if mask
                    != (!stream_info.use_map as uint32_t
                        & (*vertex_shader).reg_maps.input_registers as uint32_t)
                {
                    let fresh = binding_count;
                    binding_count = binding_count.wrapping_add(1);
                    b = (&raw mut (*key).bindings as *mut VkVertexInputBindingDescription)
                        .offset(fresh as isize);
                    (*b).binding = null_binding;
                    *null_buffer_binding = (*b).binding;
                    (*b).stride = 0;
                    (*b).inputRate = VK_VERTEX_INPUT_RATE_VERTEX;
                }
            }
        }

        (*key).input_desc.pNext = core::ptr::null::<::core::ffi::c_void>();
        (*key).input_desc.vertexBindingDescriptionCount = binding_count as uint32_t;
        (*key).input_desc.vertexAttributeDescriptionCount = attribute_count as uint32_t;

        if divisor_count != 0 {
            (*key).input_desc.pNext = &raw mut (*key).divisor_desc as *const ::core::ffi::c_void;
            (*key).divisor_desc.vertexBindingDivisorCount = divisor_count as uint32_t;
        }

        update = true;
    }

    if (*vk_info).dynamic_state2 {
        vk_topology = vk_topology_class_from_wined3d((*state).primitive_type);
        if (*key).ia_desc.topology as ::core::ffi::c_uint != vk_topology as ::core::ffi::c_uint {
            (*key).ia_desc.topology = vk_topology;
            update = true;
        }
    } else {
        vk_topology = vk_topology_from_wined3d((*state).primitive_type);
        if (*key).ia_desc.topology as ::core::ffi::c_uint != vk_topology as ::core::ffi::c_uint {
            (*key).ia_desc.topology = vk_topology;
            (*key).ia_desc.primitiveRestartEnable = ((((*d3d_info).wined3d_creation_flags
                & WINED3D_NO_PRIMITIVE_RESTART)
                == 0)
                && !wined3d_primitive_type_is_list((*state).primitive_type))
                as ::core::ffi::c_int as VkBool32;

            update = true;
        }
    }

    if !(*vk_info).dynamic_patch_vertex_count
        && (*key).ts_desc.patchControlPoints != (*state).patch_vertex_count as uint32_t
    {
        (*key).ts_desc.patchControlPoints = (*state).patch_vertex_count as uint32_t;

        update = true;
    }

    if !(*vk_info).dynamic_rasterizer_state
        && (wined3d_context_is_graphics_state_dirty(&raw mut (*context_vk).c, (STATE_RASTERIZER as u32)) as ::core::ffi::c_int != 0
            || wined3d_context_is_graphics_state_dirty(&raw mut (*context_vk).c, STATE_SHADER!((WINED3D_SHADER_TYPE_GEOMETRY) as i32)) as ::core::ffi::c_int != 0)
    {
        rasterizer_state_from_wined3d(&raw mut (*key).rs_desc, state, d3d_info);

        update = true;
    }

    if !(*vk_info).dynamic_multisample_state
        && ((*key).ms_desc.rasterizationSamples as ::core::ffi::c_uint
            != (*context_vk).sample_count as ::core::ffi::c_uint
            || isStateDirty(&raw mut (*context_vk).c, (STATE_BLEND as u32)) as ::core::ffi::c_int != 0
            || isStateDirty(&raw mut (*context_vk).c, (STATE_SAMPLE_MASK as u32)) as ::core::ffi::c_int != 0)
    {
        (*key).ms_desc.rasterizationSamples = (*context_vk).sample_count;
        (*key).ms_desc.alphaToCoverageEnable = (!(*state).blend_state.is_null()
            && (*(*state).blend_state).desc.alpha_to_coverage != 0)
            as ::core::ffi::c_int as VkBool32;
        (*key).sample_mask = (*state).sample_mask as VkSampleMask;

        update = true;
    }

    if (*vk_info).supported[WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE as usize] == 0
        && (wined3d_context_is_graphics_state_dirty(&raw mut (*context_vk).c, (STATE_DEPTH_STENCIL as u32)) as ::core::ffi::c_int != 0
            || wined3d_context_is_graphics_state_dirty(&raw mut (*context_vk).c, (STATE_FRAMEBUFFER as u32)) as ::core::ffi::c_int != 0)
    {
        let d: *const wined3d_depth_stencil_state = (*state).depth_stencil_state;

        if !d.is_null() {
            (*key).ds_desc.depthTestEnable = (*d).desc.depth as VkBool32;
            (*key).ds_desc.depthWriteEnable = (*d).desc.depth_write as VkBool32;
            (*key).ds_desc.depthCompareOp = vk_compare_op_from_wined3d((*d).desc.depth_func);
            (*key).ds_desc.stencilTestEnable = (!(*state).fb.depth_stencil.is_null()
                && (*d).desc.stencil != 0)
                as ::core::ffi::c_int as VkBool32;
            if (*key).ds_desc.stencilTestEnable != 0 {
                (*key).ds_desc.front.failOp = vk_stencil_op_from_wined3d((*d).desc.front.fail_op);
                (*key).ds_desc.front.passOp = vk_stencil_op_from_wined3d((*d).desc.front.pass_op);
                (*key).ds_desc.front.depthFailOp =
                    vk_stencil_op_from_wined3d((*d).desc.front.depth_fail_op);
                (*key).ds_desc.front.compareOp = vk_compare_op_from_wined3d((*d).desc.front.func);
                (*key).ds_desc.front.compareMask = (*d).desc.stencil_read_mask as uint32_t;
                (*key).ds_desc.front.writeMask = (*d).desc.stencil_write_mask as uint32_t;

                (*key).ds_desc.back.failOp = vk_stencil_op_from_wined3d((*d).desc.back.fail_op);
                (*key).ds_desc.back.passOp = vk_stencil_op_from_wined3d((*d).desc.back.pass_op);
                (*key).ds_desc.back.depthFailOp =
                    vk_stencil_op_from_wined3d((*d).desc.back.depth_fail_op);
                (*key).ds_desc.back.compareOp = vk_compare_op_from_wined3d((*d).desc.back.func);
                (*key).ds_desc.back.compareMask = (*d).desc.stencil_read_mask as uint32_t;
                (*key).ds_desc.back.writeMask = (*d).desc.stencil_write_mask as uint32_t;
            } else {
                memset(
                    &raw mut (*key).ds_desc.front as *mut ::core::ffi::c_void,
                    0,
                    core::mem::size_of::<VkStencilOpState>() as size_t,
                );
                memset(
                    &raw mut (*key).ds_desc.back as *mut ::core::ffi::c_void,
                    0,
                    core::mem::size_of::<VkStencilOpState>() as size_t,
                );
            }
        } else {
            (*key).ds_desc.depthTestEnable = VK_TRUE as VkBool32;
            (*key).ds_desc.depthWriteEnable = VK_TRUE as VkBool32;
            (*key).ds_desc.depthCompareOp = VK_COMPARE_OP_LESS as VkCompareOp;
            (*key).ds_desc.stencilTestEnable = VK_FALSE as VkBool32;
        }

        update = true;
    }

    if !(*vk_info).dynamic_blend_state
        && (wined3d_context_is_graphics_state_dirty(&raw mut (*context_vk).c, (STATE_BLEND as u32)) as ::core::ffi::c_int != 0
            || wined3d_context_is_graphics_state_dirty(&raw mut (*context_vk).c, (STATE_FRAMEBUFFER as u32)) as ::core::ffi::c_int != 0)
    {
        wined3d_context_vk_update_blend_state(context_vk, state, key);

        update = true;
    }

    if (*key).pipeline_desc.layout != vk_pipeline_layout {
        (*key).pipeline_desc.layout = vk_pipeline_layout;

        update = true;
    }

    if (*key).pipeline_desc.renderPass != (*context_vk).vk_render_pass {
        (*key).pipeline_desc.renderPass = (*context_vk).vk_render_pass;

        update = true;
    }

    return update;
}

unsafe extern "C" fn wined3d_context_vk_begin_render_pass(
    context_vk: *mut wined3d_context_vk,
    state: *const wined3d_state,
    vk_info: *const wined3d_vk_info,
) -> bool {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let mut clear_values: [VkClearValue; (WINED3D_MAX_RENDER_TARGETS + 1) as usize] =
        [VkClearValue { color: VkClearColorValue { float32: [0.0; 4] } };
            (WINED3D_MAX_RENDER_TARGETS + 1) as usize];
    let mut vk_views: [VkImageView; (WINED3D_MAX_RENDER_TARGETS + 1) as usize] =
        [0; (WINED3D_MAX_RENDER_TARGETS + 1) as usize];
    let mut fb_width: core::ffi::c_uint = 0;
    let mut fb_height: core::ffi::c_uint = 0;
    let mut fb_layer_count: core::ffi::c_uint = 0;
    let mut rtv_vk: *mut wined3d_rendertarget_view_vk = core::ptr::null_mut();
    let mut view: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut limits: *const VkPhysicalDeviceLimits = core::ptr::null();
    let mut query_vk: *mut wined3d_query_vk = core::ptr::null_mut();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut begin_info: VkRenderPassBeginInfo = core::mem::zeroed();
    let mut attachment_count: core::ffi::c_uint = 0;
    let mut i: core::ffi::c_uint = 0;
    let mut texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut fb_desc: VkFramebufferCreateInfo = core::mem::zeroed();
    let mut vr: VkResult = core::mem::zeroed();

    if !((*context_vk).vk_render_pass == 0) {
        return true;
    }

    limits = core::ptr::addr_of!((*wined3d_adapter_vk((*device_vk).d.adapter)).device_limits);
    fb_width = (*limits).maxFramebufferWidth as core::ffi::c_uint;
    fb_height = (*limits).maxFramebufferHeight as core::ffi::c_uint;
    fb_layer_count = (*limits).maxFramebufferLayers as core::ffi::c_uint;
    attachment_count = 0 as core::ffi::c_uint;

    (*context_vk).rt_count = 0 as core::ffi::c_uint;
    begin_info.clearValueCount = 0 as uint32_t;
    i = 0 as core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!((*state).fb.render_targets) as usize {
        view = (*state).fb.render_targets[i as usize];
        if !(view.is_null() || (*(*view).format).id == WINED3DFMT_NULL as core::ffi::c_uint) {
            rtv_vk = wined3d_rendertarget_view_vk(view);

            if (*(*rtv_vk).v.resource).bind_count != 0 {
                let texture_vk: *mut wined3d_texture_vk;
                texture_vk = wined3d_texture_vk(wined3d_texture_from_resource((*rtv_vk).v.resource));
                wined3d_texture_vk_make_generic(texture_vk, context_vk);
            }

            vk_views[attachment_count as usize] =
                wined3d_rendertarget_view_vk_get_image_view(rtv_vk, context_vk);
            wined3d_rendertarget_view_vk_barrier(rtv_vk, context_vk, WINED3D_BIND_RENDER_TARGET as uint32_t);
            wined3d_context_vk_reference_rendertarget_view(context_vk, rtv_vk);

            if (*view).width < fb_width {
                fb_width = (*view).width;
            }
            if (*view).height < fb_height {
                fb_height = (*view).height;
            }
            if (*view).layer_count < fb_layer_count {
                fb_layer_count = (*view).layer_count;
            }
            (*context_vk).rt_count = i.wrapping_add(1 as core::ffi::c_uint);

            if wined3d_rendertarget_view_get_locations(view) & WINED3D_LOCATION_CLEARED as DWORD != 0 {
                let c: *mut VkClearColorValue = core::ptr::addr_of_mut!(
                    (*clear_values.as_mut_ptr().offset(attachment_count as isize)).color
                );

                if (*(*view).resource).type_ as core::ffi::c_uint
                    == WINED3D_RTYPE_BUFFER as core::ffi::c_uint
                {
                    (*c).int32[0 as usize] =
                        0 as int32_t;
                    (*c).int32[1 as usize] =
                        (*c).int32[0 as usize];
                    (*c).int32[2 as usize] =
                        (*c).int32[1 as usize];
                    (*c).int32[3 as usize] =
                        (*c).int32[2 as usize];
                } else {
                    texture = texture_from_resource((*view).resource);
                    wined3d_format_colour_to_vk(
                        (*view).format,
                        core::ptr::addr_of_mut!(
                            (*(*texture).sub_resources.offset((*view).sub_resource_idx as isize))
                                .clear_value
                                .colour
                        ),
                        c,
                    );
                }

                begin_info.clearValueCount =
                    attachment_count.wrapping_add(1 as core::ffi::c_uint) as uint32_t;
            }
            attachment_count = attachment_count.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    view = (*state).fb.depth_stencil;
    if !view.is_null() {
        rtv_vk = wined3d_rendertarget_view_vk(view);

        if (*(*rtv_vk).v.resource).bind_count != 0 {
            let texture_vk: *mut wined3d_texture_vk;
            texture_vk = wined3d_texture_vk(wined3d_texture_from_resource((*rtv_vk).v.resource));
            wined3d_texture_vk_make_generic(texture_vk, context_vk);
        }

        vk_views[attachment_count as usize] =
            wined3d_rendertarget_view_vk_get_image_view(rtv_vk, context_vk);
        wined3d_rendertarget_view_vk_barrier(rtv_vk, context_vk, WINED3D_BIND_DEPTH_STENCIL as uint32_t);
        wined3d_context_vk_reference_rendertarget_view(context_vk, rtv_vk);

        if (*view).width < fb_width {
            fb_width = (*view).width;
        }
        if (*view).height < fb_height {
            fb_height = (*view).height;
        }
        if (*view).layer_count < fb_layer_count {
            fb_layer_count = (*view).layer_count;
        }

        if wined3d_rendertarget_view_get_locations(view) & WINED3D_LOCATION_CLEARED as DWORD != 0 {
            let c: *mut VkClearDepthStencilValue = core::ptr::addr_of_mut!(
                (*clear_values.as_mut_ptr().offset(attachment_count as isize)).depthStencil
            );

            if (*(*view).resource).type_ as core::ffi::c_uint
                == WINED3D_RTYPE_BUFFER as core::ffi::c_uint
            {
                (*c).depth = 0.0f32;
                (*c).stencil = 0 as uint32_t;
            } else {
                texture = texture_from_resource((*view).resource);
                (*c).depth = (*(*texture).sub_resources.offset((*view).sub_resource_idx as isize))
                    .clear_value

                    .__bindgen_anon_1.depth;
                (*c).stencil = (*(*texture).sub_resources.offset((*view).sub_resource_idx as isize))
                    .clear_value

                    .__bindgen_anon_1.stencil as uint32_t;
            }
            begin_info.clearValueCount =
                attachment_count.wrapping_add(1 as core::ffi::c_uint) as uint32_t;
        }
        attachment_count = attachment_count.wrapping_add(1);
    }

    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to get command buffer.\n");
        return false;
    }

    (*context_vk).vk_render_pass = wined3d_context_vk_get_render_pass(
        context_vk,
        core::ptr::addr_of!((*state).fb),
        ARRAY_SIZE!((*state).fb.render_targets) as core::ffi::c_uint,
        !(*state).fb.depth_stencil.is_null(),
        0 as uint32_t,
    );
    if ((*context_vk).vk_render_pass == 0) {
        ERR!("Failed to get render pass.\n");
        return false;
    }

    fb_desc.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO as VkStructureType;
    fb_desc.pNext = core::ptr::null();
    fb_desc.flags = 0 as VkFramebufferCreateFlags;
    fb_desc.renderPass = (*context_vk).vk_render_pass;
    fb_desc.attachmentCount = attachment_count as uint32_t;
    fb_desc.pAttachments = vk_views.as_mut_ptr();
    fb_desc.width = fb_width as uint32_t;
    fb_desc.height = fb_height as uint32_t;
    fb_desc.layers = fb_layer_count as uint32_t;

    vr = ((*vk_info).vk_ops.vkCreateFramebuffer.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(fb_desc),
        core::ptr::null(),
        core::ptr::addr_of_mut!((*context_vk).vk_framebuffer),
    );
    if (vr as core::ffi::c_int) < 0 as core::ffi::c_int {
        WARN!(
            "Failed to create Vulkan framebuffer, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return false;
    }

    begin_info.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO as VkStructureType;
    begin_info.pNext = core::ptr::null();
    begin_info.renderPass = (*context_vk).vk_render_pass;
    begin_info.framebuffer = (*context_vk).vk_framebuffer;
    begin_info.renderArea.offset.x = 0 as int32_t;
    begin_info.renderArea.offset.y = 0 as int32_t;
    begin_info.renderArea.extent.width = fb_width as uint32_t;
    begin_info.renderArea.extent.height = fb_height as uint32_t;
    begin_info.pClearValues = clear_values.as_mut_ptr();
    ((*vk_info).vk_ops.vkCmdBeginRenderPass.unwrap_unchecked())(
        vk_command_buffer,
        core::ptr::addr_of_mut!(begin_info),
        VK_SUBPASS_CONTENTS_INLINE as VkSubpassContents,
    );

    query_vk = ((*context_vk).render_pass_queries.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_query_vk, entry)) as *mut wined3d_query_vk;
    while core::ptr::addr_of_mut!((*query_vk).entry)
        != core::ptr::addr_of_mut!((*context_vk).render_pass_queries)
    {
        wined3d_query_vk_resume(query_vk, context_vk);
        query_vk = ((*query_vk).entry.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_query_vk, entry)) as *mut wined3d_query_vk;
    }
    return true;
}

unsafe extern "C" fn wined3d_context_vk_bind_vertex_buffers(
    mut context_vk: *mut wined3d_context_vk,
    mut vk_command_buffer: VkCommandBuffer,
    mut state: *const wined3d_state,
    mut vk_info: *const wined3d_vk_info,
) {
    let mut offsets: [VkDeviceSize; 16] = [0; 16];
    let mut buffers: [VkBuffer; 16] = [0; 16];
    let mut stream: *const wined3d_stream_state = ::core::ptr::null::<wined3d_stream_state>();
    let mut buffer_info: *const VkDescriptorBufferInfo =
        ::core::ptr::null::<VkDescriptorBufferInfo>();
    let mut buffer_vk: *mut wined3d_buffer_vk = ::core::ptr::null_mut::<wined3d_buffer_vk>();
    let mut buffer: *mut wined3d_buffer = ::core::ptr::null_mut::<wined3d_buffer>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut first: ::core::ffi::c_uint = 0;
    let mut count: ::core::ffi::c_uint = 0;

    first = 0;
    count = 0;
    i = 0;
    while (i as usize) < (ARRAY_SIZE!((*state).streams) as usize) {
        stream = (&raw const (*state).streams as *const wined3d_stream_state).offset(i as isize)
            as *const wined3d_stream_state;

        buffer = (*stream).buffer as *mut wined3d_buffer;
        if !buffer.is_null() {
            buffer_vk = wined3d_buffer_vk(buffer);
            buffer_info = wined3d_buffer_vk_get_buffer_info(buffer_vk);
            wined3d_context_vk_reference_bo(context_vk, wined3d_bo_vk((*buffer).buffer_object));
            buffers[count as usize] = (*buffer_info).buffer;
            offsets[count as usize] = (*buffer_info)
                .offset
                .wrapping_add((*stream).offset as VkDeviceSize);
            count = count.wrapping_add(1);
            i = i.wrapping_add(1);
            continue;
        }

        if count != 0 {
            ((*vk_info).vk_ops.vkCmdBindVertexBuffers.unwrap_unchecked())(
                vk_command_buffer,
                first as uint32_t,
                count as uint32_t,
                addr_of_mut!(buffers) as *mut VkBuffer,
                addr_of_mut!(offsets) as *mut VkDeviceSize,
            );
        }
        first = i.wrapping_add(1);
        count = 0;
        i = i.wrapping_add(1);
    }

    if count != 0 {
        ((*vk_info).vk_ops.vkCmdBindVertexBuffers.unwrap_unchecked())(
            vk_command_buffer,
            first as uint32_t,
            count as uint32_t,
            addr_of_mut!(buffers) as *mut VkBuffer,
            addr_of_mut!(offsets) as *mut VkDeviceSize,
        );
    }
}

unsafe extern "C" fn wined3d_context_vk_bind_stream_output_buffers(
    context_vk: *mut wined3d_context_vk,
    vk_command_buffer: VkCommandBuffer,
    state: *const wined3d_state,
    vk_info: *const wined3d_vk_info,
) {
    let mut offsets: [VkDeviceSize; 4] = [0; 4];
    let mut sizes: [VkDeviceSize; 4] = [0; 4];
    let mut buffers: [VkBuffer; 4] = [0; 4];
    let mut stream: *const wined3d_stream_output = core::ptr::null();
    let mut buffer_info: *const VkDescriptorBufferInfo = core::ptr::null();
    let mut buffer_vk: *mut wined3d_buffer_vk = core::ptr::null_mut();
    let mut buffer: *mut wined3d_buffer = core::ptr::null_mut();
    let mut i: core::ffi::c_uint;
    let mut first: core::ffi::c_uint;
    let mut count: core::ffi::c_uint;

    if (*context_vk).vk_so_counter_bo.vk_buffer == 0 {
        let bo: *mut wined3d_bo_vk = core::ptr::addr_of_mut!((*context_vk).vk_so_counter_bo);

        if wined3d_context_vk_create_bo(
            context_vk,
            (ARRAY_SIZE!((*context_vk).vk_so_counters) as VkDeviceSize)
                .wrapping_mul(core::mem::size_of::<uint32_t>() as VkDeviceSize)
                .wrapping_mul(2 as VkDeviceSize),
            VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_COUNTER_BUFFER_BIT_EXT as VkBufferUsageFlags,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as VkMemoryPropertyFlags,
            bo,
        ) == 0
        {
            ERR!("Failed to create counter BO.\n");
        }

        i = 0 as core::ffi::c_uint;
        while (i as usize) < ARRAY_SIZE!((*context_vk).vk_so_counters) as usize {
            (*context_vk).vk_so_offsets[i as usize] = ((*bo).b.buffer_offset as usize).wrapping_add(
                (i as usize)
                    .wrapping_mul(core::mem::size_of::<uint32_t>())
                    .wrapping_mul(2 as usize),
            ) as VkDeviceSize;
            i = i.wrapping_add(1);
        }
    }

    memset(
        core::ptr::addr_of_mut!((*context_vk).vk_so_counters) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<[VkBuffer; 4]>() as size_t,
    );
    first = 0 as core::ffi::c_uint;
    count = 0 as core::ffi::c_uint;
    i = 0 as core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!((*state).stream_output) as usize {
        stream = (core::ptr::addr_of!((*state).stream_output) as *const wined3d_stream_output)
            .offset(i as isize);

        buffer = (*stream).buffer;
        if !buffer.is_null() {
            (*context_vk).vk_so_counters[i as usize] = (*context_vk).vk_so_counter_bo.vk_buffer;

            buffer_vk = wined3d_buffer_vk(buffer);
            buffer_info = wined3d_buffer_vk_get_buffer_info(buffer_vk);
            wined3d_context_vk_reference_bo(
                context_vk,
                wined3d_bo_vk((*buffer).buffer_object),
            );
            buffers[count as usize] = (*buffer_info).buffer;
            offsets[count as usize] = (*stream).offset as VkDeviceSize;
            if offsets[count as usize] == !(0 as core::ffi::c_uint) as VkDeviceSize {
                // FIXME: Appending to stream output buffers not implemented.
                offsets[count as usize] = 0 as VkDeviceSize;
            }
            sizes[count as usize] = (*buffer_info).range.wrapping_sub(offsets[count as usize]);
            offsets[count as usize] = offsets[count as usize].wrapping_add((*buffer_info).offset);
            count = count.wrapping_add(1);
        } else {
            if count != 0 {
                ((*vk_info).vk_ops.vkCmdBindTransformFeedbackBuffersEXT.unwrap_unchecked())(
                    vk_command_buffer,
                    first as uint32_t,
                    count as uint32_t,
                    core::ptr::addr_of_mut!(buffers) as *mut VkBuffer,
                    core::ptr::addr_of_mut!(offsets) as *mut VkDeviceSize,
                    core::ptr::addr_of_mut!(sizes) as *mut VkDeviceSize,
                );
            }
            first = i.wrapping_add(1 as core::ffi::c_uint);
            count = 0 as core::ffi::c_uint;
        }
        i = i.wrapping_add(1);
    }

    if count != 0 {
        ((*vk_info).vk_ops.vkCmdBindTransformFeedbackBuffersEXT.unwrap_unchecked())(
            vk_command_buffer,
            first as uint32_t,
            count as uint32_t,
            core::ptr::addr_of_mut!(buffers) as *mut VkBuffer,
            core::ptr::addr_of_mut!(offsets) as *mut VkDeviceSize,
            core::ptr::addr_of_mut!(sizes) as *mut VkDeviceSize,
        );
    }
}

unsafe extern "C" fn wined3d_context_vk_create_vk_descriptor_pool(device_vk: *mut wined3d_device_vk, vk_info: *const wined3d_vk_info, vk_pool: *mut VkDescriptorPool) -> VkResult {
    let mut pool_desc: VkDescriptorPoolCreateInfo = core::mem::zeroed();
    let mut vr: VkResult;

    static mut pool_sizes: [VkDescriptorPoolSize; 6] = [
        VkDescriptorPoolSize { type_: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER, descriptorCount: 1024 },
        VkDescriptorPoolSize { type_: VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER, descriptorCount: 1024 },
        VkDescriptorPoolSize { type_: VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE, descriptorCount: 1024 },
        VkDescriptorPoolSize { type_: VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER, descriptorCount: 1024 },
        VkDescriptorPoolSize { type_: VK_DESCRIPTOR_TYPE_STORAGE_IMAGE, descriptorCount: 1024 },
        VkDescriptorPoolSize { type_: VK_DESCRIPTOR_TYPE_SAMPLER, descriptorCount: 1024 },
    ];

    pool_desc.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
    pool_desc.pNext = core::ptr::null();
    pool_desc.flags = 0;
    pool_desc.maxSets = 512;
    pool_desc.poolSizeCount = ARRAY_SIZE!(pool_sizes);
    pool_desc.pPoolSizes = &raw const pool_sizes as *const VkDescriptorPoolSize;

    vr = ((*vk_info).vk_ops.vkCreateDescriptorPool.unwrap_unchecked())((*device_vk).vk_device, &raw mut pool_desc, core::ptr::null_mut(), vk_pool);
    if vr < 0 {
        ERR!("Failed to create descriptor pool, vr %s.\n", wined3d_debug_vkresult(vr));
    }

    return vr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_get_vk_descriptor_pool(
    context_vk: *mut wined3d_context_vk,
) -> VkDescriptorPool {
    let mut vr: VkResult;

    if (*context_vk).vk_descriptor_pool_count != 0 {
        return *(*context_vk).vk_descriptor_pools.add(0);
    }

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*context_vk).vk_descriptor_pools) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*context_vk).vk_descriptor_pools_size),
        1 as SIZE_T,
        core::mem::size_of::<VkDescriptorPool>() as SIZE_T,
    ) == 0
    {
        ERR!("Failed to allocate array.\n");
        return (VK_NULL_HANDLE) as VkDescriptorPool;
    }

    vr = wined3d_context_vk_create_vk_descriptor_pool(
        wined3d_device_vk((*context_vk).c.device),
        (*context_vk).vk_info,
        (*context_vk).vk_descriptor_pools,
    );
    if vr as u64 != 0 {
        WARN!(
            "Failed to create descriptor pool, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return (VK_NULL_HANDLE) as VkDescriptorPool;
    }

    (*context_vk).vk_descriptor_pool_count = 1;
    return *(*context_vk).vk_descriptor_pools.add(0);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_create_vk_descriptor_set(context_vk: *mut wined3d_context_vk, mut vk_set_layout: VkDescriptorSetLayout) -> VkDescriptorSet {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut set_desc: VkDescriptorSetAllocateInfo = core::mem::zeroed();
    let mut vk_descriptor_pool: VkDescriptorPool;
    let mut vk_descriptor_set: VkDescriptorSet = 0;
    let mut vr: VkResult;

    vk_descriptor_pool = wined3d_context_vk_get_vk_descriptor_pool(context_vk);
    if vk_descriptor_pool == 0 {
        return (VK_NULL_HANDLE) as VkDescriptorSet;
    }

    set_desc.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
    set_desc.pNext = core::ptr::null_mut();
    set_desc.descriptorPool = vk_descriptor_pool;
    set_desc.descriptorSetCount = 1;
    set_desc.pSetLayouts = core::ptr::addr_of_mut!(vk_set_layout);
    vr = ((*vk_info).vk_ops.vkAllocateDescriptorSets.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(set_desc),
        core::ptr::addr_of_mut!(vk_descriptor_set),
    );
    if vr >= 0 {
        return vk_descriptor_set;
    }

    if vr == VK_ERROR_FRAGMENTED_POOL || vr == VK_ERROR_OUT_OF_POOL_MEMORY {
        wined3d_context_vk_reset_vk_descriptor_pool(
            context_vk,
            vk_descriptor_pool,
            (*context_vk).current_command_buffer.id,
        );
        {
            (*context_vk).vk_descriptor_pool_count -= 1;
        }
        let idx: usize = (*context_vk).vk_descriptor_pool_count as usize;
        *(*context_vk).vk_descriptor_pools.add(0 as usize) =
            *(*context_vk).vk_descriptor_pools.add(idx);

        vk_descriptor_pool = wined3d_context_vk_get_vk_descriptor_pool(context_vk);
        if vk_descriptor_pool == 0 {
            WARN!("Failed to create descriptor pool.\n");
            return (VK_NULL_HANDLE) as VkDescriptorSet;
        }

        set_desc.descriptorPool = vk_descriptor_pool;
        vr = ((*vk_info).vk_ops.vkAllocateDescriptorSets.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(set_desc),
            core::ptr::addr_of_mut!(vk_descriptor_set),
        );
        if vr >= 0 {
            return vk_descriptor_set;
        }
    }

    WARN!("Failed to allocate descriptor set, vr %s.\n", wined3d_debug_vkresult(vr));

    return (VK_NULL_HANDLE) as VkDescriptorSet;
}

unsafe extern "C" fn wined3d_shader_descriptor_writes_vk_add_write(
    writes: *mut wined3d_shader_descriptor_writes_vk,
    vk_descriptor_set: VkDescriptorSet,
    binding_idx: size_t,
    type_: VkDescriptorType,
    buffer_info: *const VkDescriptorBufferInfo,
    image_info: *const VkDescriptorImageInfo,
    buffer_view: *const VkBufferView,
) -> bool {
    let write_count: SIZE_T = (*writes).count;
    let mut write: *mut VkWriteDescriptorSet = core::ptr::null_mut();

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*writes).writes) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*writes).size),
        write_count + 1,
        core::mem::size_of::<VkWriteDescriptorSet>() as SIZE_T,
    ) == 0
    {
        return false;
    }

    write = (*writes).writes.add(write_count as usize) as *mut VkWriteDescriptorSet;
    (*write).sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
    (*write).pNext = core::ptr::null();
    (*write).dstSet = vk_descriptor_set;
    (*write).dstBinding = binding_idx as uint32_t;
    (*write).dstArrayElement = 0;
    (*write).descriptorCount = 1;
    (*write).descriptorType = type_;
    (*write).pImageInfo = image_info;
    (*write).pBufferInfo = buffer_info;
    (*write).pTexelBufferView = buffer_view;

    (*writes).count = (*writes).count.wrapping_add(1);

    return true;
}

unsafe extern "C" fn wined3d_shader_resource_bindings_add_null_srv_binding(
    writes: *mut wined3d_shader_descriptor_writes_vk,
    vk_descriptor_set: VkDescriptorSet,
    binding_idx: size_t,
    type_0: wined3d_shader_resource_type,
    data_type: wined3d_data_type,
    context_vk: *mut wined3d_context_vk,
) -> bool {
    let v: *const wined3d_null_views_vk =
        core::ptr::addr_of!((*wined3d_device_vk((*context_vk).c.device)).null_views_vk);

    match type_0 {
        WINED3D_SHADER_RESOURCE_BUFFER => {
            if data_type as core::ffi::c_uint == WINED3D_DATA_FLOAT as core::ffi::c_uint {
                return wined3d_shader_descriptor_writes_vk_add_write(
                    writes,
                    vk_descriptor_set,
                    binding_idx,
                    VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER,
                    core::ptr::null::<VkDescriptorBufferInfo>(),
                    core::ptr::null::<VkDescriptorImageInfo>(),
                    core::ptr::addr_of!((*v).vk_view_buffer_float),
                );
            }
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::null::<VkDescriptorImageInfo>(),
                core::ptr::addr_of!((*v).vk_view_buffer_uint),
            );
        }

        WINED3D_SHADER_RESOURCE_TEXTURE_1D => {
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::addr_of!((*v).vk_info_1d),
                core::ptr::null::<VkBufferView>(),
            );
        }

        WINED3D_SHADER_RESOURCE_TEXTURE_2D => {
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::addr_of!((*v).vk_info_2d),
                core::ptr::null::<VkBufferView>(),
            );
        }

        WINED3D_SHADER_RESOURCE_TEXTURE_2DMS => {
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::addr_of!((*v).vk_info_2dms),
                core::ptr::null::<VkBufferView>(),
            );
        }

        WINED3D_SHADER_RESOURCE_TEXTURE_3D => {
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::addr_of!((*v).vk_info_3d),
                core::ptr::null::<VkBufferView>(),
            );
        }

        WINED3D_SHADER_RESOURCE_TEXTURE_CUBE => {
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::addr_of!((*v).vk_info_cube),
                core::ptr::null::<VkBufferView>(),
            );
        }

        WINED3D_SHADER_RESOURCE_TEXTURE_1DARRAY => {
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::addr_of!((*v).vk_info_1d_array),
                core::ptr::null::<VkBufferView>(),
            );
        }

        WINED3D_SHADER_RESOURCE_TEXTURE_2DARRAY => {
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::addr_of!((*v).vk_info_2d_array),
                core::ptr::null::<VkBufferView>(),
            );
        }

        WINED3D_SHADER_RESOURCE_TEXTURE_2DMSARRAY => {
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::addr_of!((*v).vk_info_2dms_array),
                core::ptr::null::<VkBufferView>(),
            );
        }

        WINED3D_SHADER_RESOURCE_TEXTURE_CUBEARRAY => {
            return wined3d_shader_descriptor_writes_vk_add_write(
                writes,
                vk_descriptor_set,
                binding_idx,
                VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                core::ptr::null::<VkDescriptorBufferInfo>(),
                core::ptr::addr_of!((*v).vk_info_cube_array),
                core::ptr::null::<VkBufferView>(),
            );
        }

        _ => {
            FIXME!("Unhandled resource type %#x.\n", type_0);
            return false;
        }
    }
}

unsafe extern "C" fn wined3d_shader_descriptor_writes_vk_add_cbv_write(
    mut writes: *mut wined3d_shader_descriptor_writes_vk,
    mut context_vk: *mut wined3d_context_vk,
    mut vk_descriptor_set: VkDescriptorSet,
    mut state: *const wined3d_state,
    mut binding: *const wined3d_shader_resource_binding,
    mut buffer_info: *mut VkDescriptorBufferInfo,
) -> bool {
    let mut cb_state: *const wined3d_constant_buffer_state = (&raw const *(&raw const (*state).cb
        as *const [wined3d_constant_buffer_state; 15])
        .offset((*binding).shader_type as isize)
        as *const wined3d_constant_buffer_state)
        .offset((*binding).resource_idx as isize)
        as *const wined3d_constant_buffer_state;
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let mut buffer_vk: *mut wined3d_buffer_vk = ::core::ptr::null_mut::<wined3d_buffer_vk>();
    let mut buffer: *mut wined3d_buffer = ::core::ptr::null_mut::<wined3d_buffer>();
    buffer = (*cb_state).buffer;
    if buffer.is_null() {
        return wined3d_shader_descriptor_writes_vk_add_write(
            writes,
            vk_descriptor_set,
            (*binding).binding_idx,
            VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            &raw mut (*device_vk).null_resources_vk.buffer_info,
            ::core::ptr::null::<VkDescriptorImageInfo>(),
            ::core::ptr::null::<VkBufferView>(),
        );
    }
    buffer_vk = wined3d_buffer_vk(buffer);
    *buffer_info = *wined3d_buffer_vk_get_buffer_info(buffer_vk);
    (*buffer_info).offset = (*buffer_info)
        .offset
        .wrapping_add((*cb_state).offset as VkDeviceSize);
    (*buffer_info).range = min!((*cb_state).size as VkDeviceSize, (*buffer_info).range);
    if !wined3d_shader_descriptor_writes_vk_add_write(
        writes,
        vk_descriptor_set,
        (*binding).binding_idx,
        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
        buffer_info,
        ::core::ptr::null::<VkDescriptorImageInfo>(),
        ::core::ptr::null::<VkBufferView>(),
    ) {
        return false;
    }
    wined3d_context_vk_reference_bo(context_vk, wined3d_bo_vk((*buffer).buffer_object));
    return true;
}

unsafe extern "C" fn wined3d_shader_descriptor_writes_vk_add_srv_write(writes: *mut wined3d_shader_descriptor_writes_vk, context_vk: *mut wined3d_context_vk, vk_descriptor_set: VkDescriptorSet, state: *const wined3d_state, binding: *const wined3d_shader_resource_binding) -> bool {
    let mut srv_vk: *mut wined3d_shader_resource_view_vk = core::ptr::null_mut::<wined3d_shader_resource_view_vk>();
    let mut srv: *mut wined3d_shader_resource_view = core::ptr::null_mut::<wined3d_shader_resource_view>();
    let mut image_info: *const VkDescriptorImageInfo = core::ptr::null::<VkDescriptorImageInfo>();
    let mut resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut view_vk: *mut wined3d_view_vk = core::ptr::null_mut::<wined3d_view_vk>();
    let mut buffer_view: *mut VkBufferView = core::ptr::null_mut::<VkBufferView>();
    let mut type_0: VkDescriptorType = 0;

    srv = (*state).shader_resource_view[(*binding).shader_type as usize][(*binding).resource_idx as usize];
    if srv.is_null() {
        return wined3d_shader_resource_bindings_add_null_srv_binding(
            writes,
            vk_descriptor_set,
            (*binding).binding_idx,
            (*binding).resource_type,
            (*binding).resource_data_type,
            context_vk,
        );
    }

    resource = (*srv).resource;
    srv_vk = wined3d_shader_resource_view_vk(srv);
    view_vk = core::ptr::addr_of_mut!((*srv_vk).view_vk);
    if (*resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint {
        image_info = core::ptr::null::<VkDescriptorImageInfo>();
        buffer_view = core::ptr::addr_of_mut!((*view_vk).u.vk_buffer_view);
        type_0 = VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER;
    } else {
        let mut texture_vk: *mut wined3d_texture_vk = wined3d_texture_vk(texture_from_resource(resource));

        if (*view_vk).u.vk_image_info.imageView != 0 {
            image_info = core::ptr::addr_of_mut!((*view_vk).u.vk_image_info);
            if (*image_info).imageLayout as ::core::ffi::c_uint != (*texture_vk).layout as ::core::ffi::c_uint {
                wined3d_shader_resource_view_vk_update_layout(srv_vk, (*texture_vk).layout as VkImageLayout);
            }
        } else {
            image_info = wined3d_texture_vk_get_default_image_info(texture_vk, context_vk);
        }
        buffer_view = core::ptr::null_mut::<VkBufferView>();
        type_0 = VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE;
    }

    if !wined3d_shader_descriptor_writes_vk_add_write(
        writes,
        vk_descriptor_set,
        (*binding).binding_idx,
        type_0,
        core::ptr::null::<VkDescriptorBufferInfo>(),
        image_info,
        buffer_view,
    ) {
        return false;
    }
    wined3d_context_vk_reference_shader_resource_view(context_vk, srv_vk);
    return true;
}

unsafe extern "C" fn wined3d_shader_descriptor_writes_vk_add_uav_write(
    mut writes: *mut wined3d_shader_descriptor_writes_vk,
    mut context_vk: *mut wined3d_context_vk,
    mut pipeline: wined3d_pipeline,
    mut vk_descriptor_set: VkDescriptorSet,
    mut state: *const wined3d_state,
    mut binding: *const wined3d_shader_resource_binding,
) -> bool {
    let mut uav_vk: *mut wined3d_unordered_access_view_vk =
        ::core::ptr::null_mut::<wined3d_unordered_access_view_vk>();
    let mut uav: *mut wined3d_unordered_access_view =
        ::core::ptr::null_mut::<wined3d_unordered_access_view>();
    let mut image_info: *const VkDescriptorImageInfo = ::core::ptr::null::<VkDescriptorImageInfo>();
    let mut resource: *mut wined3d_resource = ::core::ptr::null_mut::<wined3d_resource>();
    let mut view_vk: *mut wined3d_view_vk = ::core::ptr::null_mut::<wined3d_view_vk>();
    let mut buffer_view: *mut VkBufferView = ::core::ptr::null_mut::<VkBufferView>();
    let mut type_0: VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER;
    uav = (*state).unordered_access_view[pipeline as usize][(*binding).resource_idx as usize];
    if uav.is_null() {
        WINE_FIXME!("NULL unordered access views not implemented.\n");
        return false;
    }
    resource = (*uav).resource;
    uav_vk = wined3d_unordered_access_view_vk(uav);
    view_vk = &raw mut (*uav_vk).view_vk;
    if (*resource).type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
    {
        image_info = ::core::ptr::null::<VkDescriptorImageInfo>();
        buffer_view = &raw mut (*view_vk).u.vk_buffer_view;
        type_0 = VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER;
    } else {
        let mut texture_vk: *mut wined3d_texture_vk =
            wined3d_texture_vk(texture_from_resource(resource));
        if (*view_vk).u.vk_image_info.imageView != 0 {
            image_info = &raw mut (*view_vk).u.vk_image_info;
        } else {
            image_info = wined3d_texture_vk_get_default_image_info(texture_vk, context_vk);
        }
        buffer_view = ::core::ptr::null_mut::<VkBufferView>();
        type_0 = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
    }
    if !wined3d_shader_descriptor_writes_vk_add_write(
        writes,
        vk_descriptor_set,
        (*binding).binding_idx,
        type_0,
        ::core::ptr::null::<VkDescriptorBufferInfo>(),
        image_info,
        buffer_view,
    ) {
        return false;
    }
    wined3d_context_vk_reference_unordered_access_view(context_vk, uav_vk);
    return true;
}

unsafe extern "C" fn wined3d_shader_descriptor_writes_vk_add_uav_counter_write(
    mut writes: *mut wined3d_shader_descriptor_writes_vk,
    mut context_vk: *mut wined3d_context_vk,
    mut pipeline: wined3d_pipeline,
    mut vk_descriptor_set: VkDescriptorSet,
    mut state: *const wined3d_state,
    mut binding: *const wined3d_shader_resource_binding,
) -> bool {
    let mut uav_vk: *mut wined3d_unordered_access_view_vk =
        ::core::ptr::null_mut::<wined3d_unordered_access_view_vk>();
    let mut uav: *mut wined3d_unordered_access_view =
        ::core::ptr::null_mut::<wined3d_unordered_access_view>();
    uav = (*state).unordered_access_view[pipeline as usize][(*binding).resource_idx as usize];
    if uav.is_null() {
        WINE_FIXME!("NULL unordered access view counters not implemented.\n");
        return false;
    }
    uav_vk = wined3d_unordered_access_view_vk(uav);
    if (*uav_vk).vk_counter_view == 0 {
        return false;
    }
    return wined3d_shader_descriptor_writes_vk_add_write(
        writes,
        vk_descriptor_set,
        (*binding).binding_idx,
        VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER,
        ::core::ptr::null::<VkDescriptorBufferInfo>(),
        ::core::ptr::null::<VkDescriptorImageInfo>(),
        core::ptr::addr_of_mut!((*uav_vk).vk_counter_view),
    );
}

unsafe extern "C" fn wined3d_shader_descriptor_writes_vk_add_sampler_write(writes: *mut wined3d_shader_descriptor_writes_vk, context_vk: *mut wined3d_context_vk, vk_descriptor_set: VkDescriptorSet, state: *const wined3d_state, binding: *const wined3d_shader_resource_binding) -> bool {
    let mut sampler: *mut wined3d_sampler = core::ptr::null_mut();

    sampler = (*state).sampler[(*binding).shader_type as usize][(*binding).resource_idx as usize];
    if sampler.is_null() {
        sampler = (*(*context_vk).c.device).null_sampler;
    }
    if !wined3d_shader_descriptor_writes_vk_add_write(writes, vk_descriptor_set, (*binding).binding_idx,
            VK_DESCRIPTOR_TYPE_SAMPLER, core::ptr::null(), core::ptr::addr_of_mut!((*wined3d_sampler_vk(sampler)).vk_image_info), core::ptr::null()) {
        return false;
    }
    wined3d_context_vk_reference_sampler(context_vk, wined3d_sampler_vk(sampler));
    return true;
}

unsafe extern "C" fn wined3d_context_vk_update_descriptors(
    context_vk: *mut wined3d_context_vk,
    vk_command_buffer: VkCommandBuffer,
    state: *const wined3d_state,
    pipeline: wined3d_pipeline,
) -> bool {
    let writes: *mut wined3d_shader_descriptor_writes_vk = core::ptr::addr_of_mut!((*context_vk).descriptor_writes);
    let mut buffers: [[VkDescriptorBufferInfo; (MAX_CONSTANT_BUFFERS as _)]; (WINED3D_SHADER_TYPE_COUNT) as _] =
        [[VkDescriptorBufferInfo { buffer: 0, offset: 0, range: 0 }; (MAX_CONSTANT_BUFFERS as _)]; (WINED3D_SHADER_TYPE_COUNT) as _];
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut binding: *const wined3d_shader_resource_binding = core::ptr::null();
    let mut bindings: *mut wined3d_shader_resource_bindings = core::ptr::null_mut();
    let mut vk_set_layout: VkDescriptorSetLayout = 0;
    let mut vk_pipeline_layout: VkPipelineLayout = 0;
    let mut vk_bind_point: VkPipelineBindPoint = 0;
    let mut vk_descriptor_set: VkDescriptorSet = 0;
    let mut i: size_t = 0;

    match pipeline as ::core::ffi::c_uint {
        WINED3D_PIPELINE_GRAPHICS => {
            bindings = core::ptr::addr_of_mut!((*context_vk).graphics.bindings);
            vk_bind_point = VK_PIPELINE_BIND_POINT_GRAPHICS;
            vk_set_layout = (*context_vk).graphics.vk_set_layout;
            vk_pipeline_layout = (*context_vk).graphics.vk_pipeline_layout;
        }
        WINED3D_PIPELINE_COMPUTE => {
            bindings = core::ptr::addr_of_mut!((*context_vk).compute.bindings);
            vk_bind_point = VK_PIPELINE_BIND_POINT_COMPUTE;
            vk_set_layout = (*context_vk).compute.vk_set_layout;
            vk_pipeline_layout = (*context_vk).compute.vk_pipeline_layout;
        }
        _ => {
            ERR!("Invalid pipeline %#x.\n", pipeline);
            return false;
        }
    }

    vk_descriptor_set = wined3d_context_vk_create_vk_descriptor_set(context_vk, vk_set_layout);
    if vk_descriptor_set == 0 {
        WARN!("Failed to create descriptor set.\n");
        return false;
    }

    (*writes).count = 0 as SIZE_T;
    i = 0 as size_t;
    while i < ((*bindings).count) as _ {
        binding = (*bindings).bindings.offset(i as isize);

        match (*binding).shader_descriptor_type as ::core::ffi::c_uint {
            WINED3D_SHADER_DESCRIPTOR_TYPE_CBV => {
                if !wined3d_shader_descriptor_writes_vk_add_cbv_write(
                    writes,
                    context_vk,
                    vk_descriptor_set,
                    state,
                    binding,
                    core::ptr::addr_of_mut!(buffers[(*binding).shader_type as usize][(*binding).resource_idx as usize]),
                ) {
                    return false;
                }
            }
            WINED3D_SHADER_DESCRIPTOR_TYPE_SRV => {
                if !wined3d_shader_descriptor_writes_vk_add_srv_write(
                    writes,
                    context_vk,
                    vk_descriptor_set,
                    state,
                    binding,
                ) {
                    return false;
                }
            }
            WINED3D_SHADER_DESCRIPTOR_TYPE_UAV => {
                if !wined3d_shader_descriptor_writes_vk_add_uav_write(
                    writes,
                    context_vk,
                    pipeline,
                    vk_descriptor_set,
                    state,
                    binding,
                ) {
                    return false;
                }
            }
            WINED3D_SHADER_DESCRIPTOR_TYPE_UAV_COUNTER => {
                if !wined3d_shader_descriptor_writes_vk_add_uav_counter_write(
                    writes,
                    context_vk,
                    pipeline,
                    vk_descriptor_set,
                    state,
                    binding,
                ) {
                    return false;
                }
            }
            WINED3D_SHADER_DESCRIPTOR_TYPE_SAMPLER => {
                if !wined3d_shader_descriptor_writes_vk_add_sampler_write(
                    writes,
                    context_vk,
                    vk_descriptor_set,
                    state,
                    binding,
                ) {
                    return false;
                }
            }
            _ => {
                ERR!("Invalid descriptor type %#x.\n", (*binding).shader_descriptor_type as ::core::ffi::c_uint);
                return false;
            }
        }
        i = i.wrapping_add(1);
    }

    ((*vk_info).vk_ops.vkUpdateDescriptorSets.unwrap_unchecked())(
        (*device_vk).vk_device,
        (*writes).count as uint32_t,
        (*writes).writes,
        0 as uint32_t,
        core::ptr::null::<VkCopyDescriptorSet>(),
    );
    ((*vk_info).vk_ops.vkCmdBindDescriptorSets.unwrap_unchecked())(
        vk_command_buffer,
        vk_bind_point,
        vk_pipeline_layout,
        0 as uint32_t,
        1 as uint32_t,
        core::ptr::addr_of_mut!(vk_descriptor_set),
        0 as uint32_t,
        core::ptr::null::<uint32_t>(),
    );

    return true;
}

unsafe extern "C" fn wined3d_context_vk_create_vk_descriptor_set_layout(
    device_vk: *mut wined3d_device_vk,
    vk_info: *const wined3d_vk_info,
    key: *const wined3d_pipeline_layout_key_vk,
    vk_set_layout: *mut VkDescriptorSetLayout,
) -> VkResult {
    let mut layout_desc: VkDescriptorSetLayoutCreateInfo = core::mem::zeroed();
    let mut vr: VkResult = core::mem::zeroed();

    layout_desc.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
    layout_desc.pNext = core::ptr::null();
    layout_desc.flags = 0;
    layout_desc.bindingCount = ((*key).binding_count) as u32;
    layout_desc.pBindings = (*key).bindings;

    vr = ((*vk_info).vk_ops.vkCreateDescriptorSetLayout.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(layout_desc),
        core::ptr::null(),
        vk_set_layout,
    );
    if (vr as i32) < 0 {
        WARN!(
            "Failed to create Vulkan descriptor set layout, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
    }

    return vr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_get_pipeline_layout(context_vk: *mut wined3d_context_vk, bindings: *mut VkDescriptorSetLayoutBinding, binding_count: SIZE_T) -> *mut wined3d_pipeline_layout_vk {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut key: wined3d_pipeline_layout_key_vk = core::mem::zeroed();
    let mut layout: *mut wined3d_pipeline_layout_vk = core::ptr::null_mut();
    let mut layout_desc: VkPipelineLayoutCreateInfo = core::mem::zeroed();
    let mut entry: *mut wine_rb_entry = core::ptr::null_mut();
    let mut vr: VkResult = 0;

    key.bindings = bindings;
    key.binding_count = binding_count;

    entry = wine_rb_get(
        core::ptr::addr_of_mut!((*context_vk).pipeline_layouts),
        core::ptr::addr_of_mut!(key) as *const core::ffi::c_void,
    );
    if !entry.is_null() {
        return WINE_RB_ENTRY_VALUE!(entry, wined3d_pipeline_layout_vk, entry);
    }

    layout = malloc((core::mem::size_of::<wined3d_pipeline_layout_vk>() as SIZE_T) as _)
        as *mut wined3d_pipeline_layout_vk;
    if layout.is_null() {
        return core::ptr::null_mut();
    }

    (*layout).key.bindings = malloc(
        ((core::mem::size_of::<VkDescriptorSetLayoutBinding>() as SIZE_T) * key.binding_count) as _,
    ) as *mut VkDescriptorSetLayoutBinding;
    if (*layout).key.bindings.is_null() {
        free(layout as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }
    memcpy(
        (*layout).key.bindings as *mut core::ffi::c_void,
        key.bindings as *const core::ffi::c_void,
        ((core::mem::size_of::<VkDescriptorSetLayoutBinding>() as SIZE_T) * key.binding_count) as _,
    );
    (*layout).key.binding_count = key.binding_count;

    'fail: {
        vr = wined3d_context_vk_create_vk_descriptor_set_layout(
            device_vk,
            vk_info,
            core::ptr::addr_of_mut!(key),
            core::ptr::addr_of_mut!((*layout).vk_set_layout),
        );
        if vr != 0 {
            WARN!("Failed to create descriptor set layout, vr %s.\n", wined3d_debug_vkresult(vr));
            break 'fail;
        }

        layout_desc.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
        layout_desc.pNext = core::ptr::null();
        layout_desc.flags = 0;
        layout_desc.setLayoutCount = 1;
        layout_desc.pSetLayouts = core::ptr::addr_of_mut!((*layout).vk_set_layout);
        layout_desc.pushConstantRangeCount = 0;
        layout_desc.pPushConstantRanges = core::ptr::null();

        vr = ((*vk_info).vk_ops.vkCreatePipelineLayout.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(layout_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!((*layout).vk_pipeline_layout),
        );
        if vr < 0 {
            WARN!("Failed to create Vulkan pipeline layout, vr %s.\n", wined3d_debug_vkresult(vr));
            ((*vk_info).vk_ops.vkDestroyDescriptorSetLayout.unwrap_unchecked())(
                (*device_vk).vk_device,
                (*layout).vk_set_layout,
                core::ptr::null(),
            );
            break 'fail;
        }

        if wine_rb_put(
            core::ptr::addr_of_mut!((*context_vk).pipeline_layouts),
            core::ptr::addr_of_mut!((*layout).key) as *const core::ffi::c_void,
            core::ptr::addr_of_mut!((*layout).entry),
        ) == -1
        {
            ERR!("Failed to insert pipeline layout.\n");
            ((*vk_info).vk_ops.vkDestroyPipelineLayout.unwrap_unchecked())(
                (*device_vk).vk_device,
                (*layout).vk_pipeline_layout,
                core::ptr::null(),
            );
            ((*vk_info).vk_ops.vkDestroyDescriptorSetLayout.unwrap_unchecked())(
                (*device_vk).vk_device,
                (*layout).vk_set_layout,
                core::ptr::null(),
            );
            break 'fail;
        }

        return layout;
    }

    free((*layout).key.bindings as *mut core::ffi::c_void);
    free(layout as *mut core::ffi::c_void);
    return core::ptr::null_mut();
}

unsafe extern "C" fn wined3d_context_vk_get_graphics_pipeline(
    context_vk: *mut wined3d_context_vk,
) -> VkPipeline {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut pipeline_vk: *mut wined3d_graphics_pipeline_vk;
    let mut key: *mut wined3d_graphics_pipeline_key_vk;
    let mut entry: *mut wine_rb_entry;
    let mut vr: VkResult;

    key = core::ptr::addr_of_mut!((*context_vk).graphics.pipeline_key_vk);
    entry = wine_rb_get(
        core::ptr::addr_of_mut!((*context_vk).graphics_pipelines),
        key as *const core::ffi::c_void,
    );
    if !entry.is_null() {
        return (*WINE_RB_ENTRY_VALUE!(entry, wined3d_graphics_pipeline_vk, entry)).vk_pipeline;
    }

    pipeline_vk = malloc(core::mem::size_of::<wined3d_graphics_pipeline_vk>())
        as *mut wined3d_graphics_pipeline_vk;
    if pipeline_vk.is_null() {
        return (VK_NULL_HANDLE) as VkPipeline;
    }
    (*pipeline_vk).key = *key;

    vr = ((*vk_info).vk_ops.vkCreateGraphicsPipelines.unwrap_unchecked())(
        (*device_vk).vk_device,
        (VK_NULL_HANDLE as _),
        1,
        core::ptr::addr_of_mut!((*key).pipeline_desc),
        core::ptr::null(),
        core::ptr::addr_of_mut!((*pipeline_vk).vk_pipeline),
    );
    if (vr as i32) < 0 {
        WARN!(
            "Failed to create graphics pipeline, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        free(pipeline_vk as *mut core::ffi::c_void);
        return (VK_NULL_HANDLE) as VkPipeline;
    }

    if wine_rb_put(
        core::ptr::addr_of_mut!((*context_vk).graphics_pipelines),
        core::ptr::addr_of!((*pipeline_vk).key) as *const core::ffi::c_void,
        core::ptr::addr_of_mut!((*pipeline_vk).entry),
    ) == -1
    {
        ERR!("Failed to insert pipeline.\n");
    }

    return (*pipeline_vk).vk_pipeline;
}

unsafe extern "C" fn wined3d_context_vk_load_buffers(
    mut context_vk: *mut wined3d_context_vk,
    mut state: *const wined3d_state,
    mut indirect_vk: *mut wined3d_buffer_vk,
    mut indexed: bool,
) {
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut buffer_vk: *mut wined3d_buffer_vk = core::ptr::null_mut();
    let mut buffer: *mut wined3d_buffer = core::ptr::null_mut();
    let mut i: u32;

    i = 0 as u32;
    while (i as usize) < (ARRAY_SIZE!((*state).streams) as usize) {
        buffer = (*state).streams[i as usize].buffer as *mut wined3d_buffer;
        if buffer.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        buffer_vk = wined3d_buffer_vk(buffer);
        wined3d_buffer_load(
            core::ptr::addr_of_mut!((*buffer_vk).b),
            core::ptr::addr_of_mut!((*context_vk).c),
            state,
        );
        wined3d_buffer_vk_barrier(buffer_vk, context_vk, WINED3D_BIND_VERTEX_BUFFER);
        if !(*buffer_vk).b.bo_user.valid {
            context_invalidate_state(
                core::ptr::addr_of_mut!((*context_vk).c),
                (STATE_STREAMSRC as u32),
            );
        }
        i = i.wrapping_add(1);
    }

    if use_transform_feedback(state) != 0
        && (*vk_info).supported[WINED3D_VK_EXT_TRANSFORM_FEEDBACK as usize] != 0
    {
        i = 0 as u32;
        while (i as usize) < (ARRAY_SIZE!((*state).stream_output) as usize) {
            buffer = (*state).stream_output[i as usize].buffer;
            if buffer.is_null() {
                i = i.wrapping_add(1);
                continue;
            }

            wined3d_buffer_acquire_bo_for_write(buffer, core::ptr::addr_of_mut!((*context_vk).c));

            buffer_vk = wined3d_buffer_vk(buffer);
            wined3d_buffer_load(
                core::ptr::addr_of_mut!((*buffer_vk).b),
                core::ptr::addr_of_mut!((*context_vk).c),
                state,
            );
            wined3d_buffer_vk_barrier(buffer_vk, context_vk, WINED3D_BIND_STREAM_OUTPUT);
            wined3d_buffer_invalidate_location(
                core::ptr::addr_of_mut!((*buffer_vk).b),
                !WINED3D_LOCATION_BUFFER,
            );
            if !(*buffer_vk).b.bo_user.valid {
                (*context_vk).set_update_stream_output((1) as _);
            }
            i = i.wrapping_add(1);
        }
        (*context_vk).c.set_transform_feedback_active((1) as _);
    }

    if indexed
        || (wined3d_context_is_graphics_state_dirty(
            core::ptr::addr_of_mut!((*context_vk).c),
            (STATE_INDEXBUFFER as u32),
        ) != false
            && !(*state).index_buffer.is_null())
    {
        buffer_vk = wined3d_buffer_vk((*state).index_buffer);
        wined3d_buffer_load(
            core::ptr::addr_of_mut!((*buffer_vk).b),
            core::ptr::addr_of_mut!((*context_vk).c),
            state,
        );
        wined3d_buffer_vk_barrier(buffer_vk, context_vk, WINED3D_BIND_INDEX_BUFFER);
        if !(*buffer_vk).b.bo_user.valid {
            context_invalidate_state(
                core::ptr::addr_of_mut!((*context_vk).c),
                (STATE_INDEXBUFFER as u32),
            );
        }
    }

    if !indirect_vk.is_null() {
        wined3d_buffer_load(
            core::ptr::addr_of_mut!((*indirect_vk).b),
            core::ptr::addr_of_mut!((*context_vk).c),
            state,
        );
        wined3d_buffer_vk_barrier(indirect_vk, context_vk, WINED3D_BIND_INDIRECT_BUFFER);
    }
}

unsafe extern "C" fn wined3d_context_vk_load_shader_resources(
    mut context_vk: *mut wined3d_context_vk,
    mut state: *const wined3d_state,
    mut pipeline: wined3d_pipeline,
) {
    let mut writes: *mut wined3d_shader_descriptor_writes_vk =
        core::ptr::addr_of_mut!((*context_vk).descriptor_writes);
    let mut bindings: *const wined3d_shader_resource_bindings =
        core::ptr::null::<wined3d_shader_resource_bindings>();
    let mut binding: *const wined3d_shader_resource_binding =
        core::ptr::null::<wined3d_shader_resource_binding>();
    let mut uav_vk: *mut wined3d_unordered_access_view_vk =
        core::ptr::null_mut::<wined3d_unordered_access_view_vk>();
    let mut srv_vk: *mut wined3d_shader_resource_view_vk =
        core::ptr::null_mut::<wined3d_shader_resource_view_vk>();
    let mut uav: *mut wined3d_unordered_access_view =
        core::ptr::null_mut::<wined3d_unordered_access_view>();
    let mut srv: *mut wined3d_shader_resource_view =
        core::ptr::null_mut::<wined3d_shader_resource_view>();
    let mut buffer_vk: *mut wined3d_buffer_vk = core::ptr::null_mut::<wined3d_buffer_vk>();
    let mut buffer: *mut wined3d_buffer = core::ptr::null_mut::<wined3d_buffer>();
    let mut i: size_t = 0;

    match pipeline as ::core::ffi::c_uint {
        WINED3D_PIPELINE_GRAPHICS => {
            bindings = core::ptr::addr_of!((*context_vk).graphics.bindings);
        }
        WINED3D_PIPELINE_COMPUTE => {
            bindings = core::ptr::addr_of!((*context_vk).compute.bindings);
        }
        _ => {
            ERR!("Invalid pipeline %#x.\n", pipeline as ::core::ffi::c_uint);
            return;
        }
    }

    (*writes).count = 0;
    i = 0;
    while i < ((*bindings).count) as _ {
        binding = (*bindings).bindings.offset(i as isize);

        'sw: {
            match (*binding).shader_descriptor_type as ::core::ffi::c_uint {
                WINED3D_SHADER_DESCRIPTOR_TYPE_CBV => {
                    buffer = (*state).cb[(*binding).shader_type as usize]
                        [(*binding).resource_idx as usize]
                        .buffer;
                    if buffer.is_null() {
                        break 'sw;
                    }

                    buffer_vk = wined3d_buffer_vk(buffer);
                    wined3d_buffer_load(buffer, core::ptr::addr_of_mut!((*context_vk).c), state);
                    if !(*buffer_vk).b.bo_user.valid {
                        if pipeline as ::core::ffi::c_uint == WINED3D_PIPELINE_GRAPHICS {
                            context_invalidate_state(
                                core::ptr::addr_of_mut!((*context_vk).c),
                                (STATE_GRAPHICS_CONSTANT_BUFFER!(((*binding).shader_type) as i32)) as u32,
                            );
                        } else {
                            context_invalidate_compute_state(
                                core::ptr::addr_of_mut!((*context_vk).c),
                                (STATE_COMPUTE_CONSTANT_BUFFER) as DWORD,
                            );
                        }
                    }
                    wined3d_buffer_vk_barrier(buffer_vk, context_vk, WINED3D_BIND_CONSTANT_BUFFER);
                }

                WINED3D_SHADER_DESCRIPTOR_TYPE_SRV => {
                    srv = (*state).shader_resource_view[(*binding).shader_type as usize]
                        [(*binding).resource_idx as usize];
                    if srv.is_null() {
                        break 'sw;
                    }

                    srv_vk = wined3d_shader_resource_view_vk(srv);
                    if (*(*srv).resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER {
                        if !(*srv_vk).view_vk.bo_user.valid {
                            wined3d_shader_resource_view_vk_update_buffer(srv_vk, context_vk);
                            if pipeline as ::core::ffi::c_uint == WINED3D_PIPELINE_GRAPHICS {
                                context_invalidate_state(
                                    core::ptr::addr_of_mut!((*context_vk).c),
                                    (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32),
                                );
                            } else {
                                context_invalidate_compute_state(
                                    core::ptr::addr_of_mut!((*context_vk).c),
                                    (STATE_COMPUTE_SHADER_RESOURCE_BINDING) as DWORD,
                                );
                            }
                        }
                        wined3d_buffer_load(
                            buffer_from_resource((*srv).resource),
                            core::ptr::addr_of_mut!((*context_vk).c),
                            state,
                        );
                    } else {
                        wined3d_texture_load(
                            texture_from_resource((*srv).resource),
                            core::ptr::addr_of_mut!((*context_vk).c),
                            (FALSE) as BOOL,
                        );
                    }
                    wined3d_shader_resource_view_vk_barrier(srv_vk, context_vk, WINED3D_BIND_SHADER_RESOURCE);
                }

                WINED3D_SHADER_DESCRIPTOR_TYPE_UAV => {
                    uav = (*state).unordered_access_view[pipeline as usize]
                        [(*binding).resource_idx as usize];
                    if uav.is_null() {
                        break 'sw;
                    }

                    uav_vk = wined3d_unordered_access_view_vk(uav);
                    if (*(*uav).resource).type_ as ::core::ffi::c_uint == WINED3D_RTYPE_BUFFER {
                        wined3d_buffer_acquire_bo_for_write(
                            buffer_from_resource((*uav).resource),
                            core::ptr::addr_of_mut!((*context_vk).c),
                        );
                        if !(*uav_vk).view_vk.bo_user.valid {
                            wined3d_unordered_access_view_vk_update(uav_vk, context_vk);
                            if pipeline as ::core::ffi::c_uint == WINED3D_PIPELINE_GRAPHICS {
                                context_invalidate_state(
                                    core::ptr::addr_of_mut!((*context_vk).c),
                                    (STATE_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING as u32),
                                );
                            } else {
                                context_invalidate_compute_state(
                                    core::ptr::addr_of_mut!((*context_vk).c),
                                    (STATE_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING) as DWORD,
                                );
                            }
                        }
                        wined3d_buffer_load(
                            buffer_from_resource((*uav).resource),
                            core::ptr::addr_of_mut!((*context_vk).c),
                            state,
                        );
                        wined3d_unordered_access_view_invalidate_location(uav, !WINED3D_LOCATION_BUFFER as uint32_t);
                    } else {
                        wined3d_texture_load(
                            texture_from_resource((*uav).resource),
                            core::ptr::addr_of_mut!((*context_vk).c),
                            (FALSE) as BOOL,
                        );
                        wined3d_unordered_access_view_invalidate_location(uav, !WINED3D_LOCATION_TEXTURE_RGB as uint32_t);
                    }
                    wined3d_unordered_access_view_vk_barrier(uav_vk, context_vk, WINED3D_BIND_UNORDERED_ACCESS);
                }

                WINED3D_SHADER_DESCRIPTOR_TYPE_UAV_COUNTER | WINED3D_SHADER_DESCRIPTOR_TYPE_SAMPLER => {}

                _ => {
                    ERR!(
                        "Invalid descriptor type %#x.\n",
                        (*binding).shader_descriptor_type as ::core::ffi::c_uint
                    );
                }
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_context_vk_prepare_used_rtv(
    context_vk: *mut wined3d_context_vk,
    rtv: *mut wined3d_rendertarget_view,
) {
    if (wined3d_rendertarget_view_get_locations(rtv) & WINED3D_LOCATION_CLEARED) != 0 {
        /* Need to restart render pass or the clear won't happen. */
        wined3d_context_vk_end_current_render_pass(context_vk);
        wined3d_rendertarget_view_prepare_location(
            rtv,
            core::ptr::addr_of_mut!((*context_vk).c),
            (*(*rtv).resource).draw_binding as u32,
        );
    } else {
        wined3d_rendertarget_view_load_location(
            rtv,
            core::ptr::addr_of_mut!((*context_vk).c),
            (*(*rtv).resource).draw_binding as u32,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_apply_draw_state(context_vk: *mut wined3d_context_vk, state: *const wined3d_state, indirect_vk: *mut wined3d_buffer_vk, indexed: bool) -> VkCommandBuffer {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let d3d_info: *const wined3d_d3d_info = (*context_vk).c.d3d_info;
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let b: *const wined3d_blend_state = (*state).blend_state;
    let mut prev_sample_count: VkSampleCountFlagBits = 0;
    let mut sample_count: VkSampleCountFlagBits = 0;
    let dual_source_blend: bool = !b.is_null() && (*b).dual_source != 0;
    let mut dsv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut buffer_vk: *mut wined3d_buffer_vk = core::ptr::null_mut();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;
    let mut invalidate_rt: ::core::ffi::c_uint = 0;
    let mut null_buffer_binding: uint32_t = 0;
    let mut invalidate_ds: bool = false;

    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_SHADER!((WINED3D_SHADER_TYPE_PIXEL) as i32))
        || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_FRAMEBUFFER as u32))
        || dual_source_blend != (((*context_vk).c.last_was_dual_source_blend()) != 0)
    {
        (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() | ((1 << WINED3D_SHADER_TYPE_PIXEL))) as _);
        (*context_vk).c.set_last_was_dual_source_blend((dual_source_blend) as _);
    }
    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_SHADER!((WINED3D_SHADER_TYPE_VERTEX) as i32)) {
        (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() | ((1 << WINED3D_SHADER_TYPE_VERTEX))) as _);
    }
    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_SHADER!((WINED3D_SHADER_TYPE_GEOMETRY) as i32)) {
        (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() | ((1 << WINED3D_SHADER_TYPE_GEOMETRY))) as _);
    }
    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_SHADER!((WINED3D_SHADER_TYPE_HULL) as i32)) {
        (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() | ((1 << WINED3D_SHADER_TYPE_HULL) | (1 << WINED3D_SHADER_TYPE_DOMAIN))) as _);
    }
    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_SHADER!((WINED3D_SHADER_TYPE_DOMAIN) as i32)) {
        (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() | ((1 << WINED3D_SHADER_TYPE_DOMAIN))) as _);
    }

    prev_sample_count = (*context_vk).sample_count;
    (*context_vk).sample_count = 0;
    i = 0;
    while i < ARRAY_SIZE!((*state).fb.render_targets) as ::core::ffi::c_uint {
        'body: {
            rtv = (*state).fb.render_targets[i as usize];
            if rtv.is_null() || (*(*rtv).format).id == WINED3DFMT_NULL {
                break 'body;
            }

            if wined3d_blend_state_get_writemask((*state).blend_state, i) != 0 {
                /* We handle clears at the beginning of the render pass, no need for an explicit clear
                 * first. */
                wined3d_context_vk_prepare_used_rtv(context_vk, rtv);
                invalidate_rt |= (1 << i);
            } else {
                wined3d_rendertarget_view_prepare_location(rtv, core::ptr::addr_of_mut!((*context_vk).c), ((*(*rtv).resource).draw_binding) as _);
            }

            sample_count = max!(1, wined3d_resource_get_sample_count((*rtv).resource)) as VkSampleCountFlagBits;
            if (*context_vk).sample_count == 0 {
                (*context_vk).sample_count = sample_count;
            } else if (*context_vk).sample_count != sample_count {
                FIXME!("Inconsistent sample counts (%u != %u).\n", (*context_vk).sample_count, sample_count);
            }
        }
        i = i.wrapping_add(1);
    }

    dsv = (*state).fb.depth_stencil;
    if !dsv.is_null() {
        if wined3d_state_uses_depth_buffer(state) != false {
            wined3d_context_vk_prepare_used_rtv(context_vk, dsv);
        } else {
            wined3d_rendertarget_view_prepare_location(dsv, core::ptr::addr_of_mut!((*context_vk).c), ((*(*dsv).resource).draw_binding) as _);
        }

        if (*state).depth_stencil_state.is_null() || (*(*state).depth_stencil_state).writes_ds != false {
            invalidate_ds = true;
        }

        sample_count = max!(1, wined3d_resource_get_sample_count((*dsv).resource)) as VkSampleCountFlagBits;
        if (*context_vk).sample_count == 0 {
            (*context_vk).sample_count = sample_count;
        } else if (*context_vk).sample_count != sample_count {
            FIXME!("Inconsistent sample counts (%u != %u).\n", (*context_vk).sample_count, sample_count);
        }
    }

    if (*context_vk).sample_count == 0 {
        (*context_vk).sample_count = VK_SAMPLE_COUNT_1_BIT;
    }

    if (*context_vk).sample_count != prev_sample_count {
        (*context_vk).c.set_update_multisample_state((1) as _);
    }

    if (*context_vk).c.shader_update_mask() & !(1 << WINED3D_SHADER_TYPE_COMPUTE) != 0 {
        (*(*device_vk).d.shader_backend).shader_apply_draw_state.unwrap_unchecked()(
            (*device_vk).d.shader_priv,
            core::ptr::addr_of_mut!((*context_vk).c),
            state,
        );
        if ((*context_vk).graphics.vk_pipeline_layout == 0) {
            ERR!("No pipeline layout set.\n");
            return core::ptr::null_mut();
        }
        (*context_vk).c.set_update_shader_resource_bindings((1) as _);
        (*context_vk).c.set_update_unordered_access_view_bindings((1) as _);
    }

    wined3d_context_vk_load_shader_resources(context_vk, state, WINED3D_PIPELINE_GRAPHICS);

    wined3d_context_vk_load_buffers(context_vk, state, indirect_vk, indexed);

    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_FRAMEBUFFER as u32)) != false {
        wined3d_context_vk_end_current_render_pass(context_vk);
    }

    if wined3d_context_vk_begin_render_pass(context_vk, state, vk_info) == false {
        ERR!("Failed to begin render pass.\n");
        return core::ptr::null_mut();
    }
    vk_command_buffer = (*context_vk).current_command_buffer.vk_command_buffer;

    while invalidate_rt != 0 {
        i = wined3d_bit_scan(core::ptr::addr_of_mut!(invalidate_rt)) as ::core::ffi::c_uint;
        rtv = (*state).fb.render_targets[i as usize];
        wined3d_rendertarget_view_validate_location(rtv, ((*(*rtv).resource).draw_binding) as uint32_t);
        wined3d_rendertarget_view_invalidate_location(rtv, (!(*(*rtv).resource).draw_binding) as _);
    }

    if invalidate_ds {
        wined3d_rendertarget_view_validate_location(dsv, ((*(*dsv).resource).draw_binding) as uint32_t);
        wined3d_rendertarget_view_invalidate_location(dsv, (!(*(*dsv).resource).draw_binding) as _);
    }

    if wined3d_context_vk_update_graphics_pipeline_key(
        context_vk,
        state,
        (*context_vk).graphics.vk_pipeline_layout,
        core::ptr::addr_of_mut!(null_buffer_binding),
    ) != false
        || ((*context_vk).graphics.vk_pipeline == 0)
    {
        (*context_vk).graphics.vk_pipeline = wined3d_context_vk_get_graphics_pipeline(context_vk);
        if ((*context_vk).graphics.vk_pipeline == 0) {
            ERR!("Failed to get graphics pipeline.\n");
            return core::ptr::null_mut();
        }

        ((*vk_info).vk_ops.vkCmdBindPipeline.unwrap_unchecked())(
            vk_command_buffer,
            VK_PIPELINE_BIND_POINT_GRAPHICS,
            (*context_vk).graphics.vk_pipeline,
        );
        if null_buffer_binding != !0 {
            let mut offset: VkDeviceSize = 0;
            ((*vk_info).vk_ops.vkCmdBindVertexBuffers.unwrap_unchecked())(
                vk_command_buffer,
                null_buffer_binding,
                1,
                core::ptr::addr_of_mut!((*device_vk).null_resources_vk.buffer_info.buffer),
                core::ptr::addr_of_mut!(offset),
            );
        }
    }

    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_STENCIL_REF as u32)) != false && !dsv.is_null() {
        ((*vk_info).vk_ops.vkCmdSetStencilReference.unwrap_unchecked())(
            vk_command_buffer,
            VK_STENCIL_FACE_FRONT_AND_BACK,
            (*state).stencil_ref & wined3d_mask_from_size(((*(*dsv).format).stencil_size) as _),
        );
    }

    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_STREAMSRC as u32)) != false {
        wined3d_context_vk_bind_vertex_buffers(context_vk, vk_command_buffer, state, vk_info);
    }

    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_STREAM_OUTPUT as u32)) != false {
        (*context_vk).set_update_stream_output((1) as _);
        (*context_vk).c.set_transform_feedback_paused((0) as _);
    }
    if (*context_vk).c.transform_feedback_active() != 0 && (*context_vk).update_stream_output() != 0 {
        wined3d_context_vk_bind_stream_output_buffers(context_vk, vk_command_buffer, state, vk_info);
        (*context_vk).set_update_stream_output((0) as _);
    }

    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_INDEXBUFFER as u32)) != false
        && !(*state).index_buffer.is_null()
    {
        let mut buffer_info: *const VkDescriptorBufferInfo = core::ptr::null();
        let mut idx_type: VkIndexType;

        if (*state).index_format == WINED3DFMT_R16_UINT {
            idx_type = VK_INDEX_TYPE_UINT16;
        } else {
            idx_type = VK_INDEX_TYPE_UINT32;
        }
        buffer_vk = wined3d_buffer_vk((*state).index_buffer);
        buffer_info = wined3d_buffer_vk_get_buffer_info(buffer_vk);
        wined3d_context_vk_reference_bo(context_vk, wined3d_bo_vk((*buffer_vk).b.buffer_object));
        ((*vk_info).vk_ops.vkCmdBindIndexBuffer.unwrap_unchecked())(
            vk_command_buffer,
            (*buffer_info).buffer,
            (*buffer_info).offset + (*state).index_offset as VkDeviceSize,
            idx_type,
        );
    }

    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_PIXEL) as i32)) != false
        || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_VERTEX) as i32)) != false
        || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_GEOMETRY) as i32)) != false
        || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_HULL) as i32)) != false
        || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_DOMAIN) as i32)) != false
        || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32)) != false
    {
        (*context_vk).c.set_update_shader_resource_bindings((1) as _);
    }
    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING as u32)) != false {
        (*context_vk).c.set_update_unordered_access_view_bindings((1) as _);
    }

    if (*context_vk).c.update_shader_resource_bindings() != 0 || (*context_vk).c.update_unordered_access_view_bindings() != 0 {
        if wined3d_context_vk_update_descriptors(context_vk, vk_command_buffer, state, WINED3D_PIPELINE_GRAPHICS) == false {
            ERR!("Failed to update shader descriptors.\n");
            return core::ptr::null_mut();
        }

        (*context_vk).c.set_update_shader_resource_bindings((0) as _);
        (*context_vk).c.set_update_unordered_access_view_bindings((0) as _);
    }

    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_BLEND_FACTOR as u32)) != false {
        ((*vk_info).vk_ops.vkCmdSetBlendConstants.unwrap_unchecked())(vk_command_buffer, core::ptr::addr_of!((*state).blend_factor.r));
    }

    if wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_VIEWPORT as u32)) != false
        || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_SCISSORRECT as u32)) != false
        || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_RASTERIZER as u32)) != false
    {
        let viewport_count: ::core::ffi::c_uint = if (*(*context_vk).vk_info).multiple_viewports != false {
            WINED3D_MAX_VIEWPORTS
        } else {
            1
        };
        let mut viewports: [VkViewport; (WINED3D_MAX_VIEWPORTS as _)] = core::mem::zeroed();
        let mut scissors: [VkRect2D; (WINED3D_MAX_VIEWPORTS as _)] = core::mem::zeroed();
        let mut i: ::core::ffi::c_uint;

        i = 0;
        while i < viewport_count {
            'vp: {
                let src_viewport: *const wined3d_viewport = core::ptr::addr_of!((*state).viewports[i as usize]);
                let viewport: *mut VkViewport = core::ptr::addr_of_mut!(viewports[i as usize]);
                let scissor: *mut VkRect2D = core::ptr::addr_of_mut!(scissors[i as usize]);

                if i >= (*state).viewport_count {
                    (*viewport).x = 0.0;
                    (*viewport).y = 0.0;
                    (*viewport).width = 1.0;
                    (*viewport).height = 1.0;
                    (*viewport).minDepth = 0.0;
                    (*viewport).maxDepth = 0.0;
                    memset(scissor as *mut ::core::ffi::c_void, 0, core::mem::size_of::<VkRect2D>());
                    break 'vp;
                }

                (*viewport).x = (*src_viewport).x;
                (*viewport).y = (*src_viewport).y;
                (*viewport).width = (*src_viewport).width;
                (*viewport).height = (*src_viewport).height;
                (*viewport).minDepth = (*src_viewport).min_z;
                (*viewport).maxDepth = (*src_viewport).max_z;

                if !(*state).rasterizer_state.is_null() && (*(*state).rasterizer_state).desc.scissor != 0 {
                    let r: *const RECT = core::ptr::addr_of!((*state).scissor_rects[i as usize]);

                    if i >= (*state).scissor_rect_count {
                        memset(scissor as *mut ::core::ffi::c_void, 0, core::mem::size_of::<VkRect2D>());
                        break 'vp;
                    }

                    (*scissor).offset.x = (*r).left as int32_t;
                    (*scissor).offset.y = (*r).top as int32_t;
                    (*scissor).extent.width = ((*r).right - (*r).left) as uint32_t;
                    (*scissor).extent.height = ((*r).bottom - (*r).top) as uint32_t;
                } else {
                    (*scissor).offset.x = (*src_viewport).x as int32_t;
                    (*scissor).offset.y = (*src_viewport).y as int32_t;
                    (*scissor).extent.width = (*src_viewport).width as uint32_t;
                    (*scissor).extent.height = (*src_viewport).height as uint32_t;
                }

                /* Scissor offsets need to be non-negative (VUID-VkPipelineViewportStateCreateInfo-x-02821) */
                if (*scissor).offset.x < 0 {
                    (*scissor).offset.x = 0;
                }
                if (*scissor).offset.y < 0 {
                    (*scissor).offset.y = 0;
                }
                (*viewport).y += (*viewport).height;
                (*viewport).height = -(*viewport).height;
            }
            i = i.wrapping_add(1);
        }

        ((*vk_info).vk_ops.vkCmdSetViewport.unwrap_unchecked())(vk_command_buffer, 0, viewport_count, viewports.as_mut_ptr());
        ((*vk_info).vk_ops.vkCmdSetScissor.unwrap_unchecked())(vk_command_buffer, 0, viewport_count, scissors.as_mut_ptr());
    }

    if (*vk_info).dynamic_state2 != false && (*context_vk).c.update_primitive_type() != 0 {
        ((*vk_info).vk_ops.vkCmdSetPrimitiveTopologyEXT.unwrap_unchecked())(
            vk_command_buffer,
            vk_topology_from_wined3d((*state).primitive_type),
        );
        if (*d3d_info).wined3d_creation_flags & WINED3D_NO_PRIMITIVE_RESTART == 0 {
            ((*vk_info).vk_ops.vkCmdSetPrimitiveRestartEnableEXT.unwrap_unchecked())(
                vk_command_buffer,
                !wined3d_primitive_type_is_list((*state).primitive_type) as VkBool32,
            );
        }
        (*context_vk).c.set_update_primitive_type((0) as _);
    }

    if (*vk_info).dynamic_patch_vertex_count != false && (*context_vk).c.update_patch_vertex_count() != 0 {
        if (*state).patch_vertex_count != 0 {
            ((*vk_info).vk_ops.vkCmdSetPatchControlPointsEXT.unwrap_unchecked())(vk_command_buffer, (*state).patch_vertex_count);
        }
        (*context_vk).c.set_update_patch_vertex_count((0) as _);
    }

    if (*vk_info).dynamic_multisample_state != false && (*context_vk).c.update_multisample_state() != 0 {
        let sample_count_0: VkSampleCountFlagBits = (*context_vk).sample_count;

        ((*vk_info).vk_ops.vkCmdSetAlphaToCoverageEnableEXT.unwrap_unchecked())(
            vk_command_buffer,
            (!(*state).blend_state.is_null() && (*(*state).blend_state).desc.alpha_to_coverage != 0) as VkBool32,
        );
        ((*vk_info).vk_ops.vkCmdSetRasterizationSamplesEXT.unwrap_unchecked())(vk_command_buffer, sample_count_0);
        ((*vk_info).vk_ops.vkCmdSetSampleMaskEXT.unwrap_unchecked())(
            vk_command_buffer,
            sample_count_0,
            core::ptr::addr_of!((*state).sample_mask),
        );
        (*context_vk).c.set_update_multisample_state((0) as _);
    }

    if (*vk_info).dynamic_rasterizer_state != false
        && (wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_RASTERIZER as u32)) != false
            || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), STATE_SHADER!((WINED3D_SHADER_TYPE_GEOMETRY) as i32)) != false)
    {
        wined3d_context_vk_set_dynamic_rasterizer_state(context_vk, vk_command_buffer, vk_info, state);
    }

    if (*vk_info).dynamic_blend_state != false
        && (wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_BLEND as u32)) != false
            || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_FRAMEBUFFER as u32)) != false)
    {
        wined3d_context_vk_set_dynamic_blend_state(context_vk, vk_command_buffer, vk_info, state);
    }

    if (*vk_info).supported[(WINED3D_VK_EXT_EXTENDED_DYNAMIC_STATE) as usize] != 0
        && (wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_DEPTH_STENCIL as u32)) != false
            || wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_FRAMEBUFFER as u32)) != false)
    {
        let d: *const wined3d_depth_stencil_state = (*state).depth_stencil_state;

        if !d.is_null() {
            let stencil_enable: VkBool32 =
                (!(*state).fb.depth_stencil.is_null() && (*d).desc.stencil != 0) as VkBool32;

            ((*vk_info).vk_ops.vkCmdSetDepthTestEnableEXT.unwrap_unchecked())(vk_command_buffer, ((*d).desc.depth) as u32);
            ((*vk_info).vk_ops.vkCmdSetDepthWriteEnableEXT.unwrap_unchecked())(vk_command_buffer, ((*d).desc.depth_write) as u32);
            ((*vk_info).vk_ops.vkCmdSetDepthCompareOpEXT.unwrap_unchecked())(
                vk_command_buffer,
                vk_compare_op_from_wined3d((*d).desc.depth_func),
            );
            ((*vk_info).vk_ops.vkCmdSetStencilTestEnableEXT.unwrap_unchecked())(vk_command_buffer, stencil_enable);
            if stencil_enable != 0 {
                ((*vk_info).vk_ops.vkCmdSetStencilOpEXT.unwrap_unchecked())(
                    vk_command_buffer,
                    VK_STENCIL_FACE_FRONT_BIT,
                    vk_stencil_op_from_wined3d((*d).desc.front.fail_op),
                    vk_stencil_op_from_wined3d((*d).desc.front.pass_op),
                    vk_stencil_op_from_wined3d((*d).desc.front.depth_fail_op),
                    vk_compare_op_from_wined3d((*d).desc.front.func),
                );
                ((*vk_info).vk_ops.vkCmdSetStencilOpEXT.unwrap_unchecked())(
                    vk_command_buffer,
                    VK_STENCIL_FACE_BACK_BIT,
                    vk_stencil_op_from_wined3d((*d).desc.back.fail_op),
                    vk_stencil_op_from_wined3d((*d).desc.back.pass_op),
                    vk_stencil_op_from_wined3d((*d).desc.back.depth_fail_op),
                    vk_compare_op_from_wined3d((*d).desc.back.func),
                );
                ((*vk_info).vk_ops.vkCmdSetStencilCompareMask.unwrap_unchecked())(
                    vk_command_buffer,
                    VK_STENCIL_FACE_FRONT_AND_BACK,
                    (*d).desc.stencil_read_mask,
                );
                ((*vk_info).vk_ops.vkCmdSetStencilWriteMask.unwrap_unchecked())(
                    vk_command_buffer,
                    VK_STENCIL_FACE_FRONT_AND_BACK,
                    (*d).desc.stencil_write_mask,
                );
            }
        } else {
            ((*vk_info).vk_ops.vkCmdSetDepthTestEnableEXT.unwrap_unchecked())(vk_command_buffer, VK_TRUE);
            ((*vk_info).vk_ops.vkCmdSetDepthWriteEnableEXT.unwrap_unchecked())(vk_command_buffer, VK_TRUE);
            ((*vk_info).vk_ops.vkCmdSetDepthCompareOpEXT.unwrap_unchecked())(vk_command_buffer, VK_COMPARE_OP_LESS);
            ((*vk_info).vk_ops.vkCmdSetStencilTestEnableEXT.unwrap_unchecked())(vk_command_buffer, VK_FALSE);
        }
    }

    memset(
        core::ptr::addr_of_mut!((*context_vk).c.dirty_graphics_states) as *mut ::core::ffi::c_void,
        0,
        core::mem::size_of::<[uint32_t; 13]>(),
    );
    (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() & (1 << WINED3D_SHADER_TYPE_COMPUTE)) as _);

    return vk_command_buffer;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_apply_compute_state(context_vk: *mut wined3d_context_vk, state: *const wined3d_state, indirect_vk: *mut wined3d_buffer_vk) -> VkCommandBuffer {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();

    wined3d_context_vk_end_current_render_pass(context_vk);

    if wined3d_context_is_compute_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_COMPUTE_SHADER as u32)) {
        (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() | (1 << WINED3D_SHADER_TYPE_COMPUTE)) as _);
    }

    if (*context_vk).c.shader_update_mask() & (1 << WINED3D_SHADER_TYPE_COMPUTE) != 0
    {
        ((*(*device_vk).d.shader_backend).shader_apply_compute_state.unwrap_unchecked())(
            (*device_vk).d.shader_priv,
            core::ptr::addr_of_mut!((*context_vk).c),
            state,
        );
        if ((*context_vk).compute.vk_pipeline == 0) {
            ERR!("No compute pipeline set.\n");
            return core::ptr::null_mut();
        }
        (*context_vk).c.set_update_compute_shader_resource_bindings((1) as _);
        (*context_vk).c.set_update_compute_unordered_access_view_bindings((1) as _);
        (*context_vk).set_update_compute_pipeline((1) as _);
    }

    wined3d_context_vk_load_shader_resources(context_vk, state, WINED3D_PIPELINE_COMPUTE);

    if !indirect_vk.is_null() {
        wined3d_buffer_load_location(
            core::ptr::addr_of_mut!((*indirect_vk).b),
            core::ptr::addr_of_mut!((*context_vk).c),
            WINED3D_LOCATION_BUFFER,
        );
        wined3d_buffer_vk_barrier(indirect_vk, context_vk, WINED3D_BIND_INDIRECT_BUFFER);
    }

    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
    if vk_command_buffer.is_null() {
        ERR!("Failed to get command buffer.\n");
        return core::ptr::null_mut();
    }

    if (*context_vk).update_compute_pipeline() != 0 {
        ((*vk_info).vk_ops.vkCmdBindPipeline.unwrap_unchecked())(
            vk_command_buffer,
            VK_PIPELINE_BIND_POINT_COMPUTE,
            (*context_vk).compute.vk_pipeline
        );
        (*context_vk).set_update_compute_pipeline((0) as _);
    }

    if wined3d_context_is_compute_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_COMPUTE_CONSTANT_BUFFER as u32))
        || wined3d_context_is_compute_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_COMPUTE_SHADER_RESOURCE_BINDING as u32))
    {
        (*context_vk).c.set_update_compute_shader_resource_bindings((1) as _);
    }
    if wined3d_context_is_compute_state_dirty(core::ptr::addr_of_mut!((*context_vk).c), (STATE_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING as u32)) {
        (*context_vk).c.set_update_compute_unordered_access_view_bindings((1) as _);
    }

    if (*context_vk).c.update_compute_shader_resource_bindings() != 0
        || (*context_vk).c.update_compute_unordered_access_view_bindings() != 0
    {
        if !wined3d_context_vk_update_descriptors(context_vk, vk_command_buffer, state, WINED3D_PIPELINE_COMPUTE) {
            ERR!("Failed to update shader descriptors.\n");
            return core::ptr::null_mut();
        }

        (*context_vk).c.set_update_compute_shader_resource_bindings((0) as _);
        (*context_vk).c.set_update_compute_unordered_access_view_bindings((0) as _);
    }

    memset(
        core::ptr::addr_of_mut!((*context_vk).c.dirty_compute_states) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[u32; 1]>(),
    );
    (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() & (!(1 << WINED3D_SHADER_TYPE_COMPUTE))) as _);

    return vk_command_buffer;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_vk_init(context_vk: *mut wined3d_context_vk, swapchain: *mut wined3d_swapchain) -> HRESULT {
    let mut command_pool_info: VkCommandPoolCreateInfo = core::mem::zeroed();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut adapter_vk: *mut wined3d_adapter_vk = core::ptr::null_mut();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut();
    let mut vr: VkResult;

    memset(
        context_vk as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<wined3d_context_vk>() as size_t,
    );
    wined3d_context_init(&raw mut (*context_vk).c, swapchain);
    device_vk = wined3d_device_vk((*swapchain).device);
    adapter_vk = wined3d_adapter_vk((*device_vk).d.adapter);
    vk_info = &raw mut (*adapter_vk).vk_info;
    (*context_vk).vk_info = vk_info;

    command_pool_info.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
    command_pool_info.pNext = core::ptr::null::<::core::ffi::c_void>();
    command_pool_info.flags = (VK_COMMAND_POOL_CREATE_TRANSIENT_BIT | VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT) as VkCommandPoolCreateFlags;
    command_pool_info.queueFamilyIndex = (*device_vk).vk_queue_family_index;
    vr = (*vk_info)
        .vk_ops
        .vkCreateCommandPool
        .unwrap_unchecked()(
        (*device_vk).vk_device,
        &raw mut command_pool_info,
        core::ptr::null::<VkAllocationCallbacks>(),
        &raw mut (*context_vk).vk_command_pool,
    );
    if (vr as ::core::ffi::c_int) < 0 {
        ERR!(
            "Failed to create Vulkan command pool, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        wined3d_context_cleanup(&raw mut (*context_vk).c);
        return E_FAIL;
    }
    (*context_vk).current_command_buffer.id = 1 as uint64_t;

    wined3d_context_vk_init_graphics_pipeline_key(context_vk);

    list_init(&raw mut (*context_vk).render_pass_queries);
    list_init(&raw mut (*context_vk).active_queries);
    list_init(&raw mut (*context_vk).completed_query_pools);
    list_init(&raw mut (*context_vk).free_occlusion_query_pools);
    list_init(&raw mut (*context_vk).free_timestamp_query_pools);
    list_init(&raw mut (*context_vk).free_pipeline_statistics_query_pools);
    list_init(&raw mut (*context_vk).free_stream_output_statistics_query_pools);

    wine_rb_init(
        &raw mut (*context_vk).render_passes,
        Some(
            wined3d_render_pass_vk_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const rb_entry,
                ) -> ::core::ffi::c_int,
        ),
    );
    wine_rb_init(
        &raw mut (*context_vk).pipeline_layouts,
        Some(
            wined3d_pipeline_layout_vk_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const rb_entry,
                ) -> ::core::ffi::c_int,
        ),
    );
    wine_rb_init(
        &raw mut (*context_vk).graphics_pipelines,
        Some(
            wined3d_graphics_pipeline_vk_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const rb_entry,
                ) -> ::core::ffi::c_int,
        ),
    );
    wine_rb_init(
        &raw mut (*context_vk).bo_slab_available,
        Some(
            wined3d_bo_slab_vk_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const rb_entry,
                ) -> ::core::ffi::c_int,
        ),
    );

    return WINED3D_OK;
}
