use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::sys::{GPIO_OUT_W1TC_REG, GPIO_OUT_W1TS_REG};
use std::thread::sleep;
use std::time::{Duration, Instant};

mod generated;
use generated::frames;

#[link_section = ".iram1"]
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take().unwrap();

    //let image = include_bytes!("../out.bin");

    // must be pins 0..=31 as pins 32..=39 are controlled by another register
    let r1 = PinDriver::output(p.pins.gpio2).unwrap();
    let g1 = PinDriver::output(p.pins.gpio4).unwrap();
    let b1 = PinDriver::output(p.pins.gpio5).unwrap();
    let r2 = PinDriver::output(p.pins.gpio18).unwrap();
    let g2 = PinDriver::output(p.pins.gpio19).unwrap();
    let b2 = PinDriver::output(p.pins.gpio21).unwrap();

    let a = PinDriver::output(p.pins.gpio12).unwrap();
    let b = PinDriver::output(p.pins.gpio13).unwrap();
    let c = PinDriver::output(p.pins.gpio14).unwrap();
    let d = PinDriver::output(p.pins.gpio15).unwrap();

    let clk = PinDriver::output(p.pins.gpio25).unwrap();
    let lat = PinDriver::output(p.pins.gpio26).unwrap();
    let oe = PinDriver::output(p.pins.gpio27).unwrap();

    let lat_pin = lat.pin() as u8;
    let oe_pin = oe.pin() as u8;

    let rgb1_mask: u32 = (1 << r1.pin()) | (1 << g1.pin()) | (1 << b1.pin());
    let rgb2_mask: u32 = (1 << r2.pin()) | (1 << g2.pin()) | (1 << b2.pin());
    let rgb_mask = rgb1_mask | rgb2_mask;

    let addrmask: u32 = (1 << a.pin()) | (1 << b.pin()) | (1 << c.pin()) | (1 << d.pin());

    let clkpin = clk.pin() as u8;

    loop {
        for frame in frames.iter() {
            // make frames last longer
            for _ in 0..5 {
                //let _start = Instant::now();
                // enable output
                fast_pin_down(oe_pin);
                let mut bit_nr = frame.len();
                for data in frame {
                    // this is a simple binary coded modulation which gives more time on for more
                    // significant bits; explanation in littel-endian ([n, n-1, .., 1, 0])
                    //
                    // to modulate different colors on RGB111,
                    // bit n     gets 2^(n  ) frames ON/OFF
                    // bit (n-1) gets 2^(n-2) frames ON/OFF
                    // ...
                    // bit (0)   gets 2^(0  ) frames ON/OFF
                    //
                    // this makes the MSB have a larger impact in perception, simulating more color
                    // resolution by utilizing the time factor instead of variable brightness
                    let tot_frames = 1 << (bit_nr - 1);
                    for _ in 0..tot_frames {
                        for (i, row) in data.iter().enumerate() {
                            fast_pin_down(oe_pin);
                            for element in row.iter() {
                                let rgb1 = *element as u32 & 0b1101_0000;
                                let rgb2 = *element as u32 & 0b0000_1011;

                                let pixdata: u32 = rgb1 >> 2;
                                let pixdata = pixdata | (rgb2 << 18) | (1 << clkpin);
                                let notpixdata: u32 = ((!pixdata) & rgb_mask) | (1 << clkpin);

                                // set some pixel values & clock _down_
                                fast_pin_clear(notpixdata);
                                // set remaining pixel values & clock _up_
                                fast_pin_set(pixdata);
                            }
                            fast_pin_up(oe_pin);
                            fast_pin_down(lat_pin);
                            fast_pin_up(lat_pin);

                            let addrdata: u32 = (i as u32) << 12;
                            let not_addrdata: u32 = !addrdata & addrmask;
                            fast_pin_clear(not_addrdata);
                            fast_pin_set(addrdata);
                        }
                    }
                    bit_nr -= 1;
                }
                // Disable the output
                // Prevents one row from being much brighter than the others
                fast_pin_up(oe_pin);
            }
        }
    }
}

#[inline]
fn fast_pin_set(pins: u32) {
    unsafe {
        core::ptr::write_volatile(GPIO_OUT_W1TS_REG as *mut _, pins);
    }
}
#[inline]
fn fast_pin_clear(pins: u32) {
    unsafe {
        core::ptr::write_volatile(GPIO_OUT_W1TC_REG as *mut _, pins);
    }
}

#[inline]
fn fast_pin_up(idx: u8) {
    unsafe {
        core::ptr::write_volatile(GPIO_OUT_W1TS_REG as *mut _, 1 << idx);
    }
}
#[inline]
fn fast_pin_down(idx: u8) {
    unsafe {
        core::ptr::write_volatile(GPIO_OUT_W1TC_REG as *mut _, 1 << idx);
    }
}
