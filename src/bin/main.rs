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
use esp_hal::delay::Delay;
use esp_hal::time::Duration;
use esp_hal::{main, uart};
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
    let peripherals = esp_hal::init(config);

    // pinout: Radio Pin # -> ESP32 Pin #
    // 6 Vcc -> ESP32 3v3
    // 7 Gnd -> ESP32 GND
    // 3 SDO -> ESP32 4 (UART Rx)
    // 4 SDI -> ESP32 5 (UART Tx)
    //
    // Serial data rate default 19.2kbps

    let mut uart0 = match uart::Uart::new(
        peripherals.UART1,
        uart::Config::default().with_baudrate(19200),
    ) {
        Ok(val) => val,
        Err(err) => panic!("Failed to initialize uart: {:?}", err),
    }
    .with_rx(peripherals.GPIO4)
    .with_tx(peripherals.GPIO5);

    let delay = Delay::new();
    loop {
        delay.delay(Duration::from_millis(1000));
        match get_radio_uart_speed(&mut uart0) {
            Ok(speed) => info!("Got uart speed of {}bps", speed),
            Err(UartError::RxError(err)) => warn!("Failed to get uart speed: {:?}", err),
            Err(UartError::TxError(err)) => warn!("Failed to get uart speed: {:?}", err),
        }
    }
}

#[derive(Debug)]
enum UartError {
    RxError(uart::RxError),
    TxError(uart::TxError),
}

impl From<uart::TxError> for UartError {
    fn from(value: uart::TxError) -> Self {
        Self::TxError(value)
    }
}

impl From<uart::RxError> for UartError {
    fn from(value: uart::RxError) -> Self {
        Self::RxError(value)
    }
}

fn get_radio_uart_speed<'d>(
    uart0: &mut uart::Uart<'d, esp_hal::Blocking>,
) -> Result<usize, UartError> {
    let mut buf = [0u8; 256];
    info!("Sending command...");
    let bytes_written = uart0.write(b"ER_CMD#U?")?;
    info!("Wrote {} bytes, Flushing buffer...", bytes_written);
    uart0.flush()?;
    info!("Buffer flushed, waiting for response...");
    let bytes_read = uart0.read(&mut buf)?;
    info!("Got response...");
    if bytes_read == 0 {
        panic!("Empty response received.");
    }
    let written = &buf[..bytes_read];
    if written.starts_with(b"ER_CMD#U") {
        match written[8] {
            b'1' => Ok(2400),
            b'2' => Ok(4800),
            b'3' => Ok(9600),
            b'4' => Ok(19200),
            b'5' => Ok(38400),
            b'6' => Ok(31250),
            b'7' => Ok(76800),
            b'8' => Ok(115200),
            _ => panic!("Unknown response: {:?}", &buf[..bytes_read]),
        }
    } else {
        panic!("Unknown response: {:?}", &buf[..bytes_read]);
    }
}
