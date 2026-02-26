use acpi::{
    AcpiError, AcpiTables,
    aml::Interpreter,
    platform::AcpiPlatform,
    registers::FixedRegisters,
    sdt::{
        fadt::Fadt,
        mcfg::{Mcfg, McfgEntry},
        spcr::Spcr,
    },
};
use alloc::sync::Arc;
use limine::request::RsdpRequest;
use spin::lazy::Lazy;

use crate::{
    mem::{PhysAddr, VirtAddr},
    platform::mem::virt_to_phys,
};
use handler::AcpiHandler;

mod handler;
pub mod power;

#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

pub static ACPI: Lazy<Acpi> = Lazy::new(|| init_acpi().unwrap());

#[derive(Debug)]
pub struct PcieInfo {
    pub paddr: PhysAddr,
    pub length: usize,
}

impl PcieInfo {
    pub fn get() -> &'static Self {
        &ACPI.pcie_info
    }
}

#[allow(dead_code)]
pub struct Acpi {
    pub aml_engine: Interpreter<AcpiHandler>,
    pub serial_base: u64,
    pub fadt: Fadt,
    pub registers: Arc<FixedRegisters<AcpiHandler>>,
    pub pcie_info: PcieInfo,
}

unsafe impl Send for Acpi {}
unsafe impl Sync for Acpi {}

fn init_acpi() -> Result<Acpi, AcpiError> {
    let response = RSDP_REQUEST.get_response().unwrap();

    let platform_info = unsafe {
        let rsdp_address = response.address() as VirtAddr;
        let tables = AcpiTables::from_rsdp(AcpiHandler, virt_to_phys(rsdp_address))?;
        AcpiPlatform::new(tables, AcpiHandler)?
    };

    let aml_engine = Interpreter::new_from_platform(&platform_info)?;

    let acpi_tables = &platform_info.tables;
    let spcr = acpi_tables.find_table::<Spcr>().unwrap();
    let fadt = *acpi_tables.find_table::<Fadt>().unwrap();
    let mcfg = acpi_tables.find_table::<Mcfg>().unwrap();
    let entries = mcfg.entries();
    let paddr = virt_to_phys(entries.as_ptr() as VirtAddr);
    let length = size_of_val(entries);

    Ok(Acpi {
        aml_engine,
        serial_base: spcr.base_address().unwrap()?.address,
        fadt,
        registers: platform_info.registers.clone(),
        pcie_info: PcieInfo { paddr, length },
    })
}
