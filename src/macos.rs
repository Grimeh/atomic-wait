use core::{
    sync::atomic::{AtomicU32, AtomicPtr, AtomicU64},
};
use std::time::Duration;
use crate::unix_futex::*;

// UPDATE 01/12/24
// There is a pending PR for adding proper futex functionality to macOS, including support for timeouts. In order to add
// timeout variants to the API the impl has been copied & adapted until the PR lands (or is reworked, whatever the case 
// may be). Technically it uses undocumented functions as a fallback and could cause issues if this is ever submitted to 
// Apple's review for macOS/iOS/whatever, so you may not want to use this branch.
// see PR status here: https://github.com/rust-lang/rust/pull/122408

// OUTDATED:
// On macOS, atomic wait/wake functionality is not available through
// any public/stable C interface, but is available through libc++.
//
// The libc++ functions declared below are are not publicly documented,
// but they are part of the stable ABI.
//
// These functions are used to implement C++20's std::atomic<T>::{wait,
// notify*} which are defined in libc++'s headers, resulting in C++ binaries
// that dynamically link these symbols. So, it's safe to rely on these from
// Rust as well, as long as we also link libc++.
//
// These exist since macOS 11, iOS 14, and watchOS 7.

#[inline]
pub fn wait(a: &AtomicU32, expected: u32) {
    futex_wait(a, expected, None);
}

#[inline]
pub fn wait_u64(a: &AtomicU64, expected: u64) {
    futex_wait_u64(a, expected, None);
}

#[inline]
pub fn wait_ptr<T>(a: &AtomicPtr<T>, expected: *mut T) {
    futex_wait_ptr(a, expected, None);
}

#[inline]
pub fn wait_timeout(a: &AtomicU32, expected: u32, timeout: Option<Duration>) {
    futex_wait(a, expected, timeout);
}

#[inline]
pub fn wait_u64_timeout(a: &AtomicU64, expected: u64, timeout: Option<Duration>) {
    futex_wait_u64(a, expected, timeout);
}

#[inline]
pub fn wait_ptr_timeout<T>(a: &AtomicPtr<T>, expected: *mut T, timeout: Option<Duration>) {
    futex_wait_ptr(a, expected, timeout);
}

#[inline]
pub fn wake_one(ptr: *const AtomicU32) {
    if !ptr.is_null() {
        futex_wake(unsafe { &*ptr });
    }
}

#[inline]
pub fn wake_all(ptr: *const AtomicU32) {
    if !ptr.is_null() {
        futex_wake_all(unsafe { &*ptr });
    }
}

#[inline]
pub fn wake_one_u64(ptr: *const AtomicU64) {
    if !ptr.is_null() {
        futex_wake_u64(unsafe { &*ptr });
    }
}

#[inline]
pub fn wake_one_ptr<T>(ptr: *const AtomicPtr<T>) {
    if !ptr.is_null() {
        futex_wake_ptr(unsafe { &*ptr });
    }
}

#[inline]
pub fn wake_all_u64(ptr: *const AtomicU64) {
    if !ptr.is_null() {
        futex_wake_all_u64(unsafe { &*ptr });
    }
}

#[inline]
pub fn wake_all_ptr<T>(ptr: *const AtomicPtr<T>) {
    if !ptr.is_null() {
        futex_wake_all_ptr(unsafe { &*ptr });
    }
}
