#![doc = include_str!("../README.md")]
#![feature(macro_metavar_expr_concat)]
#![feature(io_const_error)]
#![feature(io_error_uncategorized)]
#![feature(cfg_select)]
#![feature(temporary_niche_types)]
#![feature(panic_internals)]
#![feature(generic_atomic)]

use core::sync::atomic::AtomicU32;
use std::time::Duration;

mod unix_futex;

#[cfg(any(target_os = "linux", target_os = "android"))]
#[path = "linux.rs"]
mod platform;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "watchos"))]
#[path = "macos.rs"]
mod platform;

#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

#[cfg(target_os = "freebsd")]
#[path = "freebsd.rs"]
mod platform;

/// If the value is `value`, wait until woken up.
///
/// This function might also return spuriously,
/// without a corresponding wake operation.
#[inline]
pub fn wait(atomic: &AtomicU32, value: u32) {
    platform::wait(atomic, value)
}

/// If the value is `value`, wait until woken up.
///
/// This function might also return spuriously,
/// without a corresponding wake operation.
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wait_shared(atomic: &AtomicU32, value: u32) {
    platform::wait_shared(atomic, value)
}

/// If the value is `value`, wait until woken up.
///
/// This function might also return spuriously,
/// without a corresponding wake operation.
///
/// Returns false if the timeout expired
#[inline]
pub fn wait_timeout(atomic: &AtomicU32, value: u32, timeout: Option<Duration>) -> bool {
    platform::wait_timeout(atomic, value, timeout)
}

/// If the value is `value`, wait until woken up.
///
/// This function might also return spuriously,
/// without a corresponding wake operation.
///
/// Returns false if the timeout expired
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wait_timeout_shared(atomic: &AtomicU32, value: u32, timeout: Option<Duration>) -> bool {
    platform::wait_timeout_shared(atomic, value, timeout)
}

/// Wake one thread that is waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[inline]
pub fn wake_one(atomic: &AtomicU32) {
    platform::wake_one(atomic);
}

/// Wake one thread that is waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wake_one_shared(atomic: &AtomicU32) {
    platform::wake_one_shared(atomic);
}

/// Wake all threads that are waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[inline]
pub fn wake_all(atomic: &AtomicU32) {
    platform::wake_all(atomic);
}

/// Wake all threads that are waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wake_all_shared(atomic: &AtomicU32) {
    platform::wake_all_shared(atomic);
}
