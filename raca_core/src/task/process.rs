use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{
    boxed::Box,
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use object::{File, Object, ObjectSegment};
use spin::{Lazy, RwLock};
use x86_64::{instructions::interrupts, structures::paging::OffsetPageTable, VirtAddr};

use crate::{
    fs::{
        operation::{FileDescriptorManager, OpenMode},
        FileRef,
    },
    memory::{ExtendedPageTable, MappingType, MemoryManager, KERNEL_PAGE_TABLE},
};

use super::{
    signal::SignalManager,
    thread::{SharedThread, Thread},
    SCHEDULER,
};

const KERNEL_PROCESS_NAME: &str = "kernel";

static PROCESSES: RwLock<VecDeque<SharedProcess>> = RwLock::new(VecDeque::new());
pub static KERNEL_PROCESS: Lazy<SharedProcess> = Lazy::new(|| Process::new_kernel_process());

pub(super) type SharedProcess = Arc<RwLock<Box<Process>>>;
pub(super) type WeakSharedProcess = Weak<RwLock<Box<Process>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub u64);

impl ProcessId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ProcessId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[allow(dead_code)]
pub struct Process {
    pub id: ProcessId,
    pub name: String,
    pub page_table: OffsetPageTable<'static>,
    pub threads: Vec<SharedThread>,
    pub file_descriptor_manager: FileDescriptorManager,
    pub signal_manager: SignalManager,
    pub father: Option<WeakSharedProcess>,
}

impl Process {
    pub fn new(name: &str) -> Self {
        let process = Process {
            id: ProcessId::new(),
            name: String::from(name),
            page_table: unsafe { KERNEL_PAGE_TABLE.lock().deep_copy() },
            threads: Default::default(),
            file_descriptor_manager: FileDescriptorManager::new(BTreeMap::new()),
            signal_manager: SignalManager::new(64),
            father: None,
        };

        process
    }

    pub fn new_kernel_process() -> SharedProcess {
        let process = Arc::new(RwLock::new(Box::new(Self::new(KERNEL_PROCESS_NAME))));
        PROCESSES.write().push_back(process.clone());
        process
    }

    pub fn new_user_process(
        name: &str,
        elf_data: &'static [u8],
        stdin: FileRef,
        stdout: FileRef,
    ) -> SharedProcess {
        let binary = ProcessBinary::parse(elf_data);
        interrupts::without_interrupts(|| {
            let process = Arc::new(RwLock::new(Box::new(Self::new(name))));
            process
                .write()
                .file_descriptor_manager
                .add_file(stdin, OpenMode::Read);
            process
                .write()
                .file_descriptor_manager
                .add_file(stdout, OpenMode::Write);
            ProcessBinary::map_segments(&binary, &mut process.write().page_table, None);
            Thread::new_user_thread(Arc::downgrade(&process), binary.entry() as usize);
            PROCESSES.write().push_back(process.clone());
            process
        })
    }

    pub fn exit_process(&self) {
        for thread in self.threads.iter() {
            SCHEDULER.lock().remove(Arc::downgrade(thread));
        }

        let mut processes = PROCESSES.write();
        if let Some(index) = processes
            .iter()
            .position(|process| process.read().id == self.id)
        {
            processes.remove(index);
        }
    }
}

pub struct ProcessBinary;

impl ProcessBinary {
    fn parse(bin: &'static [u8]) -> File<'static> {
        File::parse(bin).expect("Failed to parse ELF binary")
    }

    pub fn map_segments(
        elf_file: &File,
        page_table: &mut OffsetPageTable<'static>,
        base: Option<u64>,
    ) {
        let base = base.unwrap_or(0);
        for segment in elf_file.segments() {
            let _ = MemoryManager::alloc_range(
                VirtAddr::new(segment.address() as u64 + base),
                segment.size(),
                MappingType::UserCode.flags(),
                page_table,
            );
            //.expect("Failed to allocate memory for ELF segment");

            if let Ok(data) = segment.data() {
                page_table.write_to_mapped_address(data, VirtAddr::new(segment.address() + base));
            }
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { self.page_table.free_user_page_table() };
    }
}
