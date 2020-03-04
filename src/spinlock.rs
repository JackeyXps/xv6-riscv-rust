//! spinlock module
//! unlike xv6-riscv, xv6-riscv-rust wraps data into a spinlock
//! useful reference crate spin(https://crates.io/crates/spin)

use crate::proc;
use crate::register::sstatus;
use core::cell::{Cell, UnsafeCell};
use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::{fence, AtomicBool, Ordering};

pub struct SpinLock<T: ?Sized> {
    // for debugging
    // None means this spinlock is not held by any cpu
    // TODO - Cell vs UnsafeCell
    cpu_id: Cell<Option<usize>>,
    name: &'static str,

    lock: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    pub const fn new(user_data: T, name: &'static str) -> SpinLock<T> {
        SpinLock {
            cpu_id: Cell::new(None),
            name,
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(user_data),
        }
    }
}

impl<T: ?Sized> SpinLock<T> {
    fn holding(&self) -> bool {
        let r: bool;
        push_off();
        unsafe {
            r = self.lock.load(Ordering::Relaxed) && self.cpu_id.get() == Some(proc::cpu_id());
        }
        pop_off();
        r
    }

    fn acquire_lock(&self) {
        push_off();
        if self.holding() {
            panic!("acquire");
        }
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) != false {}
        // Tell the C compiler and the processor to not move loads or stores
        // past this point, to ensure that the critical section's memory
        // references happen after the lock is acquired.
        fence(Ordering::SeqCst);
        unsafe {
            self.cpu_id.set(Some(proc::cpu_id()));
        }
    }

    /// Locks the spinlock and returns a guard.
    ///
    /// The returned guard can be deferenced for data access.
    /// i.e., we implement Deref trait for the guard.
    /// Also, the lock will also be dropped when the guard falls out of scope.
    ///
    /// ```
    /// let proc = SpinLock::new(0);
    /// {
    ///     let mut proc_locked = proc.lock();
    ///     // The lock is now locked and the data can be accessed
    ///     *proc_locked = 1;
    ///     // The lock is going to fall out of scope
    ///     // i.e. the lock will be released
    /// }
    /// ```
    pub fn lock(&self) -> SpinLockGuard<T> {
        self.acquire_lock();
        SpinLockGuard {
            spin_lock: &self,
            data: unsafe { &mut *self.data.get() },
        }
    }

    fn release_lock(&self) {
        if !self.holding() {
            panic!("release");
        }
        self.cpu_id.set(None);
        fence(Ordering::SeqCst);
        self.lock.store(false, Ordering::Release);
        pop_off();
    }
}

/// push_off/pop_off are like intr_off()/intr_on() except that they are matched:
/// it takes two pop_off()s to undo two push_off()s.  Also, if interrupts
/// are initially off, then push_off, pop_off leaves them off.
fn push_off() {
    let old: bool = sstatus::intr_get();
    sstatus::intr_off();
    proc::push_off(old);
}

fn pop_off() {
    if sstatus::intr_get() {
        panic!("spinlock.rs: pop_off - interruptable");
    }
    // a little difference from xv6-riscv
    // optional intr_on() moved to proc::pop_off()
    proc::pop_off();
}

pub struct SpinLockGuard<'a, T: ?Sized + 'a> {
    spin_lock: &'a SpinLock<T>,
    data: &'a mut T,
}

impl<'a, T: ?Sized> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinLockGuard<'a, T> {
    /// The dropping of the SpinLockGuard will call spinlock's release_lock(),
    /// through its reference to its original spinlock.
    fn drop(&mut self) {
        self.spin_lock.release_lock();
    }
}

/// Copy from crate spin(https://crates.io/crates/spin)
#[cfg(feature = "unit_test")]
pub mod tests {
    use super::*;

    pub fn smoke() {
        let m = SpinLock::new((), "smoke");
        m.lock();
        m.lock();
        panic!("spinlock::tests::smoke");
    }
}