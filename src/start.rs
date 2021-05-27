use core::convert::Into;//强制转换的库

use crate::{consts::{CLINT_MTIMECMP, NCPU}, register::sie};
use crate::register::{
    clint, medeleg, mepc, mhartid, mideleg, mie, mscratch, mstatus, mtvec, satp, tp,
};
use crate::rmain::rust_main;

/// for each cpu, only 6 of 32 usize are used, others are reserved.
static mut MSCRATCH0: [usize; NCPU * 32] = [0; NCPU * 32];

#[no_mangle]
pub unsafe fn start() -> ! {
    // set M Previous Privilege mode to Supervisor, for mret.
    // 设置模式为supervisor模式
    // riscv一般有三种模式，从高到低依次是机器模式，特权模式和用户模式
    // 这里设置模式应该是设置mret之后的模式，而不是立即变成了特权模式
    mstatus::set_mpp(mstatus::MPP::Supervisor);

    // set M Exception Program Counter to main, for mret.
    // mepc存发生异常的指令的地址
    mepc::write(rust_main as usize);

    // disable paging for now.
    satp::write(0);

    // delegate all interrupts and exceptions to supervisor mode.
    // medeleg 和 mideleg 寄存器中提供单独的读/写位，来指定某些异常和中断类型可以直接由某一较低的模式来处理
    // 置位将把S或U态的trap转交给S态的trap处理程序
    medeleg::write(0xffff);
    mideleg::write(0xffff);
    //sie:Supervisor interrupt-enable register
    //它控制了定时器等中断的开启或者不开启
    sie::intr_on();

    // ask for clock interrupts.
    timerinit();

    // keep each CPU's hartid in its tp register, for cpuid().
    let id = mhartid::read();
    // X4 (TP)来保存线程本地存储的值
    // tp: Thread pointer
    tp::write(id);

    // switch to supervisor mode and jump to main().
    llvm_asm!("mret"::::"volatile");

    // cannot panic or print here
    loop {}
}

/// set up to receive timer interrupts in machine mode,
/// which arrive at timervec in kernelvec.S,
/// which turns them into software interrupts for
/// devintr() in trap.rs.
unsafe fn timerinit() {
    // each CPU has a separate source of timer interrupts.
    let id = mhartid::read();

    // ask the CLINT for a timer interrupt.
    let interval: u64 = 1000000; // cycles; about 1/10th second in qemu.
    //设置时钟间隔
    clint::add_mtimecmp(id, interval);

    // prepare information in scratch[] for timervec.
    // scratch[0..3] : space for timervec to save registers.留出一片空间来保存将要使用的寄存器的值，类似栈的作用
    // scratch[4] : address of CLINT MTIMECMP register.
    // scratch[5] : desired interval (in cycles) between timer interrupts.
    let offset = 32 * id;

    //我们通常用xlen表示整数寄存器位数或者说地址空间位数，所以对于RV32I, xlen=32, 对于RV64I, xlen=64。
    //mscratch寄存器是一个xlen位的读/写寄存器，专用于机器模式使用。
    //通常，它用于保存一个指向机器模式hart-local上下文空间的指针，
    //并在进入m模式trap处理程序时与用户寄存器交换
    MSCRATCH0[offset + 4] = 8 * id + Into::<usize>::into(CLINT_MTIMECMP);
    MSCRATCH0[offset + 5] = interval as usize;
    mscratch::write((MSCRATCH0.as_ptr() as usize) + offset * core::mem::size_of::<usize>());

    // set the machine-mode trap handler.
    extern "C" {
        // 这里面会做如下操作，函数开始和结束先利用mscratch[0..3](stack)保存和恢复进行函数要用寄存器，懂汇编都知道
        // 然后 CLINT_MTIMECMP += interval
        // 然后 sip 的第2位置为1，来raise a supervisor software interrupt
        fn timervec();
    }
    //mtvec寄存器是一个xlen位的读/写寄存器，它保存m模式trap向量的基址。
    //也就是这里将timervec()设置为handler
    mtvec::write(timervec as usize);

    // enable machine-mode interrupts.
    mstatus::set_mie();

    // enable machine-mode timer interrupts.
    mie::set_mtie();
}
