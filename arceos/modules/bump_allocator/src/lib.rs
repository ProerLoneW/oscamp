#![no_std]

use allocator::{AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0, 
            end: 0,
            b_pos: 0,
            p_pos: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        // 新内存区域
        if start < self.end && (start + size) > self.start {
            return Err(allocator::AllocError::MemoryOverlap);
        }
        if start < self.start {
            self.start = start;
        }
        if (start + size) > self.end {
            self.end = start + size;
            self.p_pos = self.end;
        }
        Ok(())
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: core::alloc::Layout) -> AllocResult<core::ptr::NonNull<u8>> {
        if self.available_bytes() < layout.size() {
            return Err(allocator::AllocError::NoMemory);
        }
        let result = core::ptr::NonNull::new(self.start as *mut u8);
        
        if let Some(result) = result {
            self.start += layout.size();
            return Ok(result);
        } else {
            panic!("unknown err in alloc!")
        }
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        let ret = pos.as_ptr() as usize + layout.size();
        if ret == self.start {
            // log::warn('{}')
            self.start -= layout.size();
        }
    }
    
    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.end - self.b_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        // 页面的分配按倒序进行
        let _align = align_pow2 / PAGE_SIZE;
        if align_pow2 % PAGE_SIZE != 0 || !align_pow2.is_power_of_two() {
            return Err(allocator::AllocError::InvalidParam);
        }

        if self.p_pos < num_pages * PAGE_SIZE {
            return Err(allocator::AllocError::NoMemory);
        }

        // 调整 p_pos，分配页面
        self.p_pos -= num_pages * PAGE_SIZE;
        Ok(self.p_pos)
    }
    
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        self.p_pos += num_pages * PAGE_SIZE;
        // log::warn('{}')
        // 没有回收
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.start) / PAGE_SIZE
    }
}
