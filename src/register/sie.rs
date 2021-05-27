//! sie register

const SSIE: usize = 1 << 1; // software
const STIE: usize = 1 << 5; // timer
const SEIE: usize = 1 << 9; // external

#[inline]
unsafe fn read() -> usize {
    let ret: usize;
    llvm_asm!("csrr $0, sie":"=r"(ret):::"volatile");
    ret
}

#[inline]
unsafe fn write(x: usize) {
    llvm_asm!("csrw sie, $0"::"r"(x)::"volatile");
}

/// enable all software interrupts
/// still need to set SIE bit in sstatus
pub unsafe fn intr_on() {
    let mut sie = read();
    //如果设置了sip寄存器中的STIP位，则定时器中断将挂起
    //
    sie |= SSIE | STIE | SEIE;
    write(sie);
}
