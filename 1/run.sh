#!/bin/bash

cargo build --release
ffmpeg -i gen/%03d.png -y -c:v libx264 -an -preset veryslow -flags +cgop -profile:v high -level 4.0 -b:v "2048k" -movflags +faststart -r 60 -pix_fmt yuv420p -vf "format=gray" gen/output.mp4
