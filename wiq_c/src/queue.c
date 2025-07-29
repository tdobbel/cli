#include "queue.h"
#include <ctype.h>
#include <stdio.h>
#include <string.h>

#define BUFFER_SIZE 1024

Queue *createQueue() {
  Queue *queue = malloc(sizeof(Queue));
  queue->capacity = 100;
  queue->size = 0;
  queue->users = malloc(queue->capacity * sizeof(User));
  return queue;
}

void freeQueue(Queue *queue) {
  free(queue->users);
  free(queue);
}

static void addUser(Queue *queue, const char *name) {
  if (queue->size >= queue->capacity) {
    queue->capacity *= 2;
    queue->users = realloc(queue->users, queue->capacity * sizeof(User));
  }
  User *user = &queue->users[queue->size++];
  strncpy(user->name, name, 20);
  user->running = 0;
  user->pending = 0;
  user->num_partitions = 0;
}

static User *getUser(Queue *queue, const char *name) {
  for (size_t i = 0; i < queue->size; i++) {
    if (strcmp(queue->users[i].name, name) == 0)
      return queue->users + i;
  }
  addUser(queue, name);
  return queue->users + queue->size - 1;
}

static void addPartition(User *user, const char *partition) {
  for (int i = 0; i < user->num_partitions; i++) {
    if (strcmp(user->partitions[i], partition) == 0)
      return;
  }
  strcpy(user->partitions[user->num_partitions++], partition);
}

static void processPartitions(User *user, const char *partitions) {
  int start = 0, j = 0;
  char part[6];
  while (partitions[j] != '\0') {
    if (partitions[j] == ',') {
      part[j - start] = '\0';
      addPartition(user, part);
      start = j + 1;
    } else {
      part[j - start] = partitions[j];
    }
    j++;
  }
  part[j - start] = '\0';
  addPartition(user, part);
}

static void getAddedPending(char *jobid, int *added) {
  (*added)++;
  int start = strcspn(jobid, "[") + 1;
  int end = strcspn(jobid, "]");
  if (start >= strlen(jobid) || end >= strlen(jobid) || start >= end)
    return;
  int bounds[2] = {0, 0};
  int k = 0;
  for (int i = start; i < end; i++) {
    if (jobid[i] == '%')
      break;
    if (jobid[i] == '-') {
      k++;
      continue;
    }
    if (jobid[i] < '0' || jobid[i] > '9')
      return;
    bounds[k] = 10 * bounds[k] + jobid[i] - '0';
  }
  if (bounds[1] > bounds[0])
    *added += bounds[1] - bounds[0];
}

static int processLine(char *buffer, Queue *queue) {
  int added = 0;
  size_t n = strcspn(buffer, "\n");
  buffer[n] = '\0';
  int i = 0;
  while (isspace(buffer[i])) {
    i++;
  }
  char *line = buffer + i;
  User *user = getUser(queue, strtok(line, " "));
  char *state = strtok(NULL, " ");
  processPartitions(user, strtok(NULL, " "));
  if (strcmp(state, "R") == 0) {
    user->running++;
    added++;
  } else if (strcmp(state, "PD") == 0) {
    getAddedPending(strtok(NULL, " "), &added);
    user->pending += added;
  }
  return added;
}

int buildQueue(char *command, Queue *queue) {
  int total = 0;
  FILE *file = popen(command, "r");
  char buffer[BUFFER_SIZE];
  while (fgets(buffer, BUFFER_SIZE, file) != NULL) {
    total += processLine(buffer, queue);
  }
  pclose(file);
  return total;
}

static int compareUsers(const void *a, const void *b) {
  User *userA = (User *)a;
  User *userB = (User *)b;
  int totalA = userA->running + userA->pending;
  int totalB = userB->running + userB->pending;
  return totalB - totalA;
}

void sortQueue(Queue *queue) {
  qsort(queue->users, queue->size, sizeof(User), compareUsers);
}

void joinUserPartitions(User *user, char *result) {
  for (int i = 0; i < user->num_partitions; i++) {
    if (i == 0) {
      sprintf(result, "%s", user->partitions[i]);
    } else {
      sprintf(result, "%s, %s", result, user->partitions[i]);
    }
  }
}
