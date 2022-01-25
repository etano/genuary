#!/bin/bash

ffmpeg -y -i gen/%03d.png -vf palettegen gen/palette.png
ffmpeg -r 60 -i gen/%03d.png -i gen/palette.png -pix_fmt yuv420p gen/output.mp4
