use core::sync::atomic::{AtomicPtr, AtomicU32};
use windows_sys::Win32::System::{
    Threading::{WaitOnAddress, WakeByAddressAll, WakeByAddressSingle},
    WindowsProgramming::INFINITE,
};

#[inline]
pub fn wait(a: &AtomicU32, expected: u32) {
    let ptr: *const AtomicU32 = a;
    let expected_ptr: *const u32 = &expected;
    unsafe { WaitOnAddress(ptr.cast(), expected_ptr.cast(), 4, INFINITE) };
}

pub fn wait_ptr<T>(a: *const AtomicPtr<T>, expected: *const T) {
    let expected_ptr: *const *const T = &expected as _;
    unsafe { WaitOnAddress(a.cast(), expected_ptr.cast(), 8, INFINITE )};
}

#[inline]
pub fn wake_one(ptr: *const AtomicU32) {
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
pub fn wake_all_ptr<T>(ptr: *const AtomicPtr<T>) {
    unsafe { WakeByAddressAll(ptr.cast()) };
}
