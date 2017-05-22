#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>

extern void slp_free(char *image_data_buff
                    ,size_t len);

extern ssize_t slp_new_from_file(const char *file_path
                                         ,char **image_data_buff
                                         ,size_t *width
                                         ,size_t *height);

const int ERR_NO_ARG = 1;

int main(int argc, char **argv) {
  if (argc != 2) {
    printf("usage: cslp <path/to/your.slp>\n");
    return ERR_NO_ARG;
  }

  char *file_path = argv[1];
  char *ptr_image_data = NULL;
  size_t frame_width = 0;
  size_t frame_height = 0;

  ssize_t code = slp_new_from_file(file_path, &ptr_image_data, &frame_width, &frame_height);
  if (code != 0) {
    switch (code) {
      case 1: printf("'file_path' was null!"); break;
      case 2: printf("'file_path' contained non-utf8 characters!"); break;
      case -1: printf("Invalid SLP!"); break;
      case -2: printf("SLP had a bad length"); break;
      case -32767: printf("An unknown error occurred while decoding the SLP"); break;
    }

    return code;
  }

  printf("image_data_len: %zu\n", frame_width * frame_height);
  slp_free(ptr_image_data, frame_width * frame_height);
}