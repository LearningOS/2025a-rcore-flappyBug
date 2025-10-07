//! Types related to task management

use super::TaskContext;

/// The task control block (TCB) of a task.
#[derive(Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The syscall count
    pub syscall_count: SyscallCount,
}

/// The syscall count of a task.
#[derive(Clone, Default)]
pub struct SyscallCount {
    pub write: usize,
    pub exit: usize,
    pub yield_: usize,
    pub get_time: usize,
    pub trace: usize,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
