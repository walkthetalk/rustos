use core::arch::asm;

mod context;
use core::arch::global_asm;
use crate::syscall::syscall;
use crate::println;

use riscv::register::{
    mtvec::TrapMode,
    stvec,
    scause::{
        self,
        Trap,
        Exception,
        Interrupt,
    },
    stval,
    sstatus,
    sie,
};

use crate::task::{
    exit_current_and_run_next,
    suspend_current_and_run_next,
    current_user_token,
    current_trap_cx,
};
use crate::timer::set_next_trigger;
use crate::config::{TRAP_CONTEXT, TRAMPOLINE};

global_asm!(include_str!("trap.S"));

pub fn init() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

pub fn enable_interrupt() {
    unsafe { sstatus::set_sie(); }
}

pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer(); }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    let cx = current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.", stval, cx.sepc);
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!("jr {0}",
            in(reg) restore_va,
            in("a0")trap_cx_ptr,
            in("a1")user_satp
        );
    }
    panic!("Unreachable in back_to _user!");
}

pub use context::{TrapContext};
