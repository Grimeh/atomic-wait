#![doc = include_str!("../README.md")]

use core::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64};
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
#[inline]
pub fn wait_u64(atomic: &AtomicU64, value: u64) {
    platform::wait_u64(atomic, value)
}

/// If the value is `value`, wait until woken up.
///
/// This function might also return spuriously,
/// without a corresponding wake operation.
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wait_u64_shared(atomic: &AtomicU64, value: u64) {
    platform::wait_u64_shared(atomic, value)
}

/// If the value is `value`, wait until woken up.
///
/// This function might also return spuriously,
/// without a corresponding wake operation.
#[cfg(feature = "ptr")]
pub fn wait_ptr<T>(atomic: &AtomicPtr<T>, value: *mut T) {
    platform::wait_ptr(atomic, value)
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

/// If the value is `value`, wait until woken up.
///
/// This function might also return spuriously,
/// without a corresponding wake operation.
///
/// Returns false if the timeout expired
#[inline]
pub fn wait_u64_timeout(atomic: &AtomicU64, value: u64, timeout: Option<Duration>) -> bool {
    platform::wait_u64_timeout(atomic, value, timeout)
}

/// If the value is `value`, wait until woken up.
///
/// This function might also return spuriously,
/// without a corresponding wake operation.
///
/// Returns false if the timeout expired
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wait_u64_timeout_shared(atomic: &AtomicU64, value: u64, timeout: Option<Duration>) -> bool {
    platform::wait_u64_timeout_shared(atomic, value, timeout)
}

/// If the value is `value`, wait until woken up.
///
/// This function might also return spuriously,
/// without a corresponding wake operation.
///
/// Returns false if the timeout expired
#[cfg(feature = "ptr")]
pub fn wait_ptr_timeout<T>(atomic: &AtomicPtr<T>, value: *mut T, timeout: Option<Duration>) -> bool {
    platform::wait_ptr_timeout(atomic, value, timeout)
}

/// Wake one thread that is waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[inline]
pub fn wake_one(atomic: *const AtomicU32) {
    platform::wake_one(atomic);
}

/// Wake one thread that is waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wake_one_shared(atomic: *const AtomicU32) {
    platform::wake_one_shared(atomic);
}

/// Wake one thread that is waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[inline]
pub fn wake_one_u64(atomic: *const AtomicU64) {
    platform::wake_one_u64(atomic);
}

/// Wake one thread that is waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wake_one_u64_shared(atomic: *const AtomicU64) {
    platform::wake_one_u64_shared(atomic);
}

/// Wake one thread that is waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[inline]
#[cfg(feature = "ptr")]
pub fn wake_one_ptr<T>(atomic: *const AtomicPtr<T>) {
    const {
        // assert we're not trying to use a fat pointer
        let size = size_of::<*mut T>();
        assert!(size == 8, "Atomic pointers for DSTs are not supported");
    }
    platform::wake_one_ptr(atomic);
}

/// Wake all threads that are waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[inline]
pub fn wake_all(atomic: *const AtomicU32) {
    platform::wake_all(atomic);
}

/// Wake all threads that are waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wake_all_shared(atomic: *const AtomicU32) {
    platform::wake_all_shared(atomic);
}

/// Wake all threads that are waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[inline]
pub fn wake_all_u64(atomic: *const AtomicU64) {
    platform::wake_all_u64(atomic);
}

/// Wake all threads that are waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[cfg(not(target_os = "windows"))]
#[inline]
pub fn wake_all_u64_shared(atomic: *const AtomicU64) {
    platform::wake_all_u64_shared(atomic);
}

/// Wake all threads that are waiting on this atomic.
///
/// It's okay if the pointer dangles or is null.
#[cfg(feature = "ptr")]
pub fn wake_all_ptr<T>(atomic: *const AtomicPtr<T>) {
    const {
        // assert we're not trying to use a fat pointer
        let size = size_of::<*mut T>();
        assert!(size == 8, "Atomic pointers for DSTs are not supported");
    }
    platform::wake_all_ptr(atomic);
}
