//! CLINT operation, refer doc/FU540-C000-v1.0.pdf for detail.
//!
//! note: mtime and mtimecmp are both 64-bit registers
//!     they will not probably exceed the time the machine can run.

use core::ptr;//读取内存的库，直接读写指针指向的内存值
use core::convert::Into;

use crate::consts::{CLINT_MTIME, CLINT_MTIMECMP};

#[inline]
unsafe fn read_mtime() -> u64 {
    ptr::read_volatile(Into::<usize>::into(CLINT_MTIME) as *const u64)
}

#[inline]
unsafe fn write_mtimecmp(mhartid: usize, value: u64) {
    //每个core的CLINT_MTIMECMP地址有偏移
    let offset = Into::<usize>::into(CLINT_MTIMECMP) + 8 * mhartid;
    ptr::write_volatile(offset as *mut u64, value);
}

pub unsafe fn add_mtimecmp(mhartid: usize, interval: u64) {
    let value = read_mtime();
    //由于CLINT_MTIMECMP表示中断的时间，用当前时间+间隙时间来设置中断时间即可
    write_mtimecmp(mhartid, value + interval);
}

pub unsafe fn read_mtimecmp(mhartid: usize) -> u64 {
    let offset = Into::<usize>::into(CLINT_MTIMECMP) + 8 * mhartid;
    ptr::read_volatile(offset as *const u64)
}
