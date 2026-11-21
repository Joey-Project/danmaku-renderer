#include <ass/ass.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    if (argc != 2) {
        return 2;
    }
    int region_height = atoi(argv[1]);
    char input[16384];
    size_t size = fread(input, 1, sizeof(input), stdin);
    if (ferror(stdin) || size == sizeof(input)) {
        return 2;
    }

    ASS_Library *library = ass_library_init();
    if (!library) {
        return 2;
    }
    ASS_Renderer *renderer = ass_renderer_init(library);
    ASS_Track *track = ass_read_memory(library, input, size, NULL);
    if (!renderer || !track) {
        return 2;
    }
    ass_set_frame_size(renderer, 1920, 1080);
    ass_set_fonts(renderer, NULL, "sans-serif", ASS_FONTPROVIDER_AUTODETECT, NULL, 1);
    bool ok = true;

    for (int time = 100; time < 4000 && ok; time += 100) {
        int last_red = -1;
        int first_green = 1080;
        for (ASS_Image *image = ass_render_frame(renderer, track, time, NULL);
             image; image = image->next) {
            for (int y = 0; y < image->h; ++y) {
                for (int x = 0; x < image->w; ++x) {
                    if (!image->bitmap[y * image->stride + x]) {
                        continue;
                    }
                    int row = image->dst_y + y;
                    if (row < 0 || row >= region_height || (image->color & 0xff) != 0x4d) {
                        fprintf(stderr, "Unexpected paint at %d ms in row %d (color %08x)\n",
                                time, row, (unsigned)image->color);
                        ok = false;
                        break;
                    }
                    uint32_t color = image->color >> 8;
                    if (color == 0xff0000 && row > last_red) {
                        last_red = row;
                    }
                    if (color == 0x00ff00 && row < first_green) {
                        first_green = row;
                    }
                }
            }
        }
        if (last_red < 0 || first_green == 1080 || last_red >= first_green) {
            fprintf(stderr, "Missing or overlapping fixed text at %d ms (%d, %d)\n",
                    time, last_red, first_green);
            ok = false;
        }
    }

    ass_free_track(track);
    ass_renderer_done(renderer);
    ass_library_done(library);
    return ok ? 0 : 1;
}
