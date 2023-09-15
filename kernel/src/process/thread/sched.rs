use crate::arch::interrupt::StackFrame;
use crate::process::thread::{self, Thread, ThreadPtr, ThreadStatus};
use crate::util::locked::Locked;
use buf::ring::RingBuf;
use thread::ThreadScheduleStatus;

const MAX_SCHEDULED_THREADS: usize = 256;

struct Scheduler {
    run_queue: RingBuf<ThreadPtr, MAX_SCHEDULED_THREADS>,
}

unsafe impl Send for Scheduler {}

impl Scheduler {
    /// return true if thread was sucessfully scheduled.
    fn push(&mut self, thread: ThreadPtr) -> bool {
        self.run_queue.push(thread).is_ok()
    }

    fn advance(&mut self) -> Option<ThreadPtr> {
        self.run_queue.pop().ok()
    }
}

// TODO: Replace with a circular buffer.
static SCHEDULER: Locked<Scheduler> = Locked::new(Scheduler {
    run_queue: RingBuf::new(),
});

/// return `true` if the thread was successfully scheduled.
pub fn schedule(thread: ThreadPtr) -> bool {
    trace!("scheduling thread: {}", unsafe { thread.get().get_id() });
    SCHEDULER.lock().push(thread)
}

pub unsafe fn step(stackframe: *mut StackFrame) {
    let mut scheduler = SCHEDULER.lock();
    let cur_thread_ptr = thread::cur_thread();
    {
        let mut lock = cur_thread_ptr.get_locked();
        lock.set_stackframe(stackframe.read());
        let sched_status = match lock.get_schedule_status() {
            ThreadScheduleStatus::Sleep => ThreadStatus::Sleeping,
            ThreadScheduleStatus::Running => {
                scheduler.push(cur_thread_ptr);
                ThreadStatus::Waiting
            }
        };
        lock.set_status(sched_status);
    }
    match scheduler.advance() {
        Some(thread_ptr) => {
            let mut lock = thread_ptr.get_locked();
            stackframe.write(lock.get_stackframe().clone());
            lock.set_status(ThreadStatus::Running);
            Thread::make_thread_current(&mut lock);
        }
        _ => {
            warn!("scheduler: empty")
        }
    }
}
