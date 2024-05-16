extern crate alloc;

use core::mem::MaybeUninit;

use esp_hal as hal;
use hal::system::SystemExt;
use hal::clock::ClockControl;
use hal::embassy;
use hal::peripherals::Peripherals;
use esp_backtrace as _;
pub use hal::prelude::main;
pub use hal::prelude::entry;


#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

pub struct HambooBsp {
    // i2c_bus: I2C,
    // display: Display,
    // touch: Touch,
    // rtc: RTC,

}

impl HambooBsp {
    pub fn new() -> Self {
        esp_println::logger::init_logger(log::LevelFilter::Debug);
        Self::init_heap();

        let peripherals = Peripherals::take();
        let system = peripherals.SYSTEM.split();
        let clocks = ClockControl::max(system.clock_control).freeze();

        embassy::init(
            &clocks,
            hal::timer::TimerGroup::new_async(peripherals.TIMG0, &clocks),
        );
        esp_println::println!("embassy::init embassy-time-timg0");

        Self {}
    }

    fn init_heap() {
        const HEAP_SIZE: usize = 128 * 1024;
        static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

        unsafe {
            ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
            log::trace!("heap initialized");
        }
    }
}