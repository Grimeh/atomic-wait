#![cfg(any(
    target_os = "windows",
))]
#![allow(dead_code)]

use core::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64};
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
pub fn wait_u64(a: &AtomicU64, expected: u64) {
    let ptr: *const AtomicU64 = a;
    let expected_ptr: *const u64 = &expected;
    unsafe { WaitOnAddress(ptr.cast(), expected_ptr.cast(), size_of::<u64>(), INFINITE) };
}

#[inline]
pub fn wait_ptr<T>(a: &AtomicPtr<T>, expected: *const T) {
    let expected_ptr: *const *const T = &expected as _;
    unsafe { WaitOnAddress(a.as_ptr().cast(), expected_ptr.cast(), size_of::<*mut T>(), INFINITE )};
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
pub fn wait_u64_timeout(a: &AtomicU64, expected: u64, timeout: Option<Duration>) -> bool {
    let ptr: *const AtomicU64 = a;
    let expected_ptr: *const u64 = &expected;
    let timeout = match timeout {
        Some(timeout) => timeout.as_millis() as _,
        None => INFINITE,
    };
    let success = unsafe { WaitOnAddress(ptr.cast(), expected_ptr.cast(), size_of::<u64>(), timeout) };
    success == 1 || unsafe { GetLastError() } != ERROR_TIMEOUT
}

#[inline]
pub fn wait_ptr_timeout<T>(a: &AtomicPtr<T>, expected: *mut T, timeout: Option<Duration>) -> bool {
    let expected_ptr: *const *const T = &expected as *const *mut T as *const *const T;
    let timeout = match timeout {
        Some(timeout) => timeout.as_millis() as _,
        None => INFINITE,
    };
    let success = unsafe { WaitOnAddress(a.as_ptr().cast(), expected_ptr.cast(), size_of::<*mut T>(), timeout) };
    success == 1 || unsafe { GetLastError() } != ERROR_TIMEOUT
}

#[inline]
pub fn wake_one(ptr: *const AtomicU32) {
    unsafe { WakeByAddressSingle(ptr.cast()) };
}

#[inline]
pub fn wake_one_u64(ptr: *const AtomicU64) {
    unsafe { WakeByAddressSingle(ptr.cast()) };
}

#[inline]
pub fn wake_one_ptr<T>(ptr: *const AtomicPtr<T>) {
    unsafe { WakeByAddressSingle(ptr.cast()) };
}

#[inline]
pub fn wake_all(ptr: *const AtomicU32) {
    unsafe { WakeByAddressAll(ptr.cast()) };
}

#[inline]
pub fn wake_all_u64(ptr: *const AtomicU64) {
    unsafe { WakeByAddressAll(ptr.cast()) };
}

#[inline]
pub fn wake_all_ptr<T>(ptr: *const AtomicPtr<T>) {
    unsafe { WakeByAddressAll(ptr.cast()) };
}
