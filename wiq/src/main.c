#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define STRING_IMPLEMENTATION
#include "string8.h"
#define VECTOR_IMPLEMENTATION
#include "vector.h"
#define HASHMAP_IMPLEMENTATION
#include "hash_map.h"

#define BUFFER_SIZE 1024
#define COLOR_BOLD "\e[1m"
#define COLOR_OFF "\e[m"
#define ANSI_COLOR_RED "\x1b[31m"
#define ANSI_COLOR_GREEN "\x1b[32m"
#define ANSI_COLOR_YELLOW "\x1b[33m"
#define ANSI_COLOR_BLUE "\x1b[34m"
#define ANSI_COLOR_CYAN "\x1b[36m"
#define ANSI_COLOR_RESET "\x1b[0m"

#define MIN(a, b) ((a) < (b) ? (a) : (b))

typedef struct {
  string8 name;
  u64 running, pending;
  u8 n_partition;
  string8 partitions[20];
} q_user;

void str_read_queue(string8 *dst, const char *cmd);
q_user *add_user(hash_map *queue, string8 name);
void split_queue_line(const string8 s, string8 *parts);
u64 parse_jobid(const string8 s);
void add_partition(q_user *user, string8 parts);
u64 queue_build(hash_map *queue, string8 queue_output);
int compare_users(const void *a, const void *b);
void print_user(q_user *user);

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

  string8 queue_output = {0};
  str_read_queue(&queue_output, command);

  hash_map *queue = STRING_HASHMAP(q_user);
  u64 total = queue_build(queue, queue_output);

  if (total == 0) {
    printf("🥳🎉 There are no jobs in %s 🎉🥳\n", message_end);
  } else {
    q_user *users = (q_user *)hm_values(queue);
    qsort(users, queue->size, sizeof(q_user), compare_users);
    printf("There are %s%lu%s jobs in %s:\n", COLOR_BOLD, total, COLOR_OFF,
           message_end);
    for (u64 i = 0; i < queue->size; ++i) {
      print_user(&users[i]);
    }
    free(users);
  }
  hm_deinit(queue);
  free(queue_output.str);
  return EXIT_SUCCESS;
}

void str_read_queue(string8 *dst, const char *cmd) {
  memset(dst, 0, sizeof(string8));
  FILE *fp = popen(cmd, "r");
  if (fp == NULL) {
    fprintf(stderr, "Could not read queue\n");
    exit(1);
  }
  vector *output = VEC_CREATE(u8);
  char c;
  while ((c = fgetc(fp)) != EOF) {
    VEC_PUSH(output, u8, (u8)c);
  }
  u8 *str = (u8 *)malloc(output->size);
  memcpy(str, output->data, output->size);
  dst->str = str;
  dst->size = output->size;
  vector_free(output);
  fclose(fp);
}

void add_partition(q_user *user, string8 partition) {
  for (u8 i = 0; i < user->n_partition; ++i) {
    if (str_equal(user->partitions[i], partition))
      return;
  }
  user->partitions[user->n_partition] = partition;
  user->n_partition++;
}

u64 parse_jobid(const string8 s) {
  u64 start = str_contains(s, STR8_LIT("["));
  if (start == s.size)
    return 1;
  string8 block_str =
      (string8){.str = s.str + start + 1, .size = s.size - start - 2};

  vector *vec = VEC_CREATE(string8);
  split(vec, block_str, STR8_LIT(","));
  string8 *blocks = (string8 *)vec->data;
  u64 total = 0;
  for (u64 i = 0; i < vec->size; ++i) {
    string8 splitted[2];
    if (!str_split_once(splitted, blocks[i], STR8_LIT("-"))) {
      total++;
      continue;
    }
    u64 iend = str_contains(splitted[1], STR8_LIT("%"));
    string8 rhs = (string8){.str = splitted[1].str, .size = iend};
    u64 v0, v1;
    str_parse_unsigned(&v0, splitted[0]);
    str_parse_unsigned(&v1, rhs);
    total += (v1 - v0 + 1);
  }
  vector_free(vec);
  return total;
}

q_user *add_user(hash_map *queue, string8 name) {
  kv_entry entry = hm_get_or_put(queue, &name);
  q_user *user = (q_user *)entry.value_ptr;
  if (entry.found_existing) {
    return user;
  }
  user->name = name;
  user->pending = 0;
  user->running = 0;
  user->n_partition = 0;
  return user;
}

u64 queue_build(hash_map *queue, string8 queue_output) {
  vector *line_vec = VEC_CREATE(string8);
  split(line_vec, queue_output, STR8_LIT("\n"));
  string8 *lines = (string8 *)line_vec->data;
  vector *vec = VEC_CREATE(string8);
  vector *part_vec = VEC_CREATE(string8);
  u64 total = 0;
  for (u64 i = 0; i < line_vec->size; ++i) {
    split_whitespace(vec, lines[i]);
    string8 *content = (string8 *)vec->data;
    q_user *user = add_user(queue, content[0]);
    split(part_vec, content[2], STR8_LIT(","));
    string8 *partitions = (string8 *)part_vec->data;
    for (u64 ip = 0; ip < part_vec->size; ++ip) {
      add_partition(user, str_trim(partitions[ip]));
    }
    if (content[1].str[0] == 'R') {
      user->running++;
      total++;
    } else {
      u64 n_job = parse_jobid(content[3]);
      user->pending += n_job;
      total += n_job;
    }
  }
  vector_free(line_vec);
  vector_free(vec);
  vector_free(part_vec);
  return total;
}

int compare_users(const void *a, const void *b) {
  q_user *ua = (q_user *)a;
  q_user *ub = (q_user *)b;
  u64 total_a = ua->pending + ua->running;
  u64 total_b = ub->pending + ub->running;
  return total_b - total_a;
}

void print_user(q_user *user) {
  char username[13];
  u32 n = MIN(12, user->name.size);
  memcpy(username, user->name.str, n);
  username[n] = '\0';
  printf("-> %s%-12s%s: ", ANSI_COLOR_BLUE, username, ANSI_COLOR_RESET);
  printf("%s%5lu%s running, ", ANSI_COLOR_GREEN COLOR_BOLD, user->running,
         ANSI_COLOR_RESET COLOR_OFF);
  printf("%s%5lu%s pending  ", ANSI_COLOR_YELLOW COLOR_BOLD, user->pending,
         ANSI_COLOR_RESET COLOR_OFF);
  printf("(%s", ANSI_COLOR_CYAN);
  for (u8 i = 0; i < user->n_partition; ++i) {
    printf("%s" STR8_FMT, (i == 0 ? "" : ", "),
           STR8_UNWRAP(user->partitions[i]));
  }
  printf("%s)\n", ANSI_COLOR_RESET);
}
