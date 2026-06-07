#![cfg(any(
    target_os = "windows",
))]
#![allow(dead_code)]

use core::sync::atomic::AtomicU32;
use std::time::Duration;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_TIMEOUT};
use windows_sys::Win32::System::{
    Threading::{WaitOnAddress, WakeByAddressAll, WakeByAddressSingle},
    WindowsProgramming::INFINITE,
};

#[inline]
pub fn wait(a: &AtomicU32, expected: u32) {
    let ptr: *const AtomicU32 = a;
    let expected_ptr: *const u32 = &expected;
    unsafe { WaitOnAddress(ptr.cast(), expected_ptr.cast(), size_of::<u32>(), INFINITE) };
}

#[inline]
pub fn wait_timeout(a: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    let ptr: *const AtomicU32 = a;
    let expected_ptr: *const u32 = &expected;
    let timeout = match timeout {
        Some(timeout) => timeout.as_millis() as _,
        None => INFINITE,
    };
    let success = unsafe { WaitOnAddress(ptr.cast(), expected_ptr.cast(), size_of::<u32>(), timeout) };
    success == 1 || unsafe { GetLastError() } != ERROR_TIMEOUT
}

#[inline]
pub fn wake_one(ptr: *const AtomicU32) {
    unsafe { WakeByAddressSingle(ptr.cast()) };
}

#[inline]
pub fn wake_all(ptr: *const AtomicU32) {
    unsafe { WakeByAddressAll(ptr.cast()) };
}
