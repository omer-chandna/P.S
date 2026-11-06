//! Run keyboard simulation on the main queue.
//!
//! HIToolbox aborts (`dispatch_assert_queue`) when enigo asks for the current
//! keyboard layout from any other thread. Copy and paste run on Tokio workers,
//! so the chord has to hop to the main queue, which the AppKit run loop drains.

use std::ffi::c_void;

#[link(name = "System")]
extern "C" {
    fn pthread_main_np() -> i32;
    fn dispatch_sync_f(queue: *mut c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
    static _dispatch_main_q: u8;
}

struct Job {
    work: Option<Box<dyn FnOnce() + Send>>,
}

extern "C" fn execute_job(context: *mut c_void) {
    let job = unsafe { &mut *context.cast::<Job>() };
    if let Some(work) = job.work.take() {
        work();
    }
}

fn main_queue() -> *mut c_void {
    std::ptr::addr_of!(_dispatch_main_q).cast_mut().cast()
}

pub fn is_main_thread() -> bool {
    unsafe { pthread_main_np() != 0 }
}

/// Runs `work` on the main queue and waits for it.
///
/// Safe to call from a worker while the main thread is inside the AppKit run
/// loop. Calling it from the main thread runs `work` inline; `dispatch_sync`
/// back onto the main queue would deadlock.
pub fn run<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    if is_main_thread() {
        return work();
    }

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let mut job = Job {
        work: Some(Box::new(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
            let _ = sender.send(outcome);
        })),
    };
    unsafe {
        dispatch_sync_f(
            main_queue(),
            &mut job as *mut Job as *mut c_void,
            execute_job,
        );
    }
    match receiver
        .recv()
        .expect("main queue dropped the keyboard task")
    {
        Ok(value) => value,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}
