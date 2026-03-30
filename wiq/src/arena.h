#ifndef ARENA_H
#define ARENA_H

#include <assert.h>
#include <stdint.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

typedef uint64_t u64;
typedef uint32_t u32;
typedef int32_t i32;
typedef uint8_t u8;

typedef i32 b32;

#define KiB(n) ((u64)(n) << 10)
#define MiB(n) ((u64)(n) << 20)
#define GiB(n) ((u64)(n) << 30)

#define MIN(a, b) (((a) < (b)) ? (a) : (b))
#define ALIGN_UP_POW2(n, p) (((u64)(n) + ((u64)(p) - 1)) & (~((u64)(p) - 1)))
#define ARENA_BASE_POS (sizeof(mem_arena))
#define ARENA_ALIGN (sizeof(void *))

#define MAX_VECTOR_GROW 10

typedef struct {
  u64 size;
  u64 pos;
} mem_arena;

typedef struct {
  mem_arena *arena;
  u64 start_pos;
} mem_arena_tmp;

void *reserve_memory(u64 size);
b32 release_memory(void *ptr, u64 size);

mem_arena *arena_create(u64 size);
void *arena_push(mem_arena *arena, u64 size);
void arena_pop(mem_arena *arena, u64 size);
void arena_pop_to(mem_arena *arena, u64 pos);
void arena_clear(mem_arena *arena);
void arena_destroy(mem_arena *arena);

mem_arena_tmp arena_tmp_create(mem_arena *arena);
void arena_tmp_end(mem_arena_tmp arena_tmp);

#define PUSH_STRUCT(arena, T) (T *)arena_push((arena), sizeof(T))
#define PUSH_ARRAY(arena, T, n) (T *)arena_push((arena), (n) * sizeof(T))

typedef struct {
  mem_arena *arena;
  u8 n_blocks;
  u64 size, capacity, elem_size;
  u64 istart[MAX_VECTOR_GROW + 1];
  void *data[MAX_VECTOR_GROW];
} arena_vector;

arena_vector *arena_vector_create(mem_arena *arena, u64 initial_capacity,
                            u64 elem_size);
void *arena_vector_get(arena_vector *vec, u64 index);
void *arena_vector_append_get(arena_vector *vec);
void *arena_vector_flatten(arena_vector *vec);
void arena_vector_grow(arena_vector *vec);

#define AVEC_CREATE(arena, T, cap) arena_vector_create((arena), (cap), sizeof(T))
#define AVEC_GET_REF(vec, T, i) (T *)arena_vector_get((arena), (i))
#define AVEC_GET(vec, T, i) *(T *)arena_vector_get((vec), (i))
#define AVEC_PUSH(vec, T, x) (*(T *)arena_vector_append_get((vec)) = (x))
#define AVEC_FLATTEN(vec, T) (T *)arena_vector_flatten((vec))

#endif

#ifdef ARENA_IMPLEMENTATION

void *reserve_memory(u64 size) {
  void *out = mmap(NULL, size, PROT_READ | PROT_WRITE,
                   MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
  if (out == MAP_FAILED) {
    return NULL;
  }
  return out;
}

b32 release_memory(void *ptr, u64 size) {
  i32 ret = munmap(ptr, size);
  return ret == 0;
}

mem_arena *arena_create(u64 size) {
  u32 pagesize = (u32)sysconf(_SC_PAGESIZE);
  size = ALIGN_UP_POW2(size, pagesize);
  mem_arena *arena = (mem_arena *)reserve_memory(size);
  arena->pos = ARENA_BASE_POS;
  arena->size = size;
  return arena;
}

void *arena_push(mem_arena *arena, u64 size) {
  u64 pos_aligned = ALIGN_UP_POW2(arena->pos, ARENA_ALIGN);
  u64 new_pos = pos_aligned + size;
  assert(new_pos < arena->size);
  arena->pos = new_pos;
  u8 *out = (u8 *)arena + pos_aligned;
  return out;
}

void arena_pop(mem_arena *arena, u64 size) {
  size = MIN(size, arena->pos - ARENA_ALIGN);
  arena->pos -= size;
}

void arena_pop_to(mem_arena *arena, u64 pos) {
  u64 size = pos < arena->pos ? arena->pos - pos : 0;
  arena_pop(arena, size);
}

void arena_clear(mem_arena *arena) { arena_pop_to(arena, ARENA_BASE_POS); }

void arena_destroy(mem_arena *arena) { release_memory(arena, arena->size); }

mem_arena_tmp arena_tmp_create(mem_arena *arena) {
  return (mem_arena_tmp){.arena = arena, .start_pos = arena->pos};
}

void arena_tmp_end(mem_arena_tmp arena_tmp) {
  arena_pop(arena_tmp.arena, arena_tmp.start_pos);
}

arena_vector *arena_vector_create(mem_arena *arena, u64 initial_capacity,
                            u64 elem_size) {
  arena_vector *vec = PUSH_STRUCT(arena, arena_vector);
  vec->size = 0;
  vec->n_blocks = 1;
  vec->arena = arena;
  vec->capacity = initial_capacity;
  vec->elem_size = elem_size;
  vec->istart[0] = 0;
  vec->istart[1] = initial_capacity;
  vec->data[0] = arena_push(arena, vec->capacity * elem_size);
  return vec;
}

void *arena_vector_get(arena_vector *vec, u64 index) {
  assert(index < vec->size);
  u32 iblk = 0;
  while (iblk < vec->n_blocks) {
    if (index < vec->istart[iblk + 1])
      break;
    iblk++;
  }
  u64 index2 = index - vec->istart[iblk];
  return (u8 *)vec->data[iblk] + index2 * vec->elem_size;
}

void arena_vector_grow(arena_vector *vec) {
  if (vec->size < vec->capacity)
    return;
  assert(vec->n_blocks < MAX_VECTOR_GROW);
  u64 extra_capacity = vec->capacity / 2;
  vec->data[vec->n_blocks++] =
      arena_push(vec->arena, extra_capacity * vec->elem_size);
  vec->capacity += extra_capacity;
  vec->istart[vec->n_blocks] = vec->capacity;
}

void *arena_vector_append_get(arena_vector *vec) {
  arena_vector_grow(vec);
  return arena_vector_get(vec, vec->size++);
}

void *arena_vector_flatten(arena_vector *vec) {
  u8 *flat = (u8 *)arena_push(vec->arena, vec->size * vec->elem_size);
  for (u32 iblk = 0; iblk < vec->n_blocks; ++iblk) {
    u64 block_size = MIN(vec->istart[iblk + 1] - vec->istart[iblk],
                         vec->size - vec->istart[iblk]);
    memcpy(flat + vec->istart[iblk] * vec->elem_size, vec->data[iblk],
           block_size * vec->elem_size);
  }
  return flat;
}

#endif
