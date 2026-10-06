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
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{LSGlobalClkSource, LowSpeed, channel, timer};
use esp_hal::main;
use esp_hal::time::{Duration, Instant, Rate};
use esp_hal::timer::PeriodicTimer;
use esp_hal::timer::timg::TimerGroup;
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
    // generator version: 1.4.0
    // generator parameters: -o esp32c6 -o probe-rs -o defmt -o panic-rtt-target -o helix -o vscode

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut ledc = esp_hal::ledc::Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(esp_hal::ledc::timer::Number::Timer0);
    if let Err(e) = lstimer0.configure(timer::config::Config {
        duty: timer::config::Duty::Duty5Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: Rate::from_khz(24),
    }) {
        panic!("Failed to configure lstimer: {:?}", e);
    };

    let mut channel0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO8);
    if let Err(e) = channel0.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 10,
        drive_mode: esp_hal::gpio::DriveMode::PushPull,
    }) {
        panic!("Failed to configure channel: {:?}", e);
    };

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let mut periodic = PeriodicTimer::new(timg0.timer0);
    let mut duty_pct = 0;
    let mut duty_change = 10i8;
    loop {
        match channel0.set_duty(duty_pct) {
            Ok(()) => info!("set duty pct {}", &duty_pct),
            Err(e) => warn!("failed to set duty pct {:?}", e),
        };

        match periodic.start(Duration::from_secs(1)) {
            Ok(()) => {}
            Err(e) => panic!("failed to start timer: {:?}", e),
        }
        periodic.wait();

        duty_pct = (duty_pct as i8 + duty_change) as u8;
        if duty_pct >= 90 || duty_pct <= 0 {
            duty_change = -duty_change;
        }
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
