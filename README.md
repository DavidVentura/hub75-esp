## HUB75

A fast-enough bitbanged HUB75 driver on an ESP32

125 FPS on 6-bit images and 250FPS on 5-bit images.

https://github.com/DavidVentura/hub75-esp/assets/3650670/3939f794-dcc7-4f9f-943d-b73d2d7a5f91

## Image format
The image format I came up with is having the pixel for both rows packed into the same byte: `[r1_g1b1r2_g2b2]` where `_` are two ignored bits.

The ignored byte is cleverly placed in that position to reduce the amount of bit-shifting necessary, and it constrains the relationship between usable pins.

At 64x32, each RGB image takes 1KiB per bit of depth -- a 6-bit-depth image uses 6KiB. Overhead is 25% (6/8 bits are used).

The screens are 16bit -- RGB565 so going beyond 6 bit depth doesn't make sense.

As these panels are RGB111, displaying more than 1-bit color can be done by iteratively displaying each bit of the depth.
The images are composed of "frames" which use [BCM](http://www.batsocks.co.uk/readme/art_bcm_3.htm) to show higher color depth.

![image](https://github.com/DavidVentura/hub75-esp/assets/3650670/853cd795-cb46-4df6-82f5-fcf20c315052)

## Speed

Each 'frame' can be rendered by bit-banging the protocol in ~120us; but that has to be done 2<sup>bit-depth</sup> times (6 => 2<sup>6</sup> => 64) which makes each full frame take ~7.9ms.

Per my calculation the clock runs at 8.2MHz and that's probably the upper limit.

At this speed, it's reasonable to run 2 screens on 6-bit depth, or up to 4 on 5-bit depth.

## Limitations

- Only pins 0..=31 can be used for ADDR and DATA; as they are part of the port 0 and can be changed in 1 write
- This is a software implementation - each 6-bit frame takes 7.9ms to render and it must be called roughly once every 20ms to have no flicker.
	- ~Half of one core will be consumed by this
	- Using 5 bit color will bring it down to ~4ms, making it 25% of one core
- The relationship between the used pins is constrained as an optimization
	- You need to use 3 ~contiguous pins (at most a gap of 1; for me: 4, 5, 6) for R1G1B1
	- You need to use 3 ~contiguous pins (at most a gap of 1; for me: 18, 19, 21) for R2G2B2
	- You need to use 4 contiguous pins (for me: 12, 13, 14, 15) for ABCD

## Other
There's a script (`script.py`) which can convert a 64x32 PNG into a rust array, which you can copy-paste into the source.
