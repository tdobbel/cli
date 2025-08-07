#ifndef QUEUE_H
#define QUEUE_H

#include <stdlib.h>

typedef struct {
  char name[20];
  int running, pending;
  int num_partitions;
  char partitions[10][20];
} User;

typedef struct {
  size_t size;
  size_t capacity;
  User *users;
} Queue;

Queue *createQueue();
void freeQueue(Queue *queue);
int buildQueue(char *command, Queue *queue);
void sortQueue(Queue *queue);

void joinUserPartitions(User *user, char *buffer);

#endif
