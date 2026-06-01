// adapted from https://github.com/rust-lang/rust/blob/76b3461966abc36e22172a30358e8d00ac989dd2/library/std/src/sys/pal/unix/futex.rs
// from PR https://github.com/rust-lang/rust/pull/122408
// this pending draft PR adds futex support to apple platforms, by using the newly exposed `os_sync_*` functions for
// macOS 14.4+, with a fallback to private (& undocumented) `__ulock*` functions for older versions.

#![cfg(any(
    target_vendor = "apple",
))]
#![allow(dead_code)]

#[macro_use]
mod unix_weak;
mod errno;
mod util;

use errno::errno;
use core::sync::atomic::AtomicU32;
use core::time::Duration;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, AtomicU64};

/// With macOS version 14.4, Apple introduced a public futex API. Unfortunately,
/// our minimum supported version is 10.12, so we need a fallback API. Luckily
/// for us, the underlying syscalls have been available since exactly that
/// version, so we just use those when needed. This is private API however,
/// which means we need to take care to avoid breakage if the syscall is removed
/// and to avoid apps being rejected from the App Store. To do this, we use weak
/// linkage emulation for both the public and the private API.
///
/// See os/os_sync_wait_on_address.h (Apple has failed to upload the documentation
/// to their website) for documentation of the public API and
/// https://github.com/apple-oss-distributions/xnu/blob/1031c584a5e37aff177559b9f69dbd3c8c3fd30a/bsd/sys/ulock.h#L69
/// for the header file of the private API, along with its usage in libpthread
/// https://github.com/apple-oss-distributions/libpthread/blob/d8c4e3c212553d3e0f5d76bb7d45a8acd61302dc/src/pthread_cond.c#L463
#[cfg(target_vendor = "apple")]
mod apple {
    use super::unix_weak::weak;
    use core::ffi::{c_int, c_void};

    pub const OS_CLOCK_MACH_ABSOLUTE_TIME: u32 = 32;
    pub const OS_SYNC_WAIT_ON_ADDRESS_NONE: u32 = 0;
    pub const OS_SYNC_WAIT_ON_ADDRESS_SHARED: u32 = 1;
    pub const OS_SYNC_WAKE_BY_ADDRESS_NONE: u32 = 0;
    pub const OS_SYNC_WAKE_BY_ADDRESS_SHARED: u32 = 1;

    pub const UL_COMPARE_AND_WAIT: u32 = 1;
    pub const UL_COMPARE_AND_WAIT_SHARED: u32 = 3;
    pub const UL_COMPARE_AND_WAIT64: u32 = 5;
    pub const UL_COMPARE_AND_WAIT64_SHARED: u32 = 6;
    pub const ULF_WAKE_ALL: u32 = 0x100;
    // The syscalls support directly returning errors instead of going through errno.
    pub const ULF_NO_ERRNO: u32 = 0x1000000;

    // These functions appeared with macOS 14.4, iOS 17.4, tvOS 17.4, watchOS 10.4, visionOS 1.1.
    weak! {
        pub fn os_sync_wait_on_address(*mut c_void, u64, usize, u32) -> c_int
    }

    weak! {
        pub fn os_sync_wait_on_address_with_timeout(*mut c_void, u64, usize, u32, u32, u64) -> c_int
    }

    weak! {
        pub fn os_sync_wake_by_address_any(*mut c_void, usize, u32) -> c_int
    }

    weak! {
        pub fn os_sync_wake_by_address_all(*mut c_void, usize, u32) -> c_int
    }

    // This syscall appeared with macOS 11.0.
    // It is used to support nanosecond precision for timeouts, among other features.
    weak! {
        pub fn __ulock_wait2(u32, *mut c_void, u64, u64, u64) -> c_int
    }

    // These syscalls appeared with macOS 10.12.
    weak! {
        pub fn __ulock_wait(u32, *mut c_void, u64, u32) -> c_int
    }

    weak! {
        pub fn __ulock_wake(u32, *mut c_void, u64) -> c_int
    }

    pub(crate) fn wait_operation_for(size: usize, shared: bool) -> u32 {
        if shared {
            match size {
                4 => UL_COMPARE_AND_WAIT_SHARED,
                8 => UL_COMPARE_AND_WAIT64_SHARED,
                _ => panic!("invalid futex size of {}", size),
            }
        } else {
            match size {
                4 => UL_COMPARE_AND_WAIT,
                8 => UL_COMPARE_AND_WAIT64,
                _ => panic!("invalid futex size of {}", size),
            }
        }
    }
}

#[cfg(target_vendor = "apple")]
fn futex_wait_inner(addr: *mut c_void, size: usize, expected: u64, timeout: Option<Duration>, shared: bool) -> bool {
    use apple::*;

    if let Some(timeout) = timeout {
        let timeout_ns = timeout.as_nanos().clamp(1, u64::MAX as u128) as u64;
        let timeout_ms = timeout.as_micros().clamp(1, u32::MAX as u128) as u32;

        if let Some(wait) = os_sync_wait_on_address_with_timeout.get() {
            let flags = if shared {
                OS_SYNC_WAIT_ON_ADDRESS_SHARED
            } else {
                OS_SYNC_WAIT_ON_ADDRESS_NONE
            };

            let r = unsafe {
                wait(
                    addr,
                    expected,
                    size,
                    flags,
                    OS_CLOCK_MACH_ABSOLUTE_TIME,
                    timeout_ns,
                )
            };

            // We promote spurious wakeups (reported as EINTR) to normal ones for
            // simplicity and because Apple's documentation is unclear as to what the
            // unit for the deadline of `os_sync_wait_on_address_with_deadline` is,
            // making it hard to support timeouts in a fashion similar to the Linux
            // futex implementation.
            r != -1 || errno() != libc::ETIMEDOUT
        } else if let Some(wait) = __ulock_wait2.get() {
            let operation = wait_operation_for(size, shared);

            unsafe {
                wait(
                    operation | ULF_NO_ERRNO,
                    addr,
                    expected,
                    timeout_ns,
                    0,
                ) != -libc::ETIMEDOUT
            }
        } else if let Some(wait) = __ulock_wait.get() {
            let operation = wait_operation_for(size, shared);

            unsafe {
                wait(operation | ULF_NO_ERRNO, addr, expected, timeout_ms)
                    != -libc::ETIMEDOUT
            }
        } else {
            panic!("your system is below the minimum supported version of Rust");
        }
    } else {
        if let Some(wait) = os_sync_wait_on_address.get() {
            let flags = if shared {
                OS_SYNC_WAIT_ON_ADDRESS_SHARED
            } else {
                OS_SYNC_WAIT_ON_ADDRESS_NONE
            };

            unsafe {
                wait(addr, expected, size, flags);
            }
        } else if let Some(wait) = __ulock_wait.get() {
            let flags = if shared {
                OS_SYNC_WAIT_ON_ADDRESS_SHARED
            } else {
                OS_SYNC_WAIT_ON_ADDRESS_NONE
            };

            unsafe {
                wait(flags | ULF_NO_ERRNO, addr, expected, 0);
            }
        } else {
            panic!("your system is below the minimum supported version of Rust");
        }
        true
    }
}

#[cfg(target_vendor = "apple")]
pub fn futex_wait(futex: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    let value = expected as u64;
    let size = size_of::<u32>();
    futex_wait_inner(addr, size, value, timeout, false)
}

#[cfg(target_vendor = "apple")]
pub fn futex_wait_shared(futex: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    let value = expected as u64;
    let size = size_of::<u32>();
    futex_wait_inner(addr, size, value, timeout, true)
}

#[cfg(target_vendor = "apple")]
pub fn futex_wait_u64(futex: &AtomicU64, expected: u64, timeout: Option<Duration>) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    let size = size_of::<u64>();
    futex_wait_inner(addr, size, expected, timeout, false)
}

#[cfg(target_vendor = "apple")]
pub fn futex_wait_u64_shared(futex: &AtomicU64, expected: u64, timeout: Option<Duration>) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    let size = size_of::<u64>();
    futex_wait_inner(addr, size, expected, timeout, true)
}

#[cfg(target_vendor = "apple")]
pub fn futex_wait_ptr<T>(futex: &AtomicPtr<T>, expected: *mut T, timeout: Option<Duration>) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    let size = size_of::<*mut T>();
    let value = expected as u64;
    futex_wait_inner(addr, size, value, timeout, false)
}

#[cfg(target_vendor = "apple")]
pub fn futex_wait_ptr_shared<T>(futex: &AtomicPtr<T>, expected: *mut T, timeout: Option<Duration>) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    let size = size_of::<*mut T>();
    let value = expected as u64;
    futex_wait_inner(addr, size, value, timeout, true)
}

#[cfg(any(target_vendor = "apple"))]
fn futex_wake_inner(addr: *mut c_void, size: usize, shared: bool) -> bool {
    use apple::*;

    if let Some(wake) = os_sync_wake_by_address_any.get() {
        let flags = if shared {
            OS_SYNC_WAKE_BY_ADDRESS_SHARED
        } else {
            OS_SYNC_WAKE_BY_ADDRESS_NONE
        };

        unsafe { wake(addr, size, flags) == 0 }
    } else if let Some(wake) = __ulock_wake.get() {
        // __ulock_wake can get interrupted, so retry until either waking up a
        // waiter or failing because there are no waiters (ENOENT).
        loop {
            let operation = wait_operation_for(size, shared);
            let r = unsafe { wake(operation | ULF_NO_ERRNO, addr, 0) };

            if r >= 0 {
                return true;
            } else {
                match -r {
                    libc::ENOENT => return false,
                    libc::EINTR => continue,
                    err => panic!(
                        "__ulock_wake failed: {}",
                        std::io::Error::from_raw_os_error(err)
                    ),
                }
            }
        }
    } else {
        panic!("your system is below the minimum supported version of Rust");
    }
}

#[cfg(any(target_vendor = "apple"))]
pub fn futex_wake(futex: &AtomicU32) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_inner(addr, size_of::<u32>(), false)
}

#[cfg(any(target_vendor = "apple"))]
pub fn futex_wake_shared(futex: &AtomicU32) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_inner(addr, size_of::<u32>(), true)
}

#[cfg(any(target_vendor = "apple"))]
pub fn futex_wake_u64(futex: &AtomicU64) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_inner(addr, size_of::<u64>(), false)
}

#[cfg(any(target_vendor = "apple"))]
pub fn futex_wake_u64_shared(futex: &AtomicU64) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_inner(addr, size_of::<u64>(), true)
}

#[cfg(any(target_vendor = "apple"))]
pub fn futex_wake_ptr<T>(futex: &AtomicPtr<T>) -> bool {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_inner(addr, size_of::<*mut T>(), false)
}

#[cfg(target_vendor = "apple")]
fn futex_wake_all_inner(addr: *mut c_void, size: usize, shared: bool) {
    use apple::*;
    use std::io::Error;

    if let Some(wake) = os_sync_wake_by_address_all.get() {
        let flags = if shared {
            OS_SYNC_WAKE_BY_ADDRESS_SHARED
        } else {
            OS_SYNC_WAKE_BY_ADDRESS_NONE
        };

        unsafe {
            wake(addr, size, flags);
        }
    } else if let Some(wake) = __ulock_wake.get() {
        loop {
            let operation = wait_operation_for(size, shared);
            let r = unsafe { wake(operation | ULF_WAKE_ALL | ULF_NO_ERRNO, addr, 0) };

            if r >= 0 {
                return;
            } else {
                match -r {
                    libc::ENOENT => return,
                    libc::EINTR => continue,
                    err => panic!("__ulock_wake failed: {}", Error::from_raw_os_error(err)),
                }
            }
        }
    } else {
        panic!("your system is below the minimum supported version of Rust");
    }
}

#[cfg(target_vendor = "apple")]
pub fn futex_wake_all(futex: &AtomicU32) {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_all_inner(addr, size_of::<u32>(), false);
}

#[cfg(target_vendor = "apple")]
pub fn futex_wake_all_shared(futex: &AtomicU32) {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_all_inner(addr, size_of::<u32>(), true);
}

#[cfg(target_vendor = "apple")]
pub fn futex_wake_all_u64(futex: &AtomicU64) {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_all_inner(addr, size_of::<u64>(), false);
}

#[cfg(target_vendor = "apple")]
pub fn futex_wake_all_u64_shared(futex: &AtomicU64) {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_all_inner(addr, size_of::<u64>(), true);
}

#[cfg(target_vendor = "apple")]
pub fn futex_wake_all_ptr<T>(futex: &AtomicPtr<T>) {
    use core::mem::size_of;

    let addr = futex.as_ptr().cast();
    futex_wake_all_inner(addr, size_of::<*mut T>(), false);
}
