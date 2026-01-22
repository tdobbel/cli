#include <assert.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;
typedef float f32;
typedef double f64;

#define READ_U16(map, offset) (*(u16 *)((map) + (offset)))
#define READ_U32(map, offset) (*(u32 *)((map) + (offset)))
#define READ_F32(map, offset) (*(f32 *)((map) + (offset)))
#define READ_F64(map, offset) (*(f64 *)((map) + (offset)))

enum TiffType {
  TIFF_SHORT = 3,
  TIFF_LONG = 4,
  TIFF_FLOAT = 11,
  TIFF_DOUBLE = 12,
};

enum SampleType {
  SAMPLE_UNSIGNED_INT = 1,
  SAMPLE_SIGNED_INT = 2,
  SAMPLE_FLOAT = 3,
  SAMPLE_UNDEFINED = 4,
};

void parse_int(u32 *dst, u8 *map, u16 type, u32 *offset);
void parse_float(f64 *dst, u8 *map, u16 type, u32 *offset);

typedef struct {
  u32 length;
  u32 *data;
} ivector;

void ivector_init(ivector *vec, u8 *map, u16 type, u32 start, u32 count);
void ivector_free(ivector *vec);

typedef struct {
  u32 length;
  f64 *data;
} fvector;

void fvector_init(fvector *vec, u8 *map, u16 type, u32 start, u32 count);
void fvector_free(fvector *vec);

typedef struct {
  u32 image_width, image_length;
  u16 bits_per_sample;
  u16 compression;
  u16 photometric_interpretation;
  u16 samples_per_pixel;
  ivector *strip_offsets;
  u32 rows_per_strip;
  u16 planar_configuration;
  u16 sample_format;
  ivector *strip_byte_counts;
  u8 *projection;
  fvector *model_tie_points;
  fvector *model_pixel_scale_tag;
} tiff_ifd;

void parse_ifd_entry(tiff_ifd *ifd, u8 *map, u32 offset);
void free_ifd(tiff_ifd *ifd);

typedef struct {
  u8 *map;
  tiff_ifd *ifd;
  f64 *x, *y;
  f32 *data;
} tiff_dataset;

tiff_dataset *read_tiff(u8 *map);
void tiff_load_data(tiff_dataset *tif);
void free_tiff(tiff_dataset *tif);

int main(int argc, char *argv[]) {
  if (argc < 2) {
    fprintf(stderr, "Input file must be provided\n");
    return EXIT_FAILURE;
  }
  int fd = open(argv[1], O_RDONLY);
  if (fd == -1) {
    fprintf(stderr, "Could not read file\n");
    return EXIT_FAILURE;
  }
  struct stat stat_buf;
  if (fstat(fd, &stat_buf) == -1) {
    fprintf(stderr, "Could not get file stats\n");
    return EXIT_FAILURE;
  }
  u8 *map = (u8 *)mmap(NULL, stat_buf.st_size, PROT_READ, MAP_PRIVATE, fd, 0);

  tiff_dataset *tif = read_tiff(map);
  if (tif == NULL)
    return EXIT_FAILURE;
  // printf("%f\n", tif->data[0]);
  printf("%s\n", tif->ifd->projection);
  free_tiff(tif);

  munmap(map, stat_buf.st_size);

  return EXIT_SUCCESS;
}

void parse_int(u32 *dst, u8 *map, u16 type, u32 *offset) {
  switch (type) {
  case TIFF_SHORT:
    *dst = (u32)READ_U16(map, *offset);
    *offset += 2;
    break;
  case TIFF_LONG:
    *dst = READ_U32(map, *offset);
    *offset += 4;
    break;
  default:
    assert(0);
    break;
  }
}

void ivector_init(ivector *vec, u8 *map, u16 type, u32 start, u32 count) {
  vec->length = count;
  vec->data = (u32 *)malloc(sizeof(u32) * count);
  u32 iptr = start;
  for (u32 i = 0; i < count; ++i) {
    parse_int(vec->data + i, map, type, &start);
  }
}

void ivector_free(ivector *vec) {
  free(vec->data);
  free(vec);
}

void parse_float(f64 *dst, u8 *map, u16 type, u32 *offset) {
  switch (type) {
  case TIFF_FLOAT:
    *dst = (f64)READ_F32(map, *offset);
    *offset += 4;
    break;
  case TIFF_DOUBLE:
    *dst = READ_F64(map, *offset);
    *offset += 8;
    break;
  default:
    assert(0);
    break;
  }
}

void fvector_init(fvector *vec, u8 *map, u16 type, u32 start, u32 count) {
  vec->length = count;
  vec->data = (f64 *)malloc(sizeof(f64) * count);
  u32 iptr = start;
  for (u32 i = 0; i < count; ++i) {
    parse_float(vec->data + i, map, type, &start);
  }
}

void fvector_free(fvector *vec) {
  free(vec->data);
  free(vec);
}

void parse_ifd_entry(tiff_ifd *ifd, u8 *map, u32 offset) {
  u16 tag = READ_U16(map, offset);
  u16 type = READ_U16(map, offset + 2);
  u32 count = READ_U32(map, offset + 4);
  u32 value_or_offset = READ_U32(map, offset + 8);
  switch (tag) {
  case 256:
    ifd->image_width = value_or_offset;
    break;
  case 257:
    ifd->image_length = value_or_offset;
    break;
  case 258:
    ifd->bits_per_sample = (u16)value_or_offset;
    break;
  case 259:
    ifd->compression = (u16)value_or_offset;
    break;
  case 262:
    ifd->photometric_interpretation = (u16)value_or_offset;
    break;
  case 273:
    ifd->strip_offsets = (ivector *)malloc(sizeof(ivector));
    ivector_init(ifd->strip_offsets, map, type, value_or_offset, count);
    break;
  case 277:
    ifd->samples_per_pixel = (u16)value_or_offset;
    break;
  case 278:
    ifd->rows_per_strip = value_or_offset;
    break;
  case 279:
    ifd->strip_byte_counts = (ivector *)malloc(sizeof(ivector));
    ivector_init(ifd->strip_byte_counts, map, type, value_or_offset, count);
    break;
  case 284:
    ifd->planar_configuration = (u16)value_or_offset;
    break;
  case 339:
    ifd->sample_format = (u16)value_or_offset;
    break;
  case 33922:
    ifd->model_tie_points = (fvector *)malloc(sizeof(fvector));
    fvector_init(ifd->model_tie_points, map, type, value_or_offset, count);
    break;
  case 33550:
    ifd->model_pixel_scale_tag = (fvector *)malloc(sizeof(fvector));
    fvector_init(ifd->model_pixel_scale_tag, map, type, value_or_offset, count);
    break;
  case 34735:
    // GeoKeys -> do nothing with it so far...
    // u16 n_key = READ_U16(map, value_or_offset + 6);
    // u32 start = value_or_offset + 8;
    // for (u16 i = 0; i < n_key; ++i) {
    //   printf("id=%hu, tag=%hu, count=%hu, value/offset=%hu\n",
    //          READ_U16(map, start), READ_U16(map, start + 2),
    //          READ_U16(map, start + 4), READ_U16(map, start + 6));
    //   start += 8;
    // }
    break;
  case 34737:
    ifd->projection = (u8 *)malloc(count);
    memcpy(ifd->projection, map + value_or_offset, count);
    break;
  default:
    printf("Unknown IFD entry: Tag=%hu, Type=%hu, Count=%u, Offset/Value=%u\n",
           tag, type, count, value_or_offset);
    break;
  }
}

void free_ifd(tiff_ifd *ifd) {
  if (ifd->strip_byte_counts)
    ivector_free(ifd->strip_byte_counts);
  if (ifd->strip_offsets->data)
    ivector_free(ifd->strip_offsets);
  if (ifd->model_tie_points)
    fvector_free(ifd->model_tie_points);
  if (ifd->model_pixel_scale_tag)
    fvector_free(ifd->model_pixel_scale_tag);
  free(ifd->projection);
  free(ifd);
}

tiff_dataset *read_tiff(u8 *map) {
  u16 endianness = READ_U16(map, 0);
  if (endianness != *(u16 *)"II") {
    fprintf(stderr, "Current implementatio for little endian only\n");
    return NULL;
  }
  u16 magic_number = READ_U16(map, 2);
  if (magic_number == 43) {
    fprintf(stderr, "Current implementation does not support BigTIFF format\n");
    return NULL;
  }
  assert(READ_U16(map, 2) == 42);
  u32 offset = READ_U32(map, 4);
  u16 n_entry = READ_U16(map, offset);

  tiff_ifd *ifd = malloc(sizeof(tiff_ifd));
  offset += 2;
  for (u16 i = 0; i < n_entry; ++i) {
    parse_ifd_entry(ifd, map, offset);
    offset += 12;
  }
  offset = READ_U32(map, offset);
  assert(offset == 0);
  assert(ifd->sample_format == SAMPLE_FLOAT);

  tiff_dataset *tif = malloc(sizeof(tiff_dataset));
  tif->map = map;
  tif->ifd = ifd;
  tif->data = NULL;

  assert(ifd->model_pixel_scale_tag->length == 3);
  assert(ifd->model_tie_points->length == 6);

  // Assume tie point is the upper left corner
  f64 dx = ifd->model_pixel_scale_tag->data[0];
  tif->x = malloc(sizeof(f64) * ifd->image_width);
  tif->x[0] = ifd->model_tie_points->data[0];
  for (u32 ix = 1; ix < ifd->image_width; ++ix) {
    tif->x[ix] = tif->x[ix - 1] + dx;
  }
  f64 dy = ifd->model_pixel_scale_tag->data[1];
  tif->y = malloc(sizeof(f64) * ifd->image_length);
  tif->y[0] = ifd->model_tie_points->data[1];
  for (u32 iy = 1; iy < ifd->image_length; ++iy) {
    tif->y[iy] = tif->y[iy - 1] - dy;
  }

  return tif;
}

void tiff_load_data(tiff_dataset *tif) {
  u32 npix = tif->ifd->image_length * tif->ifd->image_width;
  tif->data = NULL;
  tif->data = (f32 *)malloc(sizeof(f32) * npix);
  u32 n_strip = tif->ifd->strip_offsets->length;
  u32 pixel = 0;
  for (u32 strip = 0; strip < n_strip && pixel < npix; ++strip) {
    u32 offset = tif->ifd->strip_offsets->data[strip];
    for (u32 i = 0; i < tif->ifd->strip_byte_counts->data[strip]; i += 4) {
      tif->data[pixel++] = READ_F32(tif->map, offset + i);
      if (pixel == npix)
        break;
    }
  }
}

void free_tiff(tiff_dataset *tif) {
  free_ifd(tif->ifd);
  if (tif->data)
    free(tif->data);
  free(tif->x);
  free(tif->y);
  free(tif);
}
