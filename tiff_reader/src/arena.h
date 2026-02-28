#ifndef ARENA_H
#define ARENA_H

#include <assert.h>
#include <stdint.h>
#include <sys/mman.h>
#include <unistd.h>

typedef uint64_t u64;
typedef uint32_t u32;
typedef int32_t i32;
typedef uint8_t u8;

typedef i32 b32;

#define MIN(a, b) (((a) < (b)) ? (a) : (b))
#define ALIGN_UP_POW2(n, p) (((u64)(n) + ((u64)(p) - 1)) & (~((u64)(p) - 1)))
#define ARENA_BASE_POS (sizeof(mem_arena))
#define ARENA_ALIGN (sizeof(void *))

typedef struct {
  u64 size;
  u64 pos;
} mem_arena;

void *reserve_memory(u64 size);
b32 release_memory(void *ptr, u64 size);

mem_arena *arena_create(u64 size);
void *arena_push(mem_arena *arena, u64 size);
void arena_pop(mem_arena *arena, u64 size);
void arena_pop_to(mem_arena *arena, u64 pos);
void arena_clear(mem_arena *arena);
void arena_destroy(mem_arena *arena);

#define PUSH_STRUCT(arena, T) (T *)arena_push((arena), sizeof(T))
#define PUSH_ARRAY(arena, T, n) (T *)arena_push((arena), (n) * sizeof(T))

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

#endif
