from PIL import Image
import glob

# fmt: off
lut = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
       1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4,
       4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11,
       12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22,
       22, 23, 24, 24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37,
       38, 39, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58,
       59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72, 73, 74, 75, 77, 78, 79, 81, 82, 83, 85,
       86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104, 105, 107, 109, 110, 112, 114,
       115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137, 138, 140, 142, 144,
       146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175, 177, 180,
       182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
       223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,]

out = bytearray()

i = 0

bitdepth = 6
bitmasks = []
for i in range(bitdepth, 0, -1):
    bitmasks.append(1 << (i+(7-bitdepth)))

for fname in sorted(glob.glob("/home/david/nyan/*png")):
    if i > 1111:
        break
    with Image.open(fname) as im:
        width, height = im.size
        im = im.convert("RGB")

    assert height == 32

    for mask in bitmasks:
        for y in range(0, height//2): # need to take 2 pix at once
            for x in range(0, width):
                r1, g1, b1 = im.getpixel((x, y))
                r2, g2, b2 = im.getpixel((x, y+16))

                r1 = lut[r1]
                g1 = lut[g1]
                b1 = lut[b1]
                r2 = lut[r2]
                g2 = lut[g2]
                b2 = lut[b2]

                r1 = 0b0001_0000 if r1 & mask else 0
                g1 = 0b0100_0000 if g1 & mask else 0
                b1 = 0b1000_0000 if b1 & mask else 0
                r2 = 0b0000_0001 if r2 & mask else 0
                g2 = 0b0000_0010 if g2 & mask else 0
                b2 = 0b0000_1000 if b2 & mask else 0


                outpix = r1 | g1 | b1 | r2 | g2 | b2
                out.append(outpix)
    i+=1

open('data/out.bin', 'wb').write(out)
