#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define STRING_IMPLEMENTATION
#include "string8.h"

#define VECTOR_IMPLEMENTATION
#include "vector.h"

#define HASHMAP_IMPLEMENTATION
#include "hash_map.h"

#include "arena.h"

int cmp_fn(const void *a, const void *b) {
  string8 sa = *(string8 *)a;
  string8 sb = *(string8 *)b;
  return strncmp((char *)sa.str, (char *)sb.str, MIN(sa.size, sb.size));
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    fprintf(stderr, "Missing input file\n");
    return EXIT_FAILURE;
  }
  i64 fsize = get_file_size(argv[1]);
  if (fsize < 0)
    return EXIT_FAILURE;

  mem_arena *perm_arena = arena_create(5 * fsize);
  printf("file size=%ld, arena->size=%lu\n", fsize, perm_arena->size);

  string8 file = {0};
  str_read_file(perm_arena, &file, argv[1]);

  vector *vec = VEC_ARENA_CREATE(perm_arena, string8);
  hash_map *hm = STRING_HASHMAP_ARENA(perm_arena, string8);

  split(vec, file, STR8_LIT("@"));
  string8 *entries = (string8 *)vec->data;
  u64 n_citation = vec->size;
  string8 *keys = ALLOC_ARRAY(perm_arena, string8, n_citation);
  string8 splitted[2] = {0};
  for (u64 i = 0; i < n_citation; ++i) {
    str_split_once(splitted, entries[i], STR8_LIT("{"));
    u64 n = str_contains(splitted[1], STR8_LIT(","));
    string8 name = (string8){.str = splitted[1].str, .size = n};
    string8 value =
        (string8){.str = entries[i].str - 1, .size = entries[i].size + 1};
    keys[i] = str_to_lowercase(perm_arena, name);
    hm_put(hm, &keys[i], &value);
  }
  vector_free(vec);

  qsort(keys, n_citation, sizeof(string8), cmp_fn);

  FILE *fp = fopen("sorted.bib", "w");
  if (fp == NULL) {
    fprintf(stderr, "Could not open file sorted.bib\n");
    return EXIT_FAILURE;
  }
  for (u64 i = 0; i < n_citation; i++) {
    string8 citation = HM_GET(string8, hm, &keys[i]);
    fwrite(citation.str, 1, citation.size, fp);
  }
  fclose(fp);

  arena_destroy(perm_arena);

  return EXIT_SUCCESS;
}
