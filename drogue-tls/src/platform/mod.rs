mod alloc;
mod cortex_m_alloc;

use crate::platform::alloc::layout::Layout;
use crate::platform::cortex_m_alloc::CortexMHeap;
use drogue_tls_sys::platform_set_calloc_free;
use drogue_tls_sys::types::{c_char, c_void};

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn strlen(p: *const c_char) -> usize {
    let mut n = 0;
    unsafe {
        while *p.add(n) != 0 {
            n += 1;
        }
    }
    log::info!("strlen = {}", n);
    n
}

static mut ALLOCATOR: Option<CortexMHeap> = Option::None;

pub fn setup_platform(start: usize, size: usize) {
    let heap = CortexMHeap::empty();
    unsafe {
        heap.init(start, size);
        ALLOCATOR.replace(heap);
    }
    unsafe { platform_set_calloc_free(Some(platform_calloc_f), Some(platform_free_f)) };
}

extern "C" fn platform_calloc_f(count: usize, size: usize) -> *mut c_void {
    let requested_size = count * size;
    let header_size = 2 * 4;
    let total_size = header_size + requested_size;
    let layout = Layout::from_size_align(total_size, 4)
        .unwrap()
        .pad_to_align();

    unsafe {
        if let Some(ref alloc) = ALLOCATOR {
            let mut ptr = alloc.alloc(layout) as *mut usize;
            *ptr = layout.size();
            ptr = ptr.offset(1);
            *ptr = layout.align();
            ptr = ptr.offset(1);
            ptr as *mut c_void
        } else {
            core::ptr::null_mut::<c_void>()
        }
    }
}

extern "C" fn platform_free_f(ptr: *mut c_void) {
    unsafe {
        if let Some(ref alloc) = ALLOCATOR {
            let mut ptr = ptr as *mut usize;
            ptr = ptr.offset(-1);
            let align = *ptr;
            ptr = ptr.offset(-1);
            let size = *ptr;
            alloc.dealloc(
                ptr as *mut u8,
                Layout::from_size_align(size, align).unwrap(),
            );
        }
    }
}
