use core::sync::atomic::{AtomicBool, Ordering};

use crate::driver::virtio_disk::DISK;
use crate::register::tp;
use crate::fs::BCACHE;
use crate::mm::kalloc::KERNEL_HEAP;
use crate::mm::{kvm_init, kvm_init_hart};
use crate::plic;
use crate::process::{PROC_MANAGER, CPU_MANAGER};
use crate::trap::trap_init_hart;

/// Used by hart 0 to communicate with other harts.
/// When hart 0 finished some initial work,
/// it sets started to true to tell other harts that they can run
///
/// note: actually a simple Bool would be enough,
///     because it is only written once, but just...
///
/// hart 0 用此变量来和其他hart沟通的变量。
/// 当hart0完成了某些初始化的工作时，它会把这个值设为true，
/// 这样其他的hart就可以开始工作了。
/// AtomicBool表示该值的操作是原子化的，本质上只是一个bool。
///
static STARTED: AtomicBool = AtomicBool::new(false);

/// start() jumps here in supervisor mode on all CPUs.
pub unsafe fn rust_main() -> ! {
    // explicitly use tp::read here
    /// 从tp寄存器中读出当前cpu的id
    let cpuid = tp::read();
    
    if cpuid == 0 {
        ///初始化console
        crate::console::consoleinit();
        println!();
        println!("xv6-riscv-rust is booting");
        println!();
        /// 初始化堆
        /// 找到可分配的所有内存大小，并新建一个buddy system实体
        KERNEL_HEAP.kinit();
        /// 新建kernel的page table
        /// 将io，RAM，ROM等分别建立page table上物理地址和虚拟地址的映射
        kvm_init(); // init kernel page table
        /// 为进程表里的进程分配栈空间，且建立page table上的映射
        PROC_MANAGER.proc_init(); // process table

        /// 配置satp寄存器中的值，启动页表机制
        kvm_init_hart(); // trun on paging
        /// 调用C库中的kernelvec()函数，将stvec寄存器中写入trap hendler的地址
        trap_init_hart(); // install kernel trap vector

        /// 初始化PLIC (Platform-Level Interrupt Controller)
        /// 应该不会用到吧😅
        plic::init();
        plic::init_hart(cpuid);
        /// 初始化cache
        BCACHE.binit();             // buffer cache
        /// 初始化磁盘的锁
        DISK.lock().init();         // emulated hard disk

        /// 初始化第一个进程
        PROC_MANAGER.user_init();   // first user process

        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {}

        println!("hart {} starting", cpuid);
        kvm_init_hart(); // turn on paging
        trap_init_hart(); // install kernel trap vector
        plic::init_hart(cpuid); // ask PLIC for device interrupts

        // LTODO - init other things
        loop {}
    }

    #[cfg(feature = "unit_test")]
    super::test_main_entry();

    CPU_MANAGER.scheduler();
}
