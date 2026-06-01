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
    futex_wait(a, expected, None, false);
}

#[inline]
pub fn wait_shared(a: &AtomicU32, expected: u32) {
    futex_wait(a, expected, None, true);
}

#[inline]
pub fn wait_timeout(a: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    futex_wait(a, expected, timeout, false)
}

#[inline]
pub fn wait_timeout_shared(a: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    futex_wait(a, expected, timeout, true)
}

#[inline]
pub fn wake_one(ptr: &AtomicU32) {
    if !ptr.is_null() {
        futex_wake(unsafe { &*ptr }, false);
    }
}

#[inline]
pub fn wake_one_shared(ptr: &AtomicU32) {
    if !ptr.is_null() {
        futex_wake(unsafe { &*ptr }, true);
    }
}

#[inline]
pub fn wake_all(ptr: &AtomicU32) {
    if !ptr.is_null() {
        futex_wake_all(unsafe { &*ptr }, false);
    }
}

#[inline]
pub fn wake_all_shared(ptr: &AtomicU32) {
    if !ptr.is_null() {
        futex_wake_all(unsafe { &*ptr }, true);
    }
}
