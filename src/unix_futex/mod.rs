// adapted from https://github.com/rust-lang/rust/blob/76b3461966abc36e22172a30358e8d00ac989dd2/library/std/src/sys/pal/unix/futex.rs
// from PR https://github.com/rust-lang/rust/pull/122408
// this pending draft PR adds futex support to apple platforms, by using the newly exposed `os_sync_*` functions for
// macOS 14.4+, with a fallback to private (& undocumented) `__ulock*` functions for older versions.

#![cfg(any(
    target_os = "linux",
    target_vendor = "apple",
    target_os = "android",
    all(target_os = "emscripten", target_feature = "atomics"),
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "dragonfly",
    target_os = "fuchsia",
))]

use std::sync::atomic::Atomic;
use std::time::Duration;

#[macro_use]
mod unix_weak;
mod errno;
mod util;
mod time;
mod cvt;

/// An atomic for use as a futex that is at least 32-bits but may be larger
pub type Futex = Atomic<Primitive>;
/// Must be the underlying type of Futex
pub type Primitive = u32;

/// An atomic for use as a futex that is at least 8-bits but may be larger.
pub type SmallFutex = Atomic<SmallPrimitive>;
/// Must be the underlying type of SmallFutex
pub type SmallPrimitive = u32;

/// Waits for a `futex_wake` operation to wake us.
///
/// Returns directly if the futex doesn't hold the expected value.
///
/// Returns false on timeout, and true in all other cases.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn futex_wait(futex: &Atomic<u32>, expected: u32, timeout: Option<Duration>, shared: bool) -> bool {
    use time::Timespec;
    use std::ptr::null;
    use std::sync::atomic::Ordering::Relaxed;

    // Calculate the timeout as an absolute timespec.
    //
    // Overflows are rounded up to an infinite timeout (None).
    let timespec = timeout
        .and_then(|d| Timespec::now(libc::CLOCK_MONOTONIC).checked_add_duration(&d))
        .and_then(|t| t.to_timespec());

    let flags = if shared { 0 } else { libc::FUTEX_PRIVATE_FLAG };

    loop {
        // No need to wait if the value already changed.
        if futex.load(Relaxed) != expected {
            return true;
        }

        let r = unsafe {
            cfg_select! {
                any(target_os = "linux", target_os = "android") => {
                    // Use FUTEX_WAIT_BITSET rather than FUTEX_WAIT to be able to give an
                    // absolute time rather than a relative time.
                    libc::syscall(
                        libc::SYS_futex,
                        futex as *const Atomic<u32>,
                        libc::FUTEX_WAIT_BITSET | flags,
                        expected,
                        timespec.as_ref().map_or(null(), |t| t as *const libc::timespec),
                        null::<u32>(), // This argument is unused for FUTEX_WAIT_BITSET.
                        !0u32,         // A full bitmask, to make it behave like a regular FUTEX_WAIT.
                    )
                }
                _ => {
                    compile_error!("unknown target_os");
                }
            }
        };

        match (r < 0).then(errno::errno) {
            Some(libc::ETIMEDOUT) => return false,
            Some(libc::EINTR) => continue,
            _ => return true,
        }
    }
}

/// Wakes up one thread that's blocked on `futex_wait` on this futex.
///
/// Returns true if this actually woke up such a thread,
/// or false if no thread was waiting on this futex.
///
/// On some platforms, this always returns false.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn futex_wake(futex: &Atomic<u32>, shared: bool) -> bool {
    let ptr = futex as *const Atomic<u32>;
    let mut op = libc::FUTEX_WAKE;
    if !shared {
        op |= libc::FUTEX_PRIVATE_FLAG;
    }
    unsafe { libc::syscall(libc::SYS_futex, ptr, op, 1) > 0 }
}

/// Wakes up all threads that are waiting on `futex_wait` on this futex.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn futex_wake_all(futex: &Atomic<u32>, shared: bool) {
    let ptr = futex as *const Atomic<u32>;
    let mut op = libc::FUTEX_WAKE;
    if !shared {
        op |= libc::FUTEX_PRIVATE_FLAG;
    }
    unsafe {
        libc::syscall(libc::SYS_futex, ptr, op, i32::MAX);
    }
}

/// With macOS version 14.4, Apple introduced a public futex API. Unfortunately,
/// our minimum supported version is 10.12, so we need a fallback API. Luckily
/// for us, the underlying syscalls have been available since exactly that
/// version, so we just use those when needed. This is private API however,
/// which means we need to take care to avoid breakage if the syscall is removed
/// and to avoid apps being rejected from the App Store. To do this, we use weak
/// linkage emulation for both the public and the private API. Experiments
/// indicate that this way of referencing private symbols is not flagged by the
/// App Store checks, see
/// https://github.com/rust-lang/rust/pull/122408#issuecomment-3403989895
///
/// See https://developer.apple.com/documentation/os/os_sync_wait_on_address?language=objc
/// for documentation of the public API and
/// https://github.com/apple-oss-distributions/xnu/blob/1031c584a5e37aff177559b9f69dbd3c8c3fd30a/bsd/sys/ulock.h#L69
/// for the header file of the private API, along with its usage in libpthread
/// https://github.com/apple-oss-distributions/libpthread/blob/d8c4e3c212553d3e0f5d76bb7d45a8acd61302dc/src/pthread_cond.c#L463
#[cfg(target_vendor = "apple")]
mod apple {
    use std::ffi::{c_int, c_void};
    use super::unix_weak::weak;

    pub const OS_CLOCK_MACH_ABSOLUTE_TIME: u32 = 32;
    pub const OS_SYNC_WAIT_ON_ADDRESS_NONE: u32 = 0;
    pub const OS_SYNC_WAIT_ON_ADDRESS_SHARED: u32 = 1;
    pub const OS_SYNC_WAKE_BY_ADDRESS_NONE: u32 = 0;
    pub const OS_SYNC_WAKE_BY_ADDRESS_SHARED: u32 = 1;

    pub const UL_COMPARE_AND_WAIT: u32 = 1;
    pub const UL_COMPARE_AND_WAIT_SHARED: u32 = 3;
    pub const ULF_WAKE_ALL: u32 = 0x100;
    // The syscalls support directly returning errors instead of going through errno.
    pub const ULF_NO_ERRNO: u32 = 0x1000000;

    // These functions appeared with macOS 14.4, iOS 17.4, tvOS 17.4, watchOS 10.4, visionOS 1.1.
    weak! {
        pub fn os_sync_wait_on_address(addr: *mut c_void, value: u64, size: usize, flags: u32) -> c_int;
    }

    weak! {
        pub fn os_sync_wait_on_address_with_timeout(addr: *mut c_void, value: u64, size: usize, flags: u32, clockid: u32, timeout_ns: u64) -> c_int;
    }

    weak! {
        pub fn os_sync_wake_by_address_any(addr: *mut c_void, size: usize, flags: u32) -> c_int;
    }

    weak! {
        pub fn os_sync_wake_by_address_all(addr: *mut c_void, size: usize, flags: u32) -> c_int;
    }

    // This syscall appeared with macOS 11.0.
    // It is used to support nanosecond precision for timeouts, among other features.
    weak! {
        pub fn __ulock_wait2(operation: u32, addr: *mut c_void, value: u64, timeout: u64, value2: u64) -> c_int;
    }

    // These syscalls appeared with macOS 10.12.
    weak! {
        pub fn __ulock_wait(operation: u32, addr: *mut c_void, value: u64, timeout: u32) -> c_int;
    }

    weak! {
        pub fn __ulock_wake(operation: u32, addr: *mut c_void, wake_value: u64) -> c_int;
    }
}

#[cfg(target_vendor = "apple")]
pub fn futex_wait(futex: &Atomic<u32>, expected: u32, timeout: Option<Duration>, shared: bool) -> bool {
    use apple::*;

    use crate::mem::size_of;

    let addr = futex.as_ptr().cast();
    let value = expected as u64;
    let size = size_of::<u32>();

    let flags = if shared {
        OS_SYNC_WAIT_ON_ADDRESS_SHARED
    } else {
        OS_SYNC_WAIT_ON_ADDRESS_NONE
    };
    let operation = if shared { UL_COMPARE_AND_WAIT_SHARED } else { UL_COMPARE_AND_WAIT };

    if let Some(timeout) = timeout {
        let timeout_ns = timeout.as_nanos().clamp(1, u64::MAX as u128) as u64;

        if let Some(wait) = os_sync_wait_on_address_with_timeout.get() {
            let r = unsafe {
                wait(
                    addr,
                    value,
                    size,
                    flags,
                    OS_CLOCK_MACH_ABSOLUTE_TIME,
                    timeout_ns,
                )
            };

            // We promote spurious wakeups (reported as EINTR) to normal ones for
            // simplicity.
            r != -1 || super::os::errno() != libc::ETIMEDOUT
        } else if let Some(wait) = __ulock_wait2.get() {
            let r = unsafe { wait(operation | ULF_NO_ERRNO, addr, value, timeout_ns, 0) };

            r != -libc::ETIMEDOUT
        } else if let Some(wait) = __ulock_wait.get() {
            let (timeout_us, truncated) = match timeout.as_micros().try_into() {
                Ok(timeout_us) => (u32::max(timeout_us, 1), false),
                Err(_) => (u32::MAX, true),
            };

            let r = unsafe { wait(operation | ULF_NO_ERRNO, addr, value, timeout_us) };

            // Report truncation as a spurious wakeup instead of a timeout.
            // Truncation occurs for timeout durations larger than 4295 s
            // ≈ 1 hour, so it should be considered.
            r != -libc::ETIMEDOUT || truncated
        } else {
            rtabort!("your system is below the minimum supported version of Rust");
        }
    } else {
        if let Some(wait) = os_sync_wait_on_address.get() {
            unsafe { wait(addr, value, size, flags) };
        } else if let Some(wait) = __ulock_wait2.get() {
            unsafe { wait(operation | ULF_NO_ERRNO, addr, value, 0, 0) };
        } else if let Some(wait) = __ulock_wait.get() {
            unsafe { wait(operation | ULF_NO_ERRNO, addr, value, 0) };
        } else {
            rtabort!("your system is below the minimum supported version of Rust");
        }

        true
    }
}

#[cfg(target_vendor = "apple")]
pub fn futex_wake(futex: &Atomic<u32>, shared: bool) -> bool {
    use apple::*;

    use crate::io::Error;
    use crate::mem::size_of;

    let flags = if shared {
        OS_SYNC_WAKE_BY_ADDRESS_SHARED
    } else {
        OS_SYNC_WAKE_BY_ADDRESS_NONE
    };
    let operation = if shared { UL_COMPARE_AND_WAIT_SHARED } else { UL_COMPARE_AND_WAIT };

    let addr = futex.as_ptr().cast();
    if let Some(wake) = os_sync_wake_by_address_any.get() {
        let r = unsafe { wake(addr, size_of::<u32>(), flags) };
        if r == 0 {
            true
        } else {
            match super::os::errno() {
                // There were no waiters to wake up.
                libc::ENOENT => false,
                err => rtabort!("__ulock_wake failed: {}", Error::from_raw_os_error(err)),
            }
        }
    } else if let Some(wake) = __ulock_wake.get() {
        // Judging by its use in pthreads, __ulock_wake can get interrupted, so
        // retry until either waking up a waiter or failing because there are no
        // waiters (ENOENT).
        loop {
            let r = unsafe { wake(operation | ULF_NO_ERRNO, addr, 0) };

            if r >= 0 {
                return true;
            } else {
                match -r {
                    libc::ENOENT => return false,
                    libc::EINTR => continue,
                    err => rtabort!("__ulock_wake failed: {}", Error::from_raw_os_error(err)),
                }
            }
        }
    } else {
        rtabort!("your system is below the minimum supported version of Rust");
    }
}

#[cfg(target_vendor = "apple")]
pub fn futex_wake_all(futex: &Atomic<u32>, shared: bool) {
    use apple::*;

    use crate::io::Error;
    use crate::mem::size_of;

    let flags = if shared {
        OS_SYNC_WAKE_BY_ADDRESS_SHARED
    } else {
        OS_SYNC_WAKE_BY_ADDRESS_NONE
    };
    let operation = if shared { UL_COMPARE_AND_WAIT_SHARED } else { UL_COMPARE_AND_WAIT };

    let addr = futex.as_ptr().cast();

    if let Some(wake) = os_sync_wake_by_address_all.get() {
        unsafe {
            wake(addr, size_of::<u32>(), flags);
        }
    } else if let Some(wake) = __ulock_wake.get() {
        // Judging by its use in pthreads, __ulock_wake can get interrupted, so
        // retry until either waking up a waiter or failing because there are no
        // waiters (ENOENT).
        loop {
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
