#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define STRING_IMPLEMENTATION
#include "string8.h"
#define ARENA_IMPLEMENTATION
#include "arena.h"
#define HASHMAP_IMPLEMENTATION
#include "collections.h"

#define BUFFER_SIZE 1024
#define COLOR_BOLD "\e[1m"
#define COLOR_OFF "\e[m"
#define ANSI_COLOR_RED "\x1b[31m"
#define ANSI_COLOR_GREEN "\x1b[32m"
#define ANSI_COLOR_YELLOW "\x1b[33m"
#define ANSI_COLOR_BLUE "\x1b[34m"
#define ANSI_COLOR_CYAN "\x1b[36m"
#define ANSI_COLOR_RESET "\x1b[0m"

typedef struct {
  string8 name;
  u64 running, pending;
  u8 n_partition;
  string8 partitions[20];
} user_t;

string8 str_clone(mem_arena *arena, string8 s);
void split_queue_line(const string8 s, string8 *parts);
u64 parse_jobid(const string8 s);
void add_partition(user_t *user, mem_arena *arena, string8 parts);
u64 queue_build(hash_map *queue, mem_arena *arena, char *command);
int compare_users(const void *a, const void *b);
void print_user(user_t user);

int main(int argc, char *argv[]) {
  const char *baseCommand = "squeue --noheader -o '%.20u %t %P %i'";
  char command[100];
  char message_end[100];
  if (argc == 1) {
    sprintf(message_end, "the queue");
    sprintf(command, "%s", baseCommand);
  } else {
    sprintf(message_end, "partition %s", argv[1]);
    sprintf(command, "%s -p %s", baseCommand, argv[1]);
  }

  hash_map *queue = STRING_HASHMAP(user_t);
  mem_arena *perm_arena = arena_create(KiB(500));

  u64 total = queue_build(queue, perm_arena, command);

  if (total == 0) {
    printf("🥳🎉 There are no jobs in %s 🎉🥳\n", message_end);
  } else {
    user_t *users = PUSH_ARRAY(perm_arena, user_t, queue->size);
    kv_iterator kvi = hm_iterator(queue);
    u64 indx = 0;
    while (get_next(&kvi)) {
      users[indx++] = *(user_t *)kvi.value_ptr;
    }
    qsort(users, queue->size, sizeof(user_t), compare_users);
    printf("There are %s%lu%s jobs in %s:\n", COLOR_BOLD, total, COLOR_OFF,
           message_end);
    for (u64 i = 0; i < queue->size; ++i) {
      print_user(users[i]);
    }
  }

  hm_deinit(queue);
  arena_destroy(perm_arena);
  return EXIT_SUCCESS;
}

string8 str_clone(mem_arena *arena, string8 s) {
  u8 *str = PUSH_ARRAY(arena, u8, s.size);
  memcpy(str, s.str, s.size);
  return (string8){.str = str, .size = s.size};
}

void split_queue_line(const string8 s, string8 *parts) {
  u64 start = 0, end = 0;
  for (u32 i = 0; i < 4; ++i) { // assume line contains 4 entries
    while (end < s.size && !isspace(s.str[end])) {
      end++;
    }
    parts[i] = (string8){.str = s.str + start, .size = end - start};
    start = end + 1;
    end = start;
  }
}

void add_partition(user_t *user, mem_arena *arena, string8 parts) {
  string8 s = str_trim(parts);
  while (s.size > 0) {
    u64 i = 0;
    while (i < s.size && s.str[i] != ',') {
      i++;
    }
    string8 p_ = str_trim((string8){.str = s.str, .size = i});
    string8 p = str_clone(arena, p_);
    b8 found = 0;
    for (u32 ip = 0; ip < user->n_partition && !found; ++ip) {
      found = str_equal(p, user->partitions[ip]);
    }
    if (!found) {
      user->partitions[user->n_partition++] = p;
    }
    if (i == s.size)
      return;
    i++; // character after comma
    s = (string8){.str = s.str + i, .size = s.size - i};
  }
}

u64 parse_jobid(const string8 s) {
  u64 i = 0;
  while (i < s.size && s.str[i] != '[') {
    i++;
  }
  if (i == s.size)
    return 1;
  u64 start = i + 1;
  u64 end = start;
  u64 njob = 0;
  while (end < s.size - 1) {
    // blocks are separated by commas
    while (end < s.size - 1 && s.str[end] != ',') {
      end++;
    }
    u64 i = start;
    u64 v0 = 0;
    while (i < end & s.str[i] != '-') {
      v0 = v0 * 10 + (u64)(s.str[i] - '0');
      i++;
    }
    if (i == end) {
      njob++;
    } else {
      u64 v1 = 0;
      i++;
      while (i < end && s.str[i] != '%') {
        v1 = v1 * 10 + (u64)(s.str[i] - '0');
        i++;
      }
      njob += v1 - v0 + 1;
    }
    start = end + 1;
    end = start;
  }
  return njob;
}

u64 queue_build(hash_map *queue, mem_arena *arena, char *command) {
  u64 total = 0;
  FILE *file = popen(command, "r");
  char buffer[BUFFER_SIZE];
  string8 content[4];
  while (fgets(buffer, BUFFER_SIZE, file) != NULL) {
    int n = strcspn(buffer, "\n");
    buffer[n] = '\0';
    string8 line = STR8_LIT(buffer);
    split_queue_line(str_trim(line), content);
    kv_entry entry = hm_get_or_put(queue, &content[0]);
    user_t *user = (user_t *)entry.value_ptr;
    if (!entry.found_existing) {
      string8 name = str_clone(arena, content[0]);
      string8 *key = (string8 *)entry.key_ptr;
      key->str = name.str;
      user->name = name;
      user->pending = 0;
      user->running = 0;
      user->n_partition = 0;
    }
    add_partition(user, arena, content[2]);
    u8 status = content[1].str[0];
    u64 njob = parse_jobid(content[3]);

    if (status == 'R') {
      user->running += njob;
    } else {
      user->pending += njob;
    }
    total += njob;
  }
  pclose(file);
  return total;
}

int compare_users(const void *a, const void *b) {
  user_t *ua = (user_t *)a;
  user_t *ub = (user_t *)b;
  u64 total_a = ua->pending + ua->running;
  u64 total_b = ub->pending + ub->running;
  return total_b - total_a;
}

void print_user(user_t user) {
  char user_partitions[200];
  char username[13];
  memcpy(username, user.name.str, 12);
  username[12] = '\0';
  u64 ic = 0;
  for (u64 ip = 0; ip < user.n_partition; ++ip) {
    if (ip > 0) {
      memcpy(user_partitions + ic, ", ", 2);
      ic += 2;
    }
    string8 p = user.partitions[ip];
    memcpy(user_partitions + ic, p.str, p.size);
    ic += p.size;
  }
  user_partitions[ic] = '\0';
  printf("-> %s%-12s%s: ", ANSI_COLOR_BLUE, username, ANSI_COLOR_RESET);
  printf("%s%4lu%s running, ", ANSI_COLOR_GREEN COLOR_BOLD, user.running,
         ANSI_COLOR_RESET COLOR_OFF);
  printf("%s%4lu%s pending  ", ANSI_COLOR_YELLOW COLOR_BOLD, user.pending,
         ANSI_COLOR_RESET COLOR_OFF);
  printf("(%s%s%s)\n", ANSI_COLOR_CYAN, user_partitions, ANSI_COLOR_RESET);
}
