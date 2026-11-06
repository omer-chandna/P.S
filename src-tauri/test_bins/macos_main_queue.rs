//! `TSMGetInputSourceProperty` aborts unless it runs on the main queue.
//! This process is the main thread: it drains that queue while a worker asks
//! for the current keyboard layout through the same hop copy/paste uses.

use std::ffi::c_void;
use std::sync::mpsc::{self, TryRecvError};
use std::time::{Duration, Instant};

fn main() {
    #[cfg(target_os = "macos")]
    macos();
}

#[cfg(target_os = "macos")]
fn macos() {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let on_main = ps_desktop_lib::test_support::run_on_main_thread(|| {
            let readable = layout_property_is_readable();
            (main_thread(), readable)
        });
        let _ = sender.send(on_main);
    });

    let started = Instant::now();
    loop {
        match receiver.try_recv() {
            Ok((on_main, readable)) => {
                assert!(on_main, "layout lookup did not run on the main thread");
                assert!(
                    readable,
                    "the current keyboard layout did not expose key data"
                );
                return;
            }
            Err(TryRecvError::Empty) => {
                if started.elapsed() > Duration::from_secs(3) {
                    panic!("timed out waiting for the main queue to run the layout lookup");
                }
                unsafe {
                    CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.05, 0);
                }
            }
            Err(TryRecvError::Disconnected) => {
                panic!("layout lookup stopped before it could report");
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn main_thread() -> bool {
    unsafe { pthread_main_np() != 0 }
}

#[cfg(target_os = "macos")]
fn layout_property_is_readable() -> bool {
    unsafe {
        let source = TISCopyCurrentKeyboardInputSource();
        if source.is_null() {
            return false;
        }
        let data = TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData);
        CFRelease(source);
        !data.is_null()
    }
}

#[cfg(target_os = "macos")]
#[link(name = "System")]
extern "C" {
    fn pthread_main_np() -> i32;
}

#[cfg(target_os = "macos")]
#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn TISCopyCurrentKeyboardInputSource() -> *mut c_void;
    fn TISGetInputSourceProperty(source: *mut c_void, key: *const c_void) -> *mut c_void;
    static kTISPropertyUnicodeKeyLayoutData: *const c_void;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFRunLoopDefaultMode: *const c_void;
    fn CFRunLoopRunInMode(
        mode: *const c_void,
        seconds: f64,
        return_after_source_handled: u8,
    ) -> i32;
    fn CFRelease(value: *const c_void);
}
