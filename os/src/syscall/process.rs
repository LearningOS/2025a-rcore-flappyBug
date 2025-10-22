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
/// HINT: What if [`TimeVal`] is splitted by two pages ?
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

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
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
