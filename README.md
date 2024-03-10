## HUB75

Attempt for a fast-enough bitbanged HUB75 driver on an ESP32

## Panel
As these panels are RGB111, displaying more than 1-bit color can be done by iteratively displaying each bit of the depth.
On these panels, you need to provide 2 rows of data at once: x and x+16; both halves of the panel are filled simultaneously.

## Image format
The image format I came up with is having the pixel for both rows packed into the same byte: `[r1g1b1r2g2b2__]` where `_` are two ignored bytes.

At 64x32, each RGB image takes 1KiB per bit of depth -- a normal image with 8bit depth takes 8KiB. Overhead is 25% (6/8 bits are used).

## Speed

Each 'frame' can be rendered by bit-banging the protocol in ~300us; but that has to be multiplied with the bit depth (5 => 2^5 => 32) and the brightness PWM factor (currently disabled) which makes each full frame take ~9.6ms.

It's likely that with a more clever pin arrangement this can go down; but it's probably better to do this with the I2S peripheral at that point.


## Other

There's a script (`script.py`) which can convert a 64x32 PNG into a rust array, which you can copy-paste into the source.
