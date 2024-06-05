#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use core::mem::MaybeUninit;

use display_interface_spi::SPIInterface;
use embassy_executor::Spawner;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal as hal;
use hal::clock::ClockControl;
use hal::embassy;
use hal::gpio::IO;
use hal::peripherals::Peripherals;
use hal::prelude::*;
use hal::spi::master::Spi;
use hal::spi::SpiMode;
use mipidsi::Builder;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, ColorOrder};

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Debug);

    const HEAP_SIZE: usize = 128 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
        log::trace!("heap initialized");
    }

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = hal::delay::Delay::new(&clocks);

    embassy::init(
        &clocks,
        hal::timer::TimerGroup::new_async(peripherals.TIMG0, &clocks),
    );
    esp_println::println!("embassy::init embassy-time-timg0");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let cs = io.pins.gpio16.into_push_pull_output();
    let dc = io.pins.gpio17.into_push_pull_output();
    let rst = io.pins.gpio13.into_push_pull_output();
    let clk = io.pins.gpio15.into_push_pull_output();
    let mosi = io.pins.gpio14.into_push_pull_output();
    let mut bl = io.pins.gpio18.into_push_pull_output();

    bl.set_high();

    let spi = Spi::new(
        peripherals.SPI3,
        40u32.MHz(),
        SpiMode::Mode3,
        &clocks,
    );
    let spi = spi.with_sck(clk).with_mosi(mosi);
    // log::info!("spi init.");

    let spi_device = ExclusiveDevice::new(spi, cs, delay);
    let di = SPIInterface::new(spi_device, dc);
    let display = Builder::new(ST7789, di)
        .reset_pin(rst)
        .display_size(240, 280)
        .display_offset(0, 20)
        .color_order(ColorOrder::Rgb)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap();

    // spawner.spawn(bsp::wifi_start()).ok();
    spawner.spawn(hamboo::ui::run(display)).ok();
}
