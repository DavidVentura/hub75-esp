use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::sys::{GPIO_OUT_W1TC_REG, GPIO_OUT_W1TS_REG};
use std::thread::sleep;
use std::time::{Duration, Instant};

mod generated;
use generated::frame;

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
    let a = PinDriver::output(p.pins.gpio23).unwrap();
    let b = PinDriver::output(p.pins.gpio13).unwrap();
    let c = PinDriver::output(p.pins.gpio14).unwrap();
    let d = PinDriver::output(p.pins.gpio27).unwrap();
    let _clk = PinDriver::output(p.pins.gpio15).unwrap();
    let mut lat = PinDriver::output(p.pins.gpio25).unwrap();
    let mut oe = PinDriver::output(p.pins.gpio26).unwrap();

    let rgb1_mask: u32 = (1 << r1.pin()) | (1 << g1.pin()) | (1 << b1.pin());
    let rgb2_mask: u32 = (1 << r2.pin()) | (1 << g2.pin()) | (1 << b2.pin());
    let rgb_mask = rgb1_mask | rgb2_mask;
    println!("{rgb_mask:032b}");

    let addrmask: u32 = (1 << a.pin()) | (1 << b.pin()) | (1 << c.pin()) | (1 << d.pin());

    // ~400us per frame
    // 2 bit pwm (50% bright) = 800us
    // 5 bit color (32 color) = 4ms
    // => 250fps
    // 6+ bit color = crash with stack overflow ((

    loop {
        let _start = Instant::now();

        // enable output
        oe.set_low().unwrap();
        // silly pwm
        for brightness in [u32::MAX, 0x0] {
            for data in frame {
                for i in 0..data.len() {
                    let count = i;
                    let row = data[i];
                    oe.set_low().unwrap();
                    for x in 0..row.len() {
                        let element = row[x];
                        let br1 = element & 0b1000_0000;
                        let bg1 = element & 0b0100_0000;
                        let bb1 = element & 0b0010_0000;
                        let br2: u32 = element as u32 & 0b0001_0000;
                        let bg2: u32 = element as u32 & 0b0000_1000;
                        let bb2: u32 = element as u32 & 0b0000_0100;

                        // r1 = 3 (2 base0); r2bit = 8; 8>>5 (
                        let pixdata: u32 = ((br1 >> 5) | (bg1 >> 2) | bb1) as u32;
                        // r2 = 18; r2bit = 4; <<14;
                        // g2 = 19; g2bit = 3; <<16;
                        // b2 = 21; b2bit = 2; <<19;
                        let pixdata = pixdata | ((br2 << 14) | (bg2 << 16) | (bb2 << 19)) as u32;
                        let pixdata = pixdata & rgb_mask;
                        let pixdata = pixdata & brightness;
                        let pixdata = pixdata | (1 << 15); // clk
                        let notpixdata: u32 = (!pixdata) & rgb_mask;

                        unsafe {
                            // i _assume_ it's necessary to set clk low as its own thing to give
                            // the other pins time to settle; but i've not tested
                            core::ptr::write_volatile(GPIO_OUT_W1TC_REG as *mut _, notpixdata);
                            // pix data + clk
                            core::ptr::write_volatile(GPIO_OUT_W1TS_REG as *mut _, pixdata);
                            // clk low
                            core::ptr::write_volatile(GPIO_OUT_W1TC_REG as *mut _, 1 << 15);
                        }
                    }
                    oe.set_high().unwrap();
                    // Prevents ghosting, no idea why
                    Ets::delay_us(2);
                    lat.set_low().unwrap();
                    Ets::delay_us(2);
                    lat.set_high().unwrap();
                    // Select row

                    let ba = (count & 1) as u32;
                    let bb = (count & 2) as u32;
                    let bc = (count & 4) as u32;
                    let bd = (count & 8) as u32;
                    let addrdata: u32 = ((ba << 23) | (bb << 12) | (bc << 12) | (bd << 24)) as u32;
                    let not_addrdata: u32 = !addrdata & addrmask;
                    unsafe {
                        core::ptr::write_volatile(GPIO_OUT_W1TC_REG as *mut _, not_addrdata);
                        core::ptr::write_volatile(GPIO_OUT_W1TS_REG as *mut _, addrdata);
                    }

                    //Ets::delay_us(2);
                    //oe.set_low().unwrap();
                }
            }
        }
        // Disable the output
        // Prevents one row from being much brighter than the others
        oe.set_high().unwrap();

        println!("Elapsed {:?}", _start.elapsed());
        // keep watchdog happy
        sleep(Duration::from_millis(10));
    }
}