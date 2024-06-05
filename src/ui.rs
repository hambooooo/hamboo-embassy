use alloc::boxed::Box;
use alloc::rc::Rc;
use cst816s::CST816S;

use display_interface::WriteOnlyDataCommand;
use display_interface_spi::SPIInterface;
use embassy_time::{Duration, Timer};
use embedded_graphics::prelude::OriginDimensions;
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::i2c::RefCellDevice;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::gpio::{GpioPin, Input, Output, PullUp, PushPull};
use esp_hal::i2c::I2C;
use esp_hal::peripherals::{I2C1, SPI3};
use esp_hal::spi::FullDuplexMode;
use esp_hal::spi::master::Spi;
use esp_hal::systimer::SystemTimer;
use mipidsi::Display;
use mipidsi::models::ST7789;
use slint::platform::software_renderer::{
    LineBufferProvider,
    MinimalSoftwareWindow,
    RepaintBufferType,
    Rgb565Pixel,
};
use slint::platform::WindowEvent;

slint::include_modules!();

struct EspBackend {
    window: Rc<MinimalSoftwareWindow>,
}

impl slint::platform::Platform for EspBackend {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> core::time::Duration {
        core::time::Duration::from_millis(
            SystemTimer::now() / (SystemTimer::TICKS_PER_SECOND / 1000),
        )
    }

    fn debug_log(&self, arguments: core::fmt::Arguments) {
        esp_println::println!("{}", arguments);
    }
}

struct DrawBuffer<'a, Display> {
    display: Display,
    buffer: &'a mut [Rgb565Pixel],
}

impl<DI, RST> LineBufferProvider for &mut DrawBuffer<'_, Display<DI, ST7789, RST>>
    where
        DI: WriteOnlyDataCommand,
        RST: OutputPin<Error=core::convert::Infallible>,
{
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Rgb565Pixel]),
    ) {
        let buffer = &mut self.buffer[range.clone()];

        render_fn(buffer);

        // We send empty data just to get the device in the right window
        self.display.set_pixels(
            range.start as u16,
            line as _,
            range.end as u16,
            line as u16,
            buffer.iter().map(|x| embedded_graphics::pixelcolor::raw::RawU16::new(x.0).into()),
        ).unwrap();
    }
}

#[embassy_executor::task]
pub async fn run(
    display: Display<SPIInterface<ExclusiveDevice<Spi<'static, SPI3, FullDuplexMode>, GpioPin<Output<PushPull>, 16>, Delay>, GpioPin<Output<PushPull>, 17>>, ST7789, GpioPin<Output<PushPull>, 13>>,
    mut touch: CST816S<RefCellDevice<'static, I2C<'static, I2C1, Blocking>>, GpioPin<Input<PullUp>, 9>, GpioPin<Output<PushPull>, 10>>
) {
    let mut buffer_provider = DrawBuffer {
        display,
        buffer: &mut [Rgb565Pixel(0); 240],
    };

    let size = slint::PhysicalSize::new(240, 280);
    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    window.set_size(size);
    slint::platform::set_platform(Box::new(EspBackend { window: window.clone() }))
        .expect("backend already initialized");

    let _ui = UI::new().unwrap();

    loop {
        slint::platform::update_timers_and_animations();
        window.draw_if_needed(|renderer| {
            renderer.render_by_line(&mut buffer_provider);
        });

        let button = slint::platform::PointerEventButton::Left;
        if let Some(event) = touch.read_one_touch_event(true).map(|record| {
            let position = slint::PhysicalPosition::new(record.x as _, record.y as _)
                .to_logical(window.scale_factor());
            esp_println::println!("{:?}", record);
            match record.action {
                0 => WindowEvent::PointerPressed { position, button },
                1 => WindowEvent::PointerReleased { position, button },
                2 => WindowEvent::PointerMoved { position },
                _ => WindowEvent::PointerExited,
            }
        }) {
            esp_println::println!("{:?}", event);
            let is_pointer_release_event: bool =
                matches!(event, WindowEvent::PointerReleased { .. });
            window.dispatch_event(event);

            // removes hover state on widgets
            if is_pointer_release_event {
                window.dispatch_event(WindowEvent::PointerExited);
            }
        }

        if !window.has_active_animations() {
            if let Some(duration) = slint::platform::duration_until_next_timer_update() {
                Timer::after(Duration::from_millis(duration.as_millis() as u64)).await;
                continue;
            }
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}