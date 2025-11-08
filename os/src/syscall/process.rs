//! Process management syscalls
use crate::mm::translated_byte_buffer;
use crate::task::{
    change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_count,
    suspend_current_and_run_next,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is split by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let tv = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let token = current_user_token();
    let buffers = translated_byte_buffer(token, ts as *const u8, core::mem::size_of::<TimeVal>());

    let mut offset = 0;
    let tv_bytes = unsafe {
        core::slice::from_raw_parts(
            (&tv as *const TimeVal) as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };

    for buffer in buffers {
        let copy_len = buffer.len().min(tv_bytes.len() - offset);
        buffer[..copy_len].copy_from_slice(&tv_bytes[offset..offset + copy_len]);
        offset += copy_len;
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        // trace_request == 0: read from user space virtual address
        0 => {
            let token = current_user_token();
            let buffers = translated_byte_buffer(token, id as *const u8, 1);
            if buffers.is_empty() {
                return -1;
            }
            buffers[0][0] as isize
        }
        // trace_request == 1: write to user space virtual address
        1 => {
            let token = current_user_token();
            let mut buffers = translated_byte_buffer(token, id as *mut u8, 1);
            if buffers.is_empty() {
                return -1;
            }
            buffers[0][0] = data as u8;
            0
        }
        // trace_request == 2: query syscall count
        2 => get_syscall_count(id) as isize,
        _ => -1,
    }
}

/// 申请长度为 len 字节的物理内存（不要求实际物理内存位置，可以随便找一块），将其映射到 start 开始的虚存，内存页属性为 prot
/// start 需要映射的虚存起始地址，要求按页对齐
/// len 映射字节长度，可以为 0
/// prot：第 0 位表示是否可读，第 1 位表示是否可写，第 2 位表示是否可执行。其他位无效且必须为 0
/// 返回值：执行成功则返回 0，错误返回 -1
/// 可能的错误：
/// - start 没有按页大小对齐
/// - prot & !0x7 != 0 (prot 其余位必须为0)
/// - prot & 0x7 = 0 (这样的内存无意义)
/// - [start, start + len) 中存在已经被映射的页
/// - 物理内存不足
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!(
        "kernel: sys_mmap start={:#x}, len={:#x}, prot={:#x}",
        start,
        len,
        prot
    );

    // 检查 prot 参数的有效性
    if prot & !0x7 != 0 {
        trace!("sys_mmap: invalid prot bits");
        return -1;
    }
    if prot & 0x7 == 0 {
        trace!("sys_mmap: prot cannot be 0");
        return -1;
    }

    // 检查 len 为 0 的情况
    if len == 0 {
        return 0;
    }

    let start_va = crate::mm::VirtAddr::from(start);
    // 检查地址是否按页对齐
    if !start_va.aligned() {
        trace!("sys_mmap: start address not aligned");
        return -1;
    }

    let end_va = crate::mm::VirtAddr::from(start + len);

    // 构造 MapPermission
    let mut map_perm = crate::mm::MapPermission::U; // 用户态可访问
    if prot & 0x1 != 0 {
        // 可读
        map_perm |= crate::mm::MapPermission::R;
    }
    if prot & 0x2 != 0 {
        // 可写
        map_perm |= crate::mm::MapPermission::W;
    }
    if prot & 0x4 != 0 {
        // 可执行
        map_perm |= crate::mm::MapPermission::X;
    }

    // 调用新的接口来修改当前任务的内存空间
    if crate::task::mmap_for_current_task(start_va, end_va, map_perm) {
        trace!(
            "sys_mmap: successfully mapped [{:#x}, {:#x}) with perm {:?}",
            start,
            start + len,
            map_perm
        );
        0
    } else {
        trace!("sys_mmap: failed to map, address range conflict");
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
