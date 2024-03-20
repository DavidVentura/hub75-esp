use esp_idf_hal::gpio::{AnyOutputPin, Output, Pin, PinDriver};
use esp_idf_hal::sys::{GPIO_OUT_W1TC_REG, GPIO_OUT_W1TS_REG};

/// This struct takes ownership of the necessary output pins
/// but writes directly to them in batches, so they are not used
pub struct Pins<'d> {
    oe_pin: u8,
    lat_pin: u8,
    clk_pin: u8,
    rgb_mask: u32,
    addr_mask: u32,
    _r1: PinDriver<'d, AnyOutputPin, Output>,
    _g1: PinDriver<'d, AnyOutputPin, Output>,
    _b1: PinDriver<'d, AnyOutputPin, Output>,
    _r2: PinDriver<'d, AnyOutputPin, Output>,
    _g2: PinDriver<'d, AnyOutputPin, Output>,
    _b2: PinDriver<'d, AnyOutputPin, Output>,
    _a: PinDriver<'d, AnyOutputPin, Output>,
    _b: PinDriver<'d, AnyOutputPin, Output>,
    _c: PinDriver<'d, AnyOutputPin, Output>,
    _d: PinDriver<'d, AnyOutputPin, Output>,
    _clk: PinDriver<'d, AnyOutputPin, Output>,
    _lat: PinDriver<'d, AnyOutputPin, Output>,
    _oe: PinDriver<'d, AnyOutputPin, Output>,
}

impl<'d> Pins<'d> {
    /// The pins must be 0..=31 to be part of the control register 0.
    /// * A, B, C, D must be contiguous
    /// * R1, G1, B1 must be n, n+2, n+3 (2, 4, 5)
    /// * R2, G2, B2 must be n, n+1, n+3 (18, 19, 21)
    pub fn new(
        r1: AnyOutputPin,
        g1: AnyOutputPin,
        b1: AnyOutputPin,
        r2: AnyOutputPin,
        g2: AnyOutputPin,
        b2: AnyOutputPin,
        a: AnyOutputPin,
        b: AnyOutputPin,
        c: AnyOutputPin,
        d: AnyOutputPin,
        clk: AnyOutputPin,
        lat: AnyOutputPin,
        oe: AnyOutputPin,
    ) -> Pins<'d> {
        assert_eq!(b.pin(), a.pin() + 1);
        assert_eq!(c.pin(), b.pin() + 1);
        assert_eq!(d.pin(), c.pin() + 1);

        assert_eq!(g1.pin(), r1.pin() + 2);
        assert_eq!(b1.pin(), g1.pin() + 1);

        assert_eq!(g2.pin(), r2.pin() + 1);
        assert_eq!(b2.pin(), g2.pin() + 2);

        for p in [
            r1.pin(),
            g1.pin(),
            b1.pin(),
            r2.pin(),
            g2.pin(),
            b2.pin(),
            a.pin(),
            b.pin(),
            c.pin(),
            d.pin(),
            clk.pin(),
            lat.pin(),
            oe.pin(),
        ] {
            assert!(p < 32);
        }

        let rgb1_mask: u32 = (1 << r1.pin()) | (1 << g1.pin()) | (1 << b1.pin());
        let rgb2_mask: u32 = (1 << r2.pin()) | (1 << g2.pin()) | (1 << b2.pin());
        let rgb_mask = rgb1_mask | rgb2_mask;

        let addr_mask: u32 = (1 << a.pin()) | (1 << b.pin()) | (1 << c.pin()) | (1 << d.pin());

        let _r1 = PinDriver::output(r1).unwrap();
        let _g1 = PinDriver::output(g1).unwrap();
        let _b1 = PinDriver::output(b1).unwrap();
        let _r2 = PinDriver::output(r2).unwrap();
        let _g2 = PinDriver::output(g2).unwrap();
        let _b2 = PinDriver::output(b2).unwrap();
        let _a = PinDriver::output(a).unwrap();
        let _b = PinDriver::output(b).unwrap();
        let _c = PinDriver::output(c).unwrap();
        let _d = PinDriver::output(d).unwrap();
        let _clk = PinDriver::output(clk).unwrap();
        let _lat = PinDriver::output(lat).unwrap();
        let _oe = PinDriver::output(oe).unwrap();
        Pins {
            oe_pin: _oe.pin() as u8,
            lat_pin: _lat.pin() as u8,
            clk_pin: _clk.pin() as u8,
            rgb_mask,
            addr_mask,
            _r1,
            _g1,
            _b1,
            _r2,
            _g2,
            _b2,
            _a,
            _b,
            _c,
            _d,
            _clk,
            _lat,
            _oe,
        }
    }
}
pub struct Hub75<'d> {
    pub pins: Pins<'d>,
}

/// Represents a 64x32 image in RGB111 format.
/// The depth of the data in a Frame maps to the bit depth of the resulting image.
///
/// Each Frame is composed of 16 Rows of pixel data; though each Row is rendered to
/// multiple rows on the display in parallel.
///
/// A given Row R is composed of 64 bytes, which represent a single bit on the R, G and B channels
/// for two pixels; the one at R and the one at R+16.
pub type Frame = [[[u8; 64]; 16]];

impl<'d> Hub75<'d> {
    /// Render a Frame using binary coded modulation (BCM) which displays the
    /// more significant bits for a longer time
    ///
    /// ```
    /// bit n     is displayed for 2^(n  ) frames
    /// bit (n-1) is displayed for 2^(n-2) frames
    /// ...
    /// bit (0)   is displayed for 2^(0  ) frames
    /// ```
    ///
    /// Doing this allows to represent a larger color spectrum, at the cost of lower perceived
    /// FPS;
    ///
    /// Each frame is displayed for ~120us, so on a 6-bit depth image:
    /// ```
    /// bit 5: 3840us
    /// bit 4: 1920us
    /// bit 3:  960us
    /// bit 2:  480us
    /// bit 1:  240us
    /// bit 0:  120us
    ///
    /// total frame time: 7560us (7.5ms)
    /// ```
    ///
    /// At 6-bit depth, it's possible to render 64x64, but 128x64 requires going to 5-bit.
    #[link_section = ".iram1"]
    pub fn render(&self, data: &Frame) {
        let oe_pin = self.pins.oe_pin;
        let clkpin = self.pins.clk_pin;
        let lat_pin = self.pins.lat_pin;
        let rgb_mask = self.pins.rgb_mask;
        let addrmask = self.pins.addr_mask;

        // enable output
        fast_pin_down(oe_pin);
        let mut bit_nr = data.len() - 1;
        for data in data {
            let tot_frames = 1 << bit_nr;
            for _ in 0..tot_frames {
                for (i, row) in data.iter().enumerate() {
                    fast_pin_down(oe_pin);
                    for element in row.iter() {
                        // BGR BGR
                        let rgb1 = *element as u32 & 0b1101_0000;
                        let rgb2 = *element as u32 & 0b0000_1011;
                        let r1 = rgb1 & (1 << 4);
                        let g1 = rgb1 & (1 << 6);
                        let b1 = rgb1 & (1 << 7);

                        let r2 = rgb2 & (1 << 0);
                        let g2 = rgb2 & (1 << 1);
                        let b2 = rgb2 & (1 << 3);

                        let rgb = (r1 >> 2)
                            | (g1 >> 2)
                            | (b1 >> 2)
                            | (r2 << 18)
                            | (g2 << 18)
                            | (b2 << 18);
                        let not_rgb = !rgb & rgb_mask;
                        fast_pin_clear(not_rgb | (1 << clkpin));
                        fast_pin_set((rgb & rgb_mask) | (1 << clkpin));
                    }

                    fast_pin_up(oe_pin);
                    fast_pin_down(lat_pin);
                    fast_pin_up(lat_pin);

                    // TODO: 12 hardcoded
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
