#!/bin/bash

set -x
set -e

ffmpeg -y \
    -i input.mp4 \
    -ss 00:00:04.80 \
    -to 00:00:07 \
    -filter:v "crop=in_w:in_h/2:0:in_h/8" \
    -pix_fmt yuv420p \
    -c:a copy \
    input.cut.mp4
ffmpeg -y \
    -i input.cut.mp4 \
    -filter_complex "[0:v]reverse,split=1[r1];[0:v][r1] concat=n=2:v=1[v]" \
    -map "[v]" \
    input.cut.reversed.mp4
ffmpeg -y \
    -i input.cut.reversed.mp4 \
    -i palette.png \
    -filter_complex "scale=400:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=4[p];[s1][p]paletteuse=dither=bayer:bayer_scale=0" \
    -loop 0 \
    -r 30 \
    output.gif
