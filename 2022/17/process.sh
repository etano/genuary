#!/bin/bash

ffmpeg -y -i gen/%03d.png -vf fps=60,scale=flags=lanczos,palettegen gen/palette.png
ffmpeg -framerate 60 -i gen/%03d.png -i gen/palette.png -filter_complex "fps=60,scale=flags=lanczos[x];[x][1:v]paletteuse" -y -c:v libx264 -crf 18 -an -preset veryslow -flags +cgop -profile:v high -level 4.0 -b:v "2048k" -movflags +faststart -pix_fmt yuv420p gen/output.mp4

