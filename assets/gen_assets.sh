#!/bin/bash

delete_assets() {
    rm icon_ios_touch_192.png
    rm icon-256.png
    rm icon-1024.png
    rm maskable_icon_x512.png
    rm favicon.ico
    rm "$1.png"
}

gen_assets() {
    if [ ! -f "icon_ios_touch_192.png" ]; then
        convert -background none "$1.svg" -resize 192x192 icon_ios_touch_192.png
    fi
    if [ ! -f "icon-256.png" ]; then
        convert -background none "$1.svg" -resize 256x256 icon-256.png
    fi
    if [ ! -f "icon-1024.png" ]; then
        convert -background none "$1.svg" -resize 1024x1024 icon-1024.png
    fi
    if [ ! -f "maskable_icon_x512.png" ]; then
        convert -background none "$1.svg" -resize 512x512 maskable_icon_x512.png
    fi

    # https://golb.n4n5.dev/utils-linux.html#one-liner-faviconico-generator

    TO_ICONIFY="$1.svg"
    if [ ! -f "favicon.ico" ]; then
        for i in 48 96 144 192; do convert -background none $TO_ICONIFY -resize ${i}x${i} favicon-${i}x${i}.png; done
        convert -background none favicon-* favicon.ico
        rm favicon-*
    fi

    for i in *.svg; do
        if [ "$i" = "$1.svg" ]; then
            continue
        fi
        if [ ! -f "${i%.svg}.png" ]; then
            convert -background none "$i" -resize 64x64 "${i%.svg}.png"
        fi
    done

    if [ ! -f "$1.png" ]; then
        convert -background none "$1.svg" -resize 512x512 "$1.png"
    fi
}

if [ "$1" == "delete" ]; then
    delete_assets "tarsier"
else
    gen_assets "tarsier"
fi

# The line beneath this is called `modeline`. See `:help modeline`
# vim: ts=4 sts=4 sw=4 et
