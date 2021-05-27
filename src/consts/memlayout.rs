//! Physical memory layout
//!
//! qemu -machine virt is set up like this,
//! based on qemu's hw/riscv/virt.c:
//!
//! 00001000 -- boot ROM, provided by qemu
//! 02000000 -- CLINT
//! 0C000000 -- PLIC
//! 10000000 -- uart0
//! 10001000 -- virtio disk
//! 80000000 -- boot ROM jumps here in machine mode
//!             -kernel loads the kernel here
//! unused RAM after 80000000.
//!
//! the kernel uses physical memory thus:
//! 80000000 -- entry.S, then kernel text and data
//! end -- start of kernel page allocation area
//! PHYSTOP -- end RAM used by the kernel

use super::*;
// CLINT 用来维护与 内存映射 与 软件和时钟中断 有关的状态寄存器
//CLINT 的每个地址代表一个寄存器
/// local interrupt controller, which contains the timer.
pub const CLINT: ConstAddr = ConstAddr(0x2000000);
pub const CLINT_MAP_SIZE: usize = 0x10000;
// CLINT_MTIMECMP: 这是一个读写寄存器，保存一个64位的值。每当mtime大于或等于mtimecmp寄存器中的值时，
// 计时器中断将挂起。定时器中断用于驱动RISC-V核的mip CSR的MTIP位。
pub const CLINT_MTIMECMP: ConstAddr = CLINT.const_add(0x4000);
// mtime 就是计时器，每个tick_time就加一
// CLINT_MTIME: 保存着当前mtime寄存器的值
pub const CLINT_MTIME: ConstAddr = CLINT.const_add(0xbff8); 

/// qemu puts UART registers here in physical memory.
pub const UART0: ConstAddr = ConstAddr(0x10000000);
pub const UART0_MAP_SIZE: usize = PGSIZE;
pub const UART0_IRQ: usize = 10;

/// virtio mmio interface
pub const VIRTIO0: ConstAddr = ConstAddr(0x10001000);
pub const VIRTIO0_MAP_SIZE: usize = PGSIZE;
pub const VIRTIO0_IRQ: usize = 1;

/// qemu puts programmable interrupt controller here.
pub const PLIC: ConstAddr = ConstAddr(0x0c000000);
pub const PLIC_MAP_SIZE: usize = 0x400000;

/// the kernel expects there to be RAM
/// for use by the kernel and user pages
/// from physical address 0x80000000 to PHYSTOP.
pub const KERNBASE: ConstAddr = ConstAddr(0x80000000);
pub const PHYSTOP: ConstAddr = KERNBASE.const_add(128 * 1024 * 1024);

/// map the trampoline page to the highest address,
/// in both user and kernel space.
/// 0x3FFFFFF000
pub const TRAMPOLINE: ConstAddr = MAXVA.const_sub(PGSIZE);

/// trapframe is below the trampoline
/// 0x3FFFFFE000
pub const TRAPFRAME: ConstAddr = TRAMPOLINE.const_sub(PGSIZE);

/// user text/code start address
pub const USERTEXT: ConstAddr = ConstAddr(0);
