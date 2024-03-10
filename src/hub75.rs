use esp_idf_hal::gpio::{AnyOutputPin, Output, Pin, PinDriver};
use esp_idf_hal::sys::{GPIO_OUT_W1TC_REG, GPIO_OUT_W1TS_REG};

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
    /// * Must be pins 0..=31 as pins 32..=39 are controlled by another register
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
        let rgb1_mask: u32 = (1 << r1.pin()) | (1 << g1.pin()) | (1 << b1.pin());
        let rgb2_mask: u32 = (1 << r2.pin()) | (1 << g2.pin()) | (1 << b2.pin());
        let rgb_mask = rgb1_mask | rgb2_mask;

        let addr_mask: u32 = (1 << a.pin()) | (1 << b.pin()) | (1 << c.pin()) | (1 << d.pin());

        assert_eq!(b.pin(), a.pin() + 1);
        assert_eq!(c.pin(), b.pin() + 1);
        assert_eq!(d.pin(), c.pin() + 1);

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

impl<'d> Hub75<'d> {
    #[link_section = ".iram1"]
    pub fn render(&self, data: &[[[u8; 64]; 16]; 6]) {
        let oe_pin = self.pins.oe_pin;
        let clkpin = self.pins.clk_pin;
        let lat_pin = self.pins.lat_pin;
        let rgb_mask = self.pins.rgb_mask;
        let addrmask = self.pins.addr_mask;

        //let _start = Instant::now();
        // enable output
        fast_pin_down(oe_pin);
        let mut bit_nr = data.len();
        for data in data {
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

                        // TODO 2/18 hardcoded
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
