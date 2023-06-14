use crate::arch::interrupt::StackFrame;
use crate::process::thread::{self, Thread, ThreadId};
use crate::util::locked::Locked;
use buf::ring::RingBuf;
use core::ptr::NonNull;

const MAX_SCHEDULED_PROCS: usize = 256;

struct Scheduler {
    //run_queue: RingBuf<*mut Thread, MAX_SCHEDULED_PROCS>,
    running_thread: *mut Thread,
}

unsafe impl Send for Scheduler {}

impl Scheduler {
    fn push(&mut self, thread: NonNull<Thread>) {
        self.running_thread = thread.as_ptr();
    }

    fn advance(&mut self) -> Option<NonNull<Thread>> {
        NonNull::new(self.running_thread)
    }
}

// TODO: Replace with a circular buffer.
static SCHEDULER: Locked<Scheduler> = Locked::new(Scheduler {
    /*run_queue: todo!()*/ running_thread: core::ptr::null_mut(),
});

pub fn schedule(thread: NonNull<Thread>) {
    debug!("scheduling thread '{:?}'", unsafe { thread.as_ref() });
    SCHEDULER.lock().push(thread);
}

pub fn deschedule(_thread_id: ThreadId) {
    unimplemented!()
}

pub unsafe fn step(stackframe: *mut StackFrame) {
    debug!("scheduler: stepping");
    let mut scheduler = SCHEDULER.lock();
    let mut cur_thread_ptr = thread::cur_thread();
    let cur_thread = unsafe { cur_thread_ptr.as_mut() };
    {
        let mut lock = cur_thread.lock();
        lock.set_stackframe(stackframe.read());
    }
    //scheduler.push(cur_thread_ptr);
    match scheduler.advance() {
        Some(thread_ptr) => {
            let thread = thread_ptr.as_ref();
            let lock = thread.lock();
            debug!("scheduler: running thread '{lock:?}'");
            stackframe.write(lock.get_stackframe());
            thread::set_thread(thread_ptr);
        }
        _ => {
            warn!("scheduler: empty")
        }
    }
}
