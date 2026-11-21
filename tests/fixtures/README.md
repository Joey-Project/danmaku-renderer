# Golden inputs

| Bilibili video | Fixture basename | XML comments |
| --- | --- | ---: |
| [芯片能效横评_v5](https://www.bilibili.com/video/BV1kMxM6FECM/) | chip-efficiency | 10,465 |
| [终将成为你 舞台剧](https://www.bilibili.com/video/BV1QJ411q7Sr/) | bloom-into-you-stage | 9,331 |

Each XML fixture contains comment text, playback time, type, source font size and RGB color in the original comment order. The samples omit Bilibili account hashes, comment IDs and other source metadata. The ASS snapshots were produced from the fixture XML with BBDown at commit 7c3e54d and Telegram-Video-Downloader's v3 release renderer at commit 9d0a5c3. The v3 snapshots use 3× font scale, 70% opacity, half screen and a 1,500 ms grouping window.

The fixtures are gzipped. Tests decompress them in memory and require no network connection.
