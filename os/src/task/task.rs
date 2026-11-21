//! Types related to task management

use super::TaskContext;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// 用结构体存放调用次数
    pub  syscall_num:SysCallNum,
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

#[derive(Copy, Clone,Default)]
//这里我就用结构体存放不同编号的系统调用次数了
pub struct  SysCallNum{
    pub(super)syscall_write: u32,
    pub(super)syscall_exit: u32,
    pub(super)syscall_trace: u32,
    pub(super)syscall_gettime: u32,
    pub(super)syscall_yield: u32,
}
