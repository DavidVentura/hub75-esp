use esp_idf_hal::peripherals::Peripherals;
use std::time::Instant;

mod hub75;

use hub75::{Hub75, Pins};

const PANEL_ROW_H: usize = 32;
const TOTAL_IMG_W: usize = 128; //64;
const BPLANE_SIZE: usize = PANEL_ROW_H * TOTAL_IMG_W;
const BIT_DEPTH: usize = 3;
const FRAME_COUNT: usize = 40;
static FRAMES: &[u8; BIT_DEPTH * BPLANE_SIZE * FRAME_COUNT] = include_bytes!("../data/out.bin");

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
        p.pins.gpio22.into(),
        p.pins.gpio25.into(),
        p.pins.gpio26.into(),
        p.pins.gpio27.into(),
    );

    let frames = unsafe {
        &*(FRAMES.as_ptr() as *const [[[[u8; TOTAL_IMG_W]; PANEL_ROW_H]; BIT_DEPTH]; FRAME_COUNT])
    };
    let mut h = Hub75 { pins: _pins };
    loop {
        for frame in frames.iter() {
            for _ in 0..10 {
                let start = Instant::now();
                h.render(frame);
                println!("elapsed {:?}", start.elapsed());
                //std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}
