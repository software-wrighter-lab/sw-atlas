# Image assets

| File | What it is |
|---|---|
| `sw-atlas-logo.jpg` | Source of record. 1024x1024, as supplied. Never served. |
| `sw-atlas-logo.png` | The web copy: rounded corners with a real alpha channel, 320x320, 255 colours, 49 KB. This is what the README references. |

Regenerate the web copy from the source with ImageMagick (no Python, per
`CLAUDE.md`):

```sh
magick images/sw-atlas-logo.jpg \
  \( +clone -alpha transparent -fill white \
     -draw "roundrectangle 0,0 1023,1023 150,150" \) \
  -compose CopyOpacity -composite \
  -resize 320x320 -strip -colors 255 -define png:compression-level=9 \
  images/sw-atlas-logo.png
```

The corner radius is 150 of 1024, about 14%, chosen against the artwork
rather than by formula: the badge inside is a circle, and a tighter radius
left the square reading as a box while a looser one crowded the glow ring.
Quantising to 255 colours takes the file from 195 KB to 49 KB with no
visible difference at the size the README displays it.
