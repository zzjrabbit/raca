use core::ptr::NonNull;

use pod::{FromBytes, Immutable, IntoBytes, derive};
use spin::Mutex;
use ustd::vm::{MMUFlags, Vmar, Vmo};
use virtio_drivers::{
    Error, Hal, PAGE_SIZE, PhysAddr,
    device::input::VirtIOInput,
    transport::{DeviceStatus, DeviceType, InterruptStatus, Transport, mmio::MmioVersion},
};

pub fn init(vmo: Vmo) {
    VirtIOInput::<HalImpl, TransportImpl>::new(TransportImpl::new(vmo))
        .unwrap()
        .ack_interrupt();
}

struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(
        pages: usize,
        _direction: virtio_drivers::BufferDirection,
    ) -> (PhysAddr, core::ptr::NonNull<u8>) {
        let vmo = Vmo::allocate_continuous(pages).unwrap();
        let vmar = Vmar::root().allocate(vmo.len()).unwrap();
        vmar.map(0, &vmo, MMUFlags::DATA).unwrap();
        (
            vmo.start().unwrap() as PhysAddr,
            NonNull::new(vmar.base() as *mut u8).unwrap(),
        )
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, _vaddr: core::ptr::NonNull<u8>, _pages: usize) -> i32 {
        0
    }

    unsafe fn mmio_phys_to_virt(_paddr: PhysAddr, _size: usize) -> core::ptr::NonNull<u8> {
        unimplemented!()
    }

    unsafe fn share(
        _buffer: core::ptr::NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) -> PhysAddr {
        unimplemented!()
    }

    unsafe fn unshare(
        _paddr: PhysAddr,
        _buffer: core::ptr::NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) {
        unimplemented!()
    }
}

struct TransportImpl {
    vmo: Vmo,
    header: Mutex<VirtIOHeader>,
    version: MmioVersion,
}

const CONFIG_SPACE_OFFSET: usize = 256;

impl TransportImpl {
    fn new(vmo: Vmo) -> Self {
        let header = vmo.read_val(0).unwrap();
        crate::termpln!("header: {:x?}", header);
        Self {
            vmo,
            header: Mutex::new(header),
            version: MmioVersion::try_from(header.version).unwrap(),
        }
    }

    fn read_cfg(&self, offset: usize, data: &mut [u8]) {
        self.vmo.read(offset + CONFIG_SPACE_OFFSET, data).unwrap();
    }

    fn write_cfg(&self, offset: usize, data: &[u8]) {
        self.vmo.write(offset + CONFIG_SPACE_OFFSET, data).unwrap();
    }

    fn config_space_len(&self) -> usize {
        self.vmo.len() - CONFIG_SPACE_OFFSET
    }
}

macro_rules! read_header {
    ($s: ident, $f: ident) => {{
        let mut header = $s.header.lock();
        header.$f = $s
            .vmo
            .read_val(memoffset::offset_of!(VirtIOHeader, $f))
            .unwrap();
        header.$f
    }};
}

macro_rules! write_header {
    ($s: ident, $f: ident, $value: expr) => {{
        let mut header = $s.header.lock();
        header.$f = $value;
        $s.vmo
            .write_val(memoffset::offset_of!(VirtIOHeader, $f), &header.$f)
            .unwrap();
    }};
}

impl Transport for TransportImpl {
    fn device_type(&self) -> virtio_drivers::transport::DeviceType {
        DeviceType::try_from(read_header!(self, device_id)).unwrap()
    }

    fn read_device_features(&mut self) -> u64 {
        write_header!(self, device_features_sel, 0);
        let mut device_features_bits = read_header!(self, device_features) as u64;
        write_header!(self, device_features_sel, 1);
        device_features_bits += (read_header!(self, device_features) as u64) << 32;
        device_features_bits
    }

    fn write_driver_features(&mut self, driver_features: u64) {
        write_header!(self, driver_features_sel, 0); // driver features [0, 32)
        write_header!(self, driver_features, driver_features as u32);
        write_header!(self, driver_features_sel, 1); // driver features [32, 64)
        write_header!(self, driver_features, (driver_features >> 32) as u32);
    }

    fn max_queue_size(&mut self, queue: u16) -> u32 {
        write_header!(self, queue_sel, queue as _);
        read_header!(self, queue_num_max)
    }

    fn notify(&mut self, queue: u16) {
        write_header!(self, queue_notify, queue as _);
    }

    fn get_status(&self) -> DeviceStatus {
        DeviceStatus::from_bits_truncate(read_header!(self, status))
    }

    fn set_status(&mut self, status: DeviceStatus) {
        write_header!(self, status, status.bits());
    }

    fn set_guest_page_size(&mut self, guest_page_size: u32) {
        match self.version {
            MmioVersion::Legacy => {
                write_header!(self, legacy_guest_page_size, guest_page_size);
            }
            MmioVersion::Modern => {
                // No-op, modern devices don't care.
            }
        }
    }

    fn requires_legacy_layout(&self) -> bool {
        match self.version {
            MmioVersion::Legacy => true,
            MmioVersion::Modern => false,
        }
    }

    fn queue_set(
        &mut self,
        queue: u16,
        size: u32,
        descriptors: PhysAddr,
        driver_area: PhysAddr,
        device_area: PhysAddr,
    ) {
        match self.version {
            MmioVersion::Legacy => {
                let align = PAGE_SIZE as u32;
                let pfn = (descriptors / PAGE_SIZE as u64).try_into().unwrap();
                assert_eq!(u64::from(pfn) * PAGE_SIZE as u64, descriptors);
                write_header!(self, queue_sel, queue as _);
                write_header!(self, queue_num, size);
                write_header!(self, legacy_queue_align, align);
                write_header!(self, legacy_queue_pfn, pfn);
            }
            MmioVersion::Modern => {
                write_header!(self, queue_sel, queue as _);
                write_header!(self, queue_num, size);
                write_header!(self, queue_desc_low, descriptors as _);
                write_header!(self, queue_desc_high, (descriptors >> 32) as _);
                write_header!(self, queue_driver_low, driver_area as _);
                write_header!(self, queue_driver_high, (driver_area >> 32) as _);
                write_header!(self, queue_device_low, device_area as _);
                write_header!(self, queue_device_high, (device_area >> 32) as _);
                write_header!(self, queue_ready, 1);
            }
        }
    }

    fn queue_unset(&mut self, queue: u16) {
        match self.version {
            MmioVersion::Legacy => {
                write_header!(self, queue_sel, queue as _);
                write_header!(self, queue_num, 0);
                write_header!(self, legacy_queue_align, 0);
                write_header!(self, legacy_queue_pfn, 0);
            }
            MmioVersion::Modern => {
                write_header!(self, queue_sel, queue as _);

                write_header!(self, queue_ready, 0);
                // Wait until we read the same value back, to ensure synchronisation (see 4.2.2.2).
                let mut queue_ready = read_header!(self, queue_ready);
                while queue_ready != 0 {
                    queue_ready = read_header!(self, queue_ready)
                }

                write_header!(self, queue_num, 0);
                write_header!(self, queue_desc_low, 0);
                write_header!(self, queue_desc_high, 0);
                write_header!(self, queue_driver_low, 0);
                write_header!(self, queue_driver_high, 0);
                write_header!(self, queue_device_low, 0);
                write_header!(self, queue_device_high, 0);
            }
        }
    }

    fn queue_used(&mut self, queue: u16) -> bool {
        write_header!(self, queue_sel, queue as _);
        match self.version {
            MmioVersion::Legacy => read_header!(self, legacy_queue_pfn) != 0,
            MmioVersion::Modern => read_header!(self, queue_ready) != 0,
        }
    }

    fn ack_interrupt(&mut self) -> InterruptStatus {
        let interrupt = read_header!(self, interrupt_status);
        if interrupt != 0 {
            write_header!(self, interrupt_ack, interrupt);
            InterruptStatus::from_bits_truncate(interrupt)
        } else {
            InterruptStatus::empty()
        }
    }

    fn read_config_generation(&self) -> u32 {
        read_header!(self, config_generation)
    }

    fn read_config_space<T: FromBytes + IntoBytes>(&self, offset: usize) -> Result<T, Error> {
        assert!(
            align_of::<T>() <= 4,
            "Driver expected config space alignment of {} bytes, but VirtIO only guarantees 4 byte alignment.",
            align_of::<T>()
        );
        assert!(offset % align_of::<T>() == 0);

        if self.config_space_len() < offset + size_of::<T>() {
            Err(Error::ConfigSpaceTooSmall)
        } else {
            let mut buffer = alloc::vec![0u8; size_of::<T>()];
            self.read_cfg(offset, &mut buffer);
            Ok(T::read_from_bytes(&buffer).unwrap())
        }
    }

    fn write_config_space<T: IntoBytes + Immutable>(
        &mut self,
        offset: usize,
        value: T,
    ) -> Result<(), Error> {
        assert!(
            align_of::<T>() <= 4,
            "Driver expected config space alignment of {} bytes, but VirtIO only guarantees 4 byte alignment.",
            align_of::<T>()
        );
        assert!(offset % align_of::<T>() == 0);

        if self.config_space_len() < offset + size_of::<T>() {
            Err(Error::ConfigSpaceTooSmall)
        } else {
            self.write_cfg(offset, value.as_bytes());
            Ok(())
        }
    }
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct VirtIOHeader {
    /// Magic value
    magic: u32,

    /// Device version number
    ///
    /// Legacy device returns value 0x1.
    version: u32,

    /// Virtio Subsystem Device ID
    device_id: u32,

    /// Virtio Subsystem Vendor ID
    vendor_id: u32,

    /// Flags representing features the device supports
    device_features: u32,

    /// Device (host) features word selection
    device_features_sel: u32,

    /// Reserved
    __r1: [u32; 2],

    /// Flags representing device features understood and activated by the driver
    driver_features: u32,

    /// Activated (guest) features word selection
    driver_features_sel: u32,

    /// Guest page size
    ///
    /// The driver writes the guest page size in bytes to the register during
    /// initialization, before any queues are used. This value should be a
    /// power of 2 and is used by the device to calculate the Guest address
    /// of the first queue page (see QueuePFN).
    legacy_guest_page_size: u32,

    /// Reserved
    __r2: u32,

    /// Virtual queue index
    ///
    /// Writing to this register selects the virtual queue that the following
    /// operations on the QueueNumMax, QueueNum, QueueAlign and QueuePFN
    /// registers apply to. The index number of the first queue is zero (0x0).
    queue_sel: u32,

    /// Maximum virtual queue size
    ///
    /// Reading from the register returns the maximum size of the queue the
    /// device is ready to process or zero (0x0) if the queue is not available.
    /// This applies to the queue selected by writing to QueueSel and is
    /// allowed only when QueuePFN is set to zero (0x0), so when the queue is
    /// not actively used.
    queue_num_max: u32,

    /// Virtual queue size
    ///
    /// Queue size is the number of elements in the queue. Writing to this
    /// register notifies the device what size of the queue the driver will use.
    /// This applies to the queue selected by writing to QueueSel.
    queue_num: u32,

    /// Used Ring alignment in the virtual queue
    ///
    /// Writing to this register notifies the device about alignment boundary
    /// of the Used Ring in bytes. This value should be a power of 2 and
    /// applies to the queue selected by writing to QueueSel.
    legacy_queue_align: u32,

    /// Guest physical page number of the virtual queue
    ///
    /// Writing to this register notifies the device about location of the
    /// virtual queue in the Guest’s physical address space. This value is
    /// the index number of a page starting with the queue Descriptor Table.
    /// Value zero (0x0) means physical address zero (0x00000000) and is illegal.
    /// When the driver stops using the queue it writes zero (0x0) to this
    /// register. Reading from this register returns the currently used page
    /// number of the queue, therefore a value other than zero (0x0) means that
    /// the queue is in use. Both read and write accesses apply to the queue
    /// selected by writing to QueueSel.
    legacy_queue_pfn: u32,

    /// new interface only
    queue_ready: u32,

    /// Reserved
    __r3: [u32; 2],

    /// Queue notifier
    queue_notify: u32,

    /// Reserved
    __r4: [u32; 3],

    /// Interrupt status
    interrupt_status: u32,

    /// Interrupt acknowledge
    interrupt_ack: u32,

    /// Reserved
    __r5: [u32; 2],

    /// Device status
    ///
    /// Reading from this register returns the current device status flags.
    /// Writing non-zero values to this register sets the status flags,
    /// indicating the OS/driver progress. Writing zero (0x0) to this register
    /// triggers a device reset. The device sets QueuePFN to zero (0x0) for
    /// all queues in the device. Also see 3.1 Device Initialization.
    status: u32,

    /// Reserved
    __r6: [u32; 3],

    // new interface only since here
    queue_desc_low: u32,
    queue_desc_high: u32,

    /// Reserved
    __r7: [u32; 2],

    queue_driver_low: u32,
    queue_driver_high: u32,

    /// Reserved
    __r8: [u32; 2],

    queue_device_low: u32,
    queue_device_high: u32,

    /// Reserved
    __r9: [u32; 21],

    config_generation: u32,
}
