#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::{info, warn};
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::riscv::register::{marchid, mvendorid};
use panic_rtt_target as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    let vendorid = unsafe { *(0xF11 as *const u32) };
    let proper_id = mvendorid::read();
    if proper_id.bits() != vendorid as usize {
        warn!("Vendor id fetched through different methods differs");
    }

    info!("Raw vendor id is {:x}", vendorid);
    info!(
        "Proper VendorID contains {} continuation bytes",
        proper_id.offset()
    );

    let mut out_id = [0u8; 4];
    for (b, idx) in proper_id.jedec_manufacturer().zip(0..).take(4) {
        out_id[idx] = b;
    }
    info!("Proper vendor id {:x}", out_id);

    let archid = unsafe { *(0xF12 as *const u32) };
    let proper_arch = marchid::read();
    if archid as usize != proper_arch.bits() {
        warn!("Arch id fetched through different methods differs");
    }
    info!("Marchid via pointer: {:x}", archid);
    info!("Marchid via lib: {:x}", proper_arch.bits());
    panic!("reached end");
}
