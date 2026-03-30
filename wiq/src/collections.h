#ifndef _COLLECTIONS_H_
#define _COLLECTIONS_H_

#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#ifdef STRING_IMPLEMENTATION
#include "string8.h"
#endif

#define uint128_t __uint128_t

typedef uint128_t u128;
typedef uint64_t u64;
typedef uint32_t u32;
typedef int32_t i32;
typedef uint8_t u8;
typedef u8 b8;

#define SEED 0x00000000
#define BASE_CAPACITY 16
#define DEFAULT_MAX_LOAD_FACTOR 80

typedef struct {
  u64 elem_size, capacity, size;
  void *data;
} vector;

vector *vector_create(u64 elem_size);
void vector_free(vector *vec);
void *vector_get(vector *vec, u64 index);
void *vector_append_get(vector *vec);

#ifdef STRING_IMPLEMENTATION
vector *split_whitespace(string8 s);
#endif

#define VEC_CREATE(T) vector_create(sizeof(T))
#define VEC_PUSH(vec, T, x) (*(T *)vector_append_get((vec)) = (x))

#define READ4(a) (u64)(*(u32 *)(a))

const u64 secret[4] = {
    0xa0761d6478bd642f,
    0xe7037ed1a0b428db,
    0x8ebc6af09c88c6e3,
    0x589965cc75374cc3,
};

typedef struct {
  u64 a, b;
  u64 state[3];
  u64 total_len;
} Wyhash;

static inline void mum(u64 *a, u64 *b) {
  u128 r = *a;
  r *= *b;
  *a = (u64)r;
  *b = (u64)(r >> 64);
}

static inline u64 mix(u64 a_, u64 b_) {
  u64 a = a_;
  u64 b = b_;
  mum(&a, &b);
  return a ^ b;
}

static inline void wy_round(Wyhash *self, const u8 *input);
static inline void wyhash_small_key(Wyhash *self, const u8 *input,
                                    u64 input_len);
static inline void final0(Wyhash *self) {
  self->state[0] ^= self->state[1] ^ self->state[2];
}
static inline void final1(Wyhash *self, const u8 *input_lb, u64 input_len,
                          u64 start_pos);
static inline u64 final2(Wyhash *self);

Wyhash wyhash_init(u64 seed);
u64 wyhash(const u8 *input, u64 input_len, u64 seed);

typedef struct {
  u64 key_size, value_size;
} hash_map_context;

typedef b8 (*eq_fn)(const hash_map_context ctx, const void *, const void *);
typedef u64 (*hash_fn)(const hash_map_context, const void *);

u64 wyhash_auto(const hash_map_context ctx, const void *input);

typedef struct {
  b8 found_existing;
  void *key_ptr;
  void *value_ptr;
} kv_entry;

typedef struct {
  u64 capacity, size;
  hash_map_context ctx;
  u8 *keys;
  u8 *values;
  u8 *fingerprint; // bit 1: 0 if free, 1 if used, 7 other bits -> hash
  hash_fn hash;
  eq_fn eq;
} hash_map;

b8 bytes_eql(const hash_map_context ctx, const void *a, const void *b);

static u64 ensure_pow2(u64 cap);

hash_map *hm_init(u64 capacity, hash_map_context ctx, hash_fn hash, eq_fn eq);
u64 hm_get_index(hash_map *hm, const void *key);
kv_entry hm_get_entry(hash_map *hm, const void *key);
kv_entry hm_get_or_put(hash_map *hm, const void *key);
void grow_if_needed(hash_map *hm);
b8 hm_get(hash_map *hm, const void *key, void *value_ptr);
void hm_put(hash_map *hm, const void *key, const void *value);
void hm_put_assume_capacity(hash_map *hm, const void *key, const void *value);
void hm_deinit(hash_map *hm);

#define AUTO_HASHMAP(K, V)                                                     \
  hm_init(BASE_CAPACITY,                                                       \
          (hash_map_context){.key_size = sizeof(K), .value_size = sizeof(V)},  \
          wyhash_auto, bytes_eql);

#ifdef STRING_IMPLEMENTATION

b8 string8_eql(const hash_map_context ctx, const void *a, const void *b);
u64 wyhash_string8(const hash_map_context ctx, const void *strkey);

#define STRING_HASHMAP(V)                                                      \
  hm_init(BASE_CAPACITY,                                                       \
          (hash_map_context){.key_size = sizeof(string8),                      \
                             .value_size = sizeof(V)},                         \
          wyhash_string8, string8_eql);

#endif

typedef struct {
  hash_map *hm;
  u64 cntr, indx;
  void *key_ptr;
  void *value_ptr;
} kv_iterator;

kv_iterator hm_iterator(hash_map *hm);
b8 get_next(kv_iterator *kv_iter);

#ifdef VECTOR_IMPLEMENTATION

vector *vector_create(u64 elem_size) {
  vector *vec = (vector *)malloc(sizeof(vector));
  vec->size = 0;
  vec->elem_size = elem_size;
  vec->capacity = BASE_CAPACITY;
  vec->data = malloc(BASE_CAPACITY * elem_size);
  return vec;
}

void *vector_get(vector *vec, u64 index) {
  assert(vec && index < vec->size);
  return (void *)((char *)vec->data + index * vec->elem_size);
}

void *vector_append_get(vector *vec) {
  if (vec->size == vec->capacity) {
    vec->capacity *= 2;
    vec->data = realloc(vec->data, vec->capacity * vec->elem_size);
  }
  return vector_get(vec, vec->size++);
}

void vector_free(vector *vec) {
  if (!vec)
    return;
  free(vec->data);
  free(vec);
}

#ifdef STRING_IMPLEMENTATION

vector *split_whitespace(string8 s) {
  vector *vec = VEC_CREATE(string8);
  string8 s2 = str_trim(s);
  while (s2.size > 0) {
    u64 start = 0;
    u64 end = start;
    while (end < s2.size && !isspace(s2.str[end])) {
      end++;
    }
    string8 word = (string8){.str = s2.str, .size = end - start};
    VEC_PUSH(vec, string8, word);
    string8 r = (string8){.str = s2.str + end, .size = s2.size - end};
    s2 = str_trim_left(r);
  }
  return vec;
}

#endif

#endif

#ifdef HASHMAP_IMPLEMENTATION

static inline void wy_round(Wyhash *self, const u8 *input) {
  for (u64 i = 0; i < 3; ++i) {
    u64 a = *(u64 *)(input + 8 * (2 * i));
    u64 b = *(u64 *)(input + 8 * (2 * i + 1));
    self->state[i] = mix(a ^ secret[i + 1], b ^ self->state[i]);
  }
}

Wyhash wyhash_init(u64 seed) {
  Wyhash self = (Wyhash){.total_len = 0};
  self.state[0] = seed ^ mix(seed ^ secret[0], secret[1]);
  self.state[1] = self.state[0];
  self.state[2] = self.state[0];
  return self;
}

static inline void wyhash_small_key(Wyhash *self, const u8 *input,
                                    u64 input_len) {
  assert(input_len <= 16);
  if (input_len >= 4) {
    u64 end = input_len - 4;
    u64 quarter = (input_len >> 3) << 2;
    self->a = (READ4(input) << 32) | READ4(input + quarter);
    self->b = (*(u64 *)(input + end) << 32) | *(u64 *)(input + end - quarter);
  } else if (input_len > 0) {
    self->a = ((u64)input[0] << 16) | ((u64)input[input_len >> 1] << 8) |
              (u64)input[input_len - 1];
    self->b = 0;
  } else {
    self->a = 0;
    self->b = 0;
  }
}

static inline void final1(Wyhash *self, const u8 *input_lb, u64 input_len,
                          u64 start_pos) {
  assert(input_len >= 16);
  assert(input_len - start_pos <= 48);
  const u8 *input = input_lb + start_pos;
  u64 len = input_len - start_pos;

  u64 i = 0;
  while (i + 16 < len) {
    self->state[0] = mix((*(u64 *)(input + i)) ^ secret[1],
                         (*(u64 *)(input + i + 8)) ^ self->state[0]);
    i += 16;
  }
  self->a = *(u64 *)(input_lb + input_len - 16);
  self->b = *(u64 *)(input_lb + input_len - 8);
}

static inline u64 final2(Wyhash *self) {
  self->a ^= secret[1];
  self->b ^= self->state[0];
  mum(&self->a, &self->b);
  return mix(self->a ^ secret[0] ^ self->total_len, self->b ^ secret[1]);
}

u64 wyhash(const u8 *input, u64 input_len, u64 seed) {
  Wyhash self = wyhash_init(seed);
  if (input_len <= 16) {
    wyhash_small_key(&self, input, input_len);
  } else {
    u64 i = 0;
    if (input_len >= 48) {
      while (i + 48 < input_len) {
        wy_round(&self, input);
        i += 48;
      }
      final0(&self);
    }
    final1(&self, input, input_len, i);
  }

  self.total_len = input_len;
  return final2(&self);
}

u64 wyhash_auto(const hash_map_context ctx, const void *key) {
  return wyhash((u8 *)key, ctx.key_size, SEED);
}

#define ISUSED(hm, indx) ((hm)->fingerprint[indx] & 0x01)

static u64 ensure_pow2(u64 cap) {
  u64 k = 1;
  while (k < cap) {
    k <<= 1;
  }
  return k;
}

hash_map *hm_init(u64 capacity, hash_map_context ctx, hash_fn hash, eq_fn eq) {
  hash_map *hm = (hash_map *)malloc(sizeof(hash_map));
  u64 cap = ensure_pow2(capacity);
  hm->capacity = cap;
  hm->size = 0;
  hm->ctx = ctx;
  hm->keys = (u8 *)malloc(cap * ctx.key_size);
  hm->values = (u8 *)malloc(cap * ctx.value_size);
  hm->hash = hash;
  hm->eq = eq;
  hm->fingerprint = (u8 *)malloc(
      cap); // first bit 7 bits for the hash and 1 bit for used or not
  memset(hm->fingerprint, 0x00, cap);
  return hm;
}

b8 bytes_eql(const hash_map_context ctx, const void *a, const void *b) {
  u8 *abytes = (u8 *)a;
  u8 *bbytes = (u8 *)b;
  for (u64 i = 0; i < ctx.key_size; ++i) {
    if (abytes[i] != bbytes[i])
      return 0;
  }
  return 1;
}

u64 hm_get_index(hash_map *hm, const void *key) {
  u64 hash = hm->hash(hm->ctx, key);
  u64 indx = hash & (hm->capacity - 1);
  u64 limit = hm->capacity;
  u8 fingerprint = hash >> 57;
  u64 ksz = hm->ctx.key_size;
  while (ISUSED(hm, indx) && limit > 0) {
    u8 fp = hm->fingerprint[indx] >> 1;
    if (fp == fingerprint) {
      const u8 *test_key = hm->keys + indx * ksz;
      if (hm->eq(hm->ctx, test_key, key))
        return indx;
    }
    indx = (indx + 1) & (hm->capacity - 1);
    limit--;
  }
  assert(limit > 0);
  hm->fingerprint[indx] = fingerprint << 1;
  return indx;
}

kv_entry hm_get_entry(hash_map *hm, const void *key) {
  u64 indx = hm_get_index(hm, key);
  if (!ISUSED(hm, indx))
    return (kv_entry){.found_existing = 0, .key_ptr = NULL, .value_ptr = NULL};
  u8 *key_ptr = hm->keys + indx * hm->ctx.key_size;
  u8 *value_ptr = hm->values + indx * hm->ctx.value_size;
  return (kv_entry){
      .found_existing = 1, .key_ptr = key_ptr, .value_ptr = value_ptr};
}

void hm_put_assume_capacity(hash_map *hm, const void *key, const void *value) {
  u64 indx = hm_get_index(hm, key);
  u8 *value_ptr = hm->values + indx * hm->ctx.value_size;
  memcpy(value_ptr, value, hm->ctx.value_size);
  if (!ISUSED(hm, indx)) {
    u8 *key_ptr = hm->keys + indx * hm->ctx.key_size;
    memcpy(key_ptr, key, hm->ctx.key_size);
    hm->fingerprint[indx] |= 0x01;
    hm->size++;
  }
}

void grow_if_needed(hash_map *hm) {
  u64 load = (100 * hm->size) / hm->capacity;
  if (load < DEFAULT_MAX_LOAD_FACTOR)
    return;
  u64 new_cap = hm->capacity * 2;
  hash_map *map = hm_init(new_cap, hm->ctx, hm->hash, hm->eq);
  for (u64 i = 0; i < hm->capacity; ++i) {
    if (!ISUSED(hm, i))
      continue;
    u8 *key_ptr = hm->keys + i * hm->ctx.key_size;
    u8 *value_ptr = hm->values + i * hm->ctx.value_size;
    hm_put_assume_capacity(map, key_ptr, value_ptr);
  }
  hm->capacity = new_cap;
  free(hm->values);
  free(hm->keys);
  free(hm->fingerprint);
  hm->values = map->values;
  hm->keys = map->keys;
  hm->fingerprint = map->fingerprint;
  free(map);
}

kv_entry hm_get_or_put(hash_map *hm, const void *key) {
  grow_if_needed(hm);
  u64 indx = hm_get_index(hm, key);
  u8 *value_ptr = hm->values + hm->ctx.value_size * indx;
  u8 *key_ptr = hm->keys + hm->ctx.key_size * indx;
  if (ISUSED(hm, indx)) {
    return (kv_entry){
        .found_existing = 1, .value_ptr = value_ptr, .key_ptr = key_ptr};
  }
  memcpy(key_ptr, key, hm->ctx.key_size);
  hm->fingerprint[indx] |= 0x01;
  hm->size++;
  return (kv_entry){
      .found_existing = 0, .key_ptr = key_ptr, .value_ptr = value_ptr};
}

void hm_put(hash_map *hm, const void *key, const void *value) {
  kv_entry entry = hm_get_or_put(hm, key);
  memcpy(entry.value_ptr, value, hm->ctx.value_size);
}

void hm_deinit(hash_map *hm) {
  free(hm->keys);
  free(hm->values);
  free(hm->fingerprint);
  free(hm);
}

kv_iterator hm_iterator(hash_map *hm) {
  return (kv_iterator){.hm = hm,
                       .cntr = hm->size,
                       .indx = 0,
                       .key_ptr = NULL,
                       .value_ptr = NULL};
}

b8 get_next(kv_iterator *kvi) {
  if (kvi->cntr == 0)
    return 0;
  if (kvi->cntr < kvi->hm->size)
    kvi->indx++;
  while (!ISUSED(kvi->hm, kvi->indx)) {
    kvi->indx++;
  }
  kvi->cntr--;
  kvi->value_ptr = kvi->hm->values + kvi->indx * kvi->hm->ctx.value_size;
  kvi->key_ptr = kvi->hm->keys + kvi->indx * kvi->hm->ctx.key_size;
  return 1;
}

#ifdef STRING_IMPLEMENTATION

b8 string8_eql(const hash_map_context ctx, const void *a, const void *b) {
  assert(ctx.key_size == sizeof(string8));
  string8 *sa = (string8 *)a;
  string8 *sb = (string8 *)b;
  if (sa->size != sb->size)
    return 0;
  for (u64 i = 0; i < sa->size; ++i) {
    if (sa->str[i] != sb->str[i])
      return 0;
  }
  return 1;
}

u64 wyhash_string8(const hash_map_context ctx, const void *strkey) {
  assert(ctx.key_size == sizeof(string8));
  string8 *key = (string8 *)strkey;
  return wyhash(key->str, key->size, SEED);
}

#endif

#endif
#endif
