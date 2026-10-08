# FreokRO startup video

Place `intro.ivf` in this directory. The client plays it in a loop on the login and character selection screens. The FreokRO identity is already part of the video, so the client draws no separate logo. If the file is absent or invalid, the client shows a dark background. Game maps load only after a character enters the world.

The built-in decoder requires **AV1 video in an IVF container**. An MP4 cannot be copied here unchanged. The current eight-second, 1280 × 720, 20 fps video was converted from [`docs/assets/freokro-intro-source.mp4`](../../../docs/assets/freokro-intro-source.mp4) with FFmpeg:

```powershell
ffmpeg -i intro.mp4 -an -c:v libaom-av1 -cpu-used 6 -crf 32 -b:v 0 -pix_fmt yuv420p -f ivf intro.ivf
```

Keep the clip short (about 5–15 seconds) and export it without audio. The client loads the compressed video into memory at startup, and the existing game music remains separate. The accepted maximum resolution is 3840 × 2160; 1280 × 720 is a good starting point for quick launches.
