use core::sync::atomic::AtomicU32;
use std::time::Duration;
use crate::unix_futex::*;

#[inline]
pub fn wait(a: &AtomicU32, expected: u32) {
    futex_wait(a, expected, None, false);
}

#[inline]
pub fn wait_timeout(a: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    futex_wait(a, expected, timeout, false)
}

#[inline]
pub fn wake_one(ptr: &AtomicU32) {
    futex_wake(ptr, false);
}

#[inline]
pub fn wake_all(ptr: &AtomicU32) {
    futex_wake_all(ptr, false);
}

#[inline]
pub fn wait_shared(a: &AtomicU32, expected: u32) {
    futex_wait(a, expected, None, true);
}

#[inline]
pub fn wait_timeout_shared(a: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    futex_wait(a, expected, timeout, true)
}

#[inline]
pub fn wake_one_shared(ptr: &AtomicU32) {
    futex_wake(ptr, true);
}

#[inline]
pub fn wake_all_shared(ptr: &AtomicU32) {
    futex_wake_all(ptr, true);
}
