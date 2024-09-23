use atomic_wait::{wait, wait_u64, wake_all, wake_all_u64, wake_one, wake_one_u64};
use std::{
    sync::atomic::{AtomicU32, Ordering::Relaxed},
    thread::sleep,
    time::{Duration, Instant},
};
use std::sync::atomic::AtomicU64;

#[test]
fn wake_null() {
    wake_one(std::ptr::null::<AtomicU32>());
    wake_all(std::ptr::null::<AtomicU32>());
}

#[test]
fn wake_null_u64() {
    wake_one_u64(std::ptr::null::<AtomicU64>());
    wake_all_u64(std::ptr::null::<AtomicU64>());
}

#[test]
fn wake_nothing() {
    let a = AtomicU32::new(0);
    wake_one(&a);
    wake_all(&a);
}

#[test]
fn wake_nothing_u64() {
    let a = AtomicU64::new(0);
    wake_one_u64(&a);
    wake_all_u64(&a);
}

#[test]
fn wait_unexpected() {
    let t = Instant::now();
    let a = AtomicU32::new(0);
    wait(&a, 1);
    assert!(t.elapsed().as_millis() < 100);
}

#[test]
fn wait_unexpected_u64() {
    let t = Instant::now();
    let a = AtomicU64::new(0);
    wait_u64(&a, 1);
    assert!(t.elapsed().as_millis() < 100);
}

#[test]
fn wait_wake() {
    let t = Instant::now();
    let a = AtomicU32::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            sleep(Duration::from_millis(100));
            a.store(1, Relaxed);
            wake_one(&a);
        });
        while a.load(Relaxed) == 0 {
            wait(&a, 0);
        }
        assert_eq!(a.load(Relaxed), 1);
        assert!((90..400).contains(&t.elapsed().as_millis()));
    });
}

#[test]
fn wait_wake_u64() {
    let t = Instant::now();
    let a = AtomicU64::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            sleep(Duration::from_millis(100));
            a.store(1, Relaxed);
            wake_one_u64(&a);
        });
        while a.load(Relaxed) == 0 {
            wait_u64(&a, 0);
        }
        assert_eq!(a.load(Relaxed), 1);
        assert!((90..400).contains(&t.elapsed().as_millis()));
    });
}
