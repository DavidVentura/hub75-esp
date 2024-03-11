use esp_idf_hal::peripherals::Peripherals;

mod hub75;

use hub75::{Hub75, Pins};

static FRAMES: &[u8; 675840] = include_bytes!("../data/out.bin");

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take().unwrap();

    let _pins = Pins::new(
        p.pins.gpio2.into(),
        p.pins.gpio4.into(),
        p.pins.gpio5.into(),
        p.pins.gpio18.into(),
        p.pins.gpio19.into(),
        p.pins.gpio21.into(),
        p.pins.gpio12.into(),
        p.pins.gpio13.into(),
        p.pins.gpio14.into(),
        p.pins.gpio15.into(),
        p.pins.gpio25.into(),
        p.pins.gpio26.into(),
        p.pins.gpio27.into(),
    );

    let frames = unsafe { &*(FRAMES.as_ptr() as *const [[[[u8; 64]; 16]; 6]; 110]) };
    let h = Hub75 { pins: _pins };
    loop {
        for frame in frames.iter() {
            for _ in 0..5 {
                h.render(frame);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}
