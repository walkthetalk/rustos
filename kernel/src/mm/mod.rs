mod heap_allocator;
mod address;
mod frame_allocator;
mod page_tables;
mod memory_set;

use page_tables::{PageTable, PTEFlags};
use address::VPNRange;
pub use heap_allocator::init_heap;
pub use heap_allocator::heap_test;
pub use address::{PhysAddr, VirtAddr, PhysPageNum, VirtPageNum};
pub use frame_allocator::{FrameTracker, frame_alloc};
pub use page_tables::PageTableEntry;
pub use memory_set::{MemorySet, KERNEL_SPACE};
pub use memory_set::remap_test;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.clone().lock().activate();
}
