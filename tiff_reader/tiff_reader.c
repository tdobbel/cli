#include <assert.h>
#include <fcntl.h>
#include <float.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef int16_t i16;
typedef uint32_t u32;
typedef uint64_t u64;
typedef float f32;
typedef double f64;
typedef u32 b32;

#define READ_U16(map, offset) (*(u16 *)((map) + (offset)))
#define READ_I16(map, offset) (*(i16 *)((map) + (offset)))
#define READ_U32(map, offset) (*(u32 *)((map) + (offset)))
#define READ_F32(map, offset) (*(f32 *)((map) + (offset)))
#define READ_F64(map, offset) (*(f64 *)((map) + (offset)))

#define MIN(a, b) (((a) < (b)) ? (a) : (b))
#define MAX(a, b) (((a) > (b)) ? (a) : (b))
#define ABS(a) (((a) < 0) ? -(a) : (a))

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

typedef struct {
  u16 tag, type;
  u32 count, value_offset;
} ifd_entry;

ifd_entry read_entry(u8 *map, u32 *offset);

typedef struct {
  u32 capacity, length;
  enum TiffType dtype;
  u16 bytesize;
  u8 *data;
} vector;

u16 get_byte_size(enum TiffType dtype);
void vector_free(vector *vec);
vector *vector_from_slice(u8 *map, ifd_entry entry);
u32 vector_get_u32(vector *vec, u32 i);
f64 vector_get_f64(vector *vec, u32 i);

typedef struct {
  u32 image_width, image_length;
  u16 bits_per_sample;
  u16 compression;
  u16 photometric_interpretation;
  u16 samples_per_pixel;
  vector *strip_offsets;
  u32 rows_per_strip;
  u16 planar_configuration;
  u16 sample_format;
  vector *strip_byte_counts;
  u8 *projection;
  vector *model_tie_points;
  vector *model_pixel_scale_tag;
  vector *model_transformation_tag;
  vector *geo_double_params_tag;
} tiff_ifd;

tiff_ifd *ifd_init();
void parse_ifd_entry(tiff_ifd *ifd, u8 *map, u32 *offset);
void free_ifd(tiff_ifd *ifd);

typedef struct {
  u8 *map;
  tiff_ifd *ifd;
  f64 *x, *y;
  void *data;
} tiff_dataset;

tiff_dataset *read_tiff(u8 *map);
void tiff_load_data(tiff_dataset *tif);
void free_tiff(tiff_dataset *tif);
u32 searchsorted(f64 *values, f64 key, u32 n);

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
  printf("%s\n", tif->ifd->projection);
  tiff_load_data(tif);
  u32 size = tif->ifd->image_length * tif->ifd->image_width;
  switch (tif->ifd->sample_format) {
  case SAMPLE_UNSIGNED_INT:
    printf("data[%u]=%hu\n", size - 1, *((u16 *)tif->data + size - 1));
    break;
  case SAMPLE_SIGNED_INT:
    printf("data[%u]=%d\n", size - 1, *((i16 *)tif->data + size - 1));
    break;
  case SAMPLE_FLOAT:
    printf("data[%u]=%f\n", size - 1, *((f32 *)tif->data + size - 1));
    break;
  }
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

u16 get_byte_size(enum TiffType dtype) {
  switch (dtype) {
  case TIFF_SHORT:
    return sizeof(u16);
  case TIFF_LONG:
    return sizeof(u32);
  case TIFF_FLOAT:
    return sizeof(f32);
  case TIFF_DOUBLE:
    return sizeof(f64);
  }
}

vector *vector_from_slice(u8 *map, ifd_entry entry) {
  vector *vec = malloc(sizeof(vector));
  u16 bytesize = get_byte_size(entry.type);
  vec->capacity = entry.count;
  vec->length = entry.count;
  vec->bytesize = bytesize, vec->dtype = entry.type,
  vec->data = (u8 *)malloc((u32)bytesize * entry.count);
  memcpy(vec->data, map + entry.value_offset, bytesize * entry.count);
  return vec;
}

void vector_free(vector *vec) {
  if (vec == NULL)
    return;
  free(vec->data);
  free(vec);
}

u32 vector_get_u32(vector *vec, u32 i) {
  return READ_U32(vec->data, i * vec->bytesize);
}

f64 vector_get_f64(vector *vec, u32 i) {
  return READ_F64(vec->data, i * vec->bytesize);
}

ifd_entry read_entry(u8 *map, u32 *offset) {
  ifd_entry entry = {.tag = READ_U16(map, *offset),
                     .type = READ_U16(map, *offset + 2),
                     .count = READ_U32(map, *offset + 4),
                     .value_offset = READ_U32(map, *offset + 8)};
  *offset += 12;
  return entry;
}

void parse_ifd_entry(tiff_ifd *ifd, u8 *map, u32 *offset) {
  ifd_entry entry = read_entry(map, offset);
  switch (entry.tag) {
  case 256:
    ifd->image_width = entry.value_offset;
    break;
  case 257:
    ifd->image_length = entry.value_offset;
    break;
  case 258:
    ifd->bits_per_sample = (u16)entry.value_offset;
    break;
  case 259:
    ifd->compression = (u16)entry.value_offset;
    break;
  case 262:
    ifd->photometric_interpretation = (u16)entry.value_offset;
    break;
  case 273:
    ifd->strip_offsets = vector_from_slice(map, entry);
    break;
  case 277:
    ifd->samples_per_pixel = (u16)entry.value_offset;
    break;
  case 278:
    ifd->rows_per_strip = entry.value_offset;
    break;
  case 279:
    ifd->strip_byte_counts = vector_from_slice(map, entry);
    break;
  case 284:
    ifd->planar_configuration = (u16)entry.value_offset;
    break;
  case 339:
    ifd->sample_format = (u16)entry.value_offset;
    break;
  case 33922:
    ifd->model_tie_points = vector_from_slice(map, entry);
    break;
  case 33550:
    ifd->model_pixel_scale_tag = vector_from_slice(map, entry);
    break;
  case 34264:
    assert(entry.count == 16);
    ifd->model_transformation_tag = vector_from_slice(map, entry);
    break;
  case 34735:
    // GeoKeys -> do nothing with it so far...
    // u16 n_key = READ_U16(map, entry.value_offset + 6);
    // u32 start = entry.value_offset + 8;
    // for (u16 i = 0; i < n_key; ++i) {
    //   printf("id=%hu, tag=%hu, count=%hu, value/offset=%hu\n",
    //          READ_U16(map, start), READ_U16(map, start + 2),
    //          READ_U16(map, start + 4), READ_U16(map, start + 6));
    //   start += 8;
    // }
    break;
  case 34736:
    ifd->geo_double_params_tag = vector_from_slice(map, entry);
    break;
  case 34737:
    ifd->projection = (u8 *)malloc(entry.count);
    memcpy(ifd->projection, map + entry.value_offset, entry.count);
    break;
  default:
    printf("Unknown IFD entry: Tag=%hu, Type=%hu, Count=%u, Offset/Value=%u\n",
           entry.tag, entry.type, entry.count, entry.value_offset);
    break;
  }
}

void free_ifd(tiff_ifd *ifd) {
  vector_free(ifd->strip_byte_counts);
  vector_free(ifd->strip_offsets);
  vector_free(ifd->model_tie_points);
  vector_free(ifd->model_pixel_scale_tag);
  vector_free(ifd->model_transformation_tag);
  vector_free(ifd->geo_double_params_tag);
  free(ifd->projection);
  free(ifd);
}

tiff_ifd *ifd_init() {
  tiff_ifd *ifd = (tiff_ifd *)malloc(sizeof(tiff_ifd));
  ifd->model_tie_points = NULL;
  ifd->model_pixel_scale_tag = NULL;
  ifd->model_transformation_tag = NULL;
  ifd->geo_double_params_tag = NULL;
  return ifd;
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

  tiff_ifd *ifd = ifd_init();
  offset += 2;
  for (u16 i = 0; i < n_entry; ++i) {
    parse_ifd_entry(ifd, map, &offset);
  }
  offset = READ_U32(map, offset);
  assert(offset == 0);
  assert(ifd->sample_format != SAMPLE_UNDEFINED);

  tiff_dataset *tif = malloc(sizeof(tiff_dataset));
  tif->map = map;
  tif->ifd = ifd;
  tif->data = NULL;
  tif->x = malloc(sizeof(f64) * ifd->image_width);
  tif->y = malloc(sizeof(f64) * ifd->image_length);

  if (ifd->model_pixel_scale_tag && ifd->model_tie_points) {
    printf("%d\n", ifd->model_pixel_scale_tag->length);
    assert(ifd->model_pixel_scale_tag->length == 3);
    assert(ifd->model_tie_points->length == 6);
    // Assume tie point is the upper left corner
    f64 *tie_points = (f64 *)ifd->model_tie_points->data;
    f64 *scale_tag = (f64 *)ifd->model_pixel_scale_tag->data;
    f64 dx = scale_tag[0];
    tif->x[0] = tie_points[3];
    for (u32 ix = 1; ix < ifd->image_width; ++ix) {
      tif->x[ix] = tif->x[ix - 1] + dx;
    }
    f64 dy = scale_tag[1];
    tif->y[0] = tie_points[4];
    for (u32 iy = 1; iy < ifd->image_length; ++iy) {
      tif->y[iy] = tif->y[iy - 1] - dy;
    }
    return tif;
  }
  if (ifd->model_transformation_tag) {
    f64 *trans = (f64 *)ifd->model_transformation_tag->data;
    assert(ABS(trans[1]) < DBL_EPSILON && ABS(trans[4]) < DBL_EPSILON);
    for (u64 i = 0; i < (u64)ifd->image_width; ++i) {
      tif->x[i] = trans[3] + trans[0] * (f64)i;
    }
    for (u64 i = 0; i < (u64)ifd->image_length; ++i) {
      tif->y[i] = trans[7] + trans[5] * (f64)i;
    }
    return tif;
  }

  fprintf(stderr, "Invalid transformation\n");
  free_tiff(tif);
  return NULL;
}

u32 searchsorted(f64 *values, f64 key, u32 n) {
  f64 step = values[1] - values[0];
  u32 index = (u32)((key - values[0]) / step);
  return MAX(0, MIN(index, n - 2));
}

void tiff_load_data(tiff_dataset *tif) {
  u32 nx = tif->ifd->image_width;
  u32 ny = tif->ifd->image_length;
  u32 size = (u32)tif->ifd->bits_per_sample / 8;
  tif->data = malloc(size * nx * ny);
  u32 n_stripe = tif->ifd->strip_offsets->length;
  u32 *strip_offsets = (u32 *)tif->ifd->strip_offsets->data;
  u16 *strip_byte_counts = (u16 *)tif->ifd->strip_byte_counts->data;
  u32 pixel = 0;
  for (u32 stripe = 0; stripe < n_stripe; ++stripe) {
    u32 offset = strip_offsets[stripe];
    u32 byte_count = (u32)strip_byte_counts[stripe];
    for (u32 i = 0; i < byte_count; i += size) {
      switch (tif->ifd->sample_format) {
      case SAMPLE_UNSIGNED_INT:
        *((u16 *)tif->data + pixel++) = READ_U16(tif->map, offset + i);
        break;
      case SAMPLE_SIGNED_INT:
        *((i16 *)tif->data + pixel++) = READ_I16(tif->map, offset + i);
        break;
      case SAMPLE_FLOAT:
        *((f32 *)tif->data + pixel++) = READ_F32(tif->map, offset + i);
        break;
      default:
        fprintf(stderr, "Unexpected sample format");
        free(tif->data);
        return;
      }
      // printf("pixel=%u/%u\n", pixel, nx * ny);
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
