use core::arch::asm;
use xmas_elf::ElfFile;
//use core::slice::SlicePattern;
use crate::trap::TrapContext;
use crate::task::TaskContext;
use crate::config::*;
use crate::println;

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [
    KernelStack { data: [0; KERNEL_STACK_SIZE], };
    MAX_APP_NUM
];

static USER_STACK: [UserStack; MAX_APP_NUM] = [
    UserStack { data: [0; USER_STACK_SIZE], };
    MAX_APP_NUM
];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, trap_cx: TrapContext, task_cx: TaskContext) -> &'static mut TaskContext {
        unsafe {
            let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
            *trap_cx_ptr = trap_cx;
            let task_cx_ptr = (trap_cx_ptr as usize - core::mem::size_of::<TaskContext>()) as *mut TaskContext;
            *task_cx_ptr = task_cx;
            task_cx_ptr.as_mut().unwrap()
        }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

static mut APP_BASE_ADDR: [usize; MAX_APP_NUM] = [
    0;
    MAX_APP_NUM
];
fn get_base_i(app_id: usize) -> usize {
    unsafe { println!("get base addr {} as {:#x}", app_id, (*APP_BASE_ADDR.get_unchecked(app_id))); }
    unsafe { *APP_BASE_ADDR.get_unchecked(app_id)}
}
fn set_base_i(app_id: usize, addr: usize) {
    println!("set base addr {} as {:#x}", app_id, addr);
    unsafe {
        *APP_BASE_ADDR.get_unchecked_mut(app_id) = addr;
    }
}

pub fn get_num_app() -> usize {
    extern "C" { fn _num_app(); }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

pub fn load_apps() {
    extern "C" { fn _num_app(); }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };

    println!("num_app = {}", num_app);
    for i in 0..num_app {
        println!(
            "app_{} [{:#x}, {:#x}] size={:#x}",
            i,
            app_start[i],
            app_start[i+1],
            app_start[i+1] - app_start[i]
        );
        debug_elf(app_start[i], app_start[i+1]);
    }
    loop{}

    // clear i-cache first
    unsafe { asm!("fence.i"); }
    // load apps
    for i in 0..num_app {
        // load app from data section to memory
        let addr_size = 8;
        let laddr = unsafe {*(app_start[i] as *const usize)};
        let len = app_start[i+1] - app_start[i];
        // load address is emebedded in app
        set_base_i(i, laddr + addr_size);

        // clear region
        (laddr..laddr + APP_SIZE_LIMIT).for_each(|addr| unsafe {
            (addr as *mut u8).write_volatile(0)
        });

        // copy
        let src = unsafe {
            core::slice::from_raw_parts(
                (app_start[i]) as *const u8,
                len
            )
        };
        let dst = unsafe {
            core::slice::from_raw_parts_mut(laddr as *mut u8, len)
        };
        dst.copy_from_slice(src);
    }
}

pub fn init_app_cx(app_id: usize) -> &'static TaskContext {
    KERNEL_STACK[app_id].push_context(
        TrapContext::app_init_context(get_base_i(app_id), USER_STACK[app_id].get_sp()),
        TaskContext::goto_restore(),
    )
}

fn debug_elf(start_addr: usize, end_addr: usize) {
    let data_array = unsafe {
        core::slice::from_raw_parts(start_addr as *const u8, end_addr - start_addr)
    };
    let elf = ElfFile::new(data_array).unwrap();
    let elf_header = elf.header;
    let magic = elf_header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
    let ph_count = elf_header.pt2.ph_count();
    println!("ph_count = {}", ph_count);
    for i in 0..ph_count {
        let ph = elf.program_header(i).unwrap();
        if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
            println!(
                "offset={:#x},va={:#x},pa={:#x},filesz={:#x},memsz={:#x},align={:#x}",
                ph.offset(),
                ph.virtual_addr(),
                ph.physical_addr(),
                ph.file_size(),
                ph.mem_size(),
                ph.align(),
            );
        }
    }
}
