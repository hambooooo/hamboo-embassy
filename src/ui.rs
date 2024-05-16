use alloc::boxed::Box;
use alloc::rc::Rc;
use display_interface::WriteOnlyDataCommand;
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;
use esp_hal::systimer::SystemTimer;
use mipidsi::Display;
use mipidsi::models::ST7789;
use slint::platform::software_renderer::{LineBufferProvider, MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel};

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
pub async fn run() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    slint::platform::set_platform(Box::new(EspBackend { window: window.clone() }))
        .expect("backend already initialized");

    let _demo = UI::new().unwrap();

    loop {
        // slint::platform::update_timers_and_animations();
        // window.draw_if_needed(|renderer| {
        //     renderer.render_by_line(DisplayWrapper{
        //         display: &mut display,
        //         line_buffer: &mut line_buffer
        //     });
        // });
        //
        // if !window.has_active_animations() {
        //     if let Some(duration) = slint::platform::duration_until_next_timer_update() {
        //         Timer::after(Duration::from_millis(duration.as_millis() as u64)).await;
        //         continue;
        //     }
        // }
        Timer::after(embassy_time::Duration::from_millis(10)).await;
    }
}