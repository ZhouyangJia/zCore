mod drivers;
mod sbi;
mod trap;

pub mod config;
pub mod cpu;
pub mod interrupt;
pub mod mem;
pub mod timer;
pub mod vm;

use alloc::{string::String, vec::Vec};
use core::ops::Range;
use zcore_drivers::utils::devicetree::Devicetree;

use crate::{mem::phys_to_virt, utils::init_once::InitOnce, PhysAddr};

static CMDLINE: InitOnce<String> = InitOnce::new_with_default(String::new());
static INITRD_REGION: InitOnce<Option<Range<PhysAddr>>> = InitOnce::new_with_default(None);
static MEMORY_REGIONS: InitOnce<Vec<Range<PhysAddr>>> = InitOnce::new_with_default(Vec::new());

pub fn cmdline() -> String {
    CMDLINE.clone()
}

pub fn init_ram_disk() -> Option<&'static mut [u8]> {
    INITRD_REGION.as_ref().map(|range| unsafe {
        core::slice::from_raw_parts_mut(phys_to_virt(range.start) as *mut u8, range.len())
    })
}

pub fn primary_init_early() {
    let dt = Devicetree::from(phys_to_virt(crate::KCONFIG.dtb_paddr)).unwrap();
    if let Some(cmdline) = dt.bootargs() {
        info!("Load kernel cmdline from DTB: {:?}", cmdline);
        CMDLINE.init_once_by(cmdline.into());
    }
    if let Some(time_freq) = dt.timebase_frequency() {
        info!("Load CPU clock frequency from DTB: {} Hz", time_freq);
        super::cpu::CPU_FREQ_MHZ.init_once_by((time_freq / 1_000_000) as u16);
    }
    if let Some(initrd_region) = dt.initrd_region() {
        info!("Load initrd regions from DTB: {:#x?}", initrd_region);
        INITRD_REGION.init_once_by(Some(initrd_region));
    }
    if let Ok(regions) = dt.memory_regions() {
        info!("Load memory regions from DTB: {:#x?}", regions);
        MEMORY_REGIONS.init_once_by(regions);
    }
}

pub fn primary_init() {
    vm::init();
    drivers::init().unwrap();
    timer::init();
}

pub fn secondary_init() {}
