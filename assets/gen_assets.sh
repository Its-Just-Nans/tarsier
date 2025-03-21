#!/bin/bash

convert -background none tarsier.svg -resize 192x192 icon_ios_touch_192.png
convert -background none tarsier.svg -resize 256x256 icon-256.png
convert -background none tarsier.svg -resize 1024x1024 icon-1024.png
convert -background none tarsier.svg -resize 512x512 maskable_icon_x512.png

# https://golb.n4n5.dev/utils-linux.html#one-liner-faviconico-generator

TO_ICONIFY=tarsier.svg
for i in 48 96 144 192; do convert -background none $TO_ICONIFY -resize ${i}x${i} favicon-${i}x${i}.png; done
convert -background none favicon-* favicon.ico
rm favicon-*

for i in *.svg; do
    if [ ! -f "${i%.svg}.png" ]; then
        convert -background none "$i" -resize 64x64 "${i%.svg}.png"
    fi
done
