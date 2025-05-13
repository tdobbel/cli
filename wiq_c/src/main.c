#include "queue.h"
#include <stdio.h>

#define BUFFER_SIZE 1024
#define COLOR_BOLD  "\e[1m"
#define COLOR_OFF   "\e[m"
#define ANSI_COLOR_RED     "\x1b[31m"
#define ANSI_COLOR_GREEN   "\x1b[32m"
#define ANSI_COLOR_YELLOW  "\x1b[33m"
#define ANSI_COLOR_BLUE    "\x1b[34m"
#define ANSI_COLOR_CYAN    "\x1b[36m"
#define ANSI_COLOR_RESET   "\x1b[0m"


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
  Queue *queue = createQueue();
  int total = buildQueue(command, queue);

  if (total == 0) {
    printf("🥳🎉 There are no jobs in %s 🎉🥳\n", message_end);
  } else {
    sortQueue(queue);
    printf("There are %s%d%s jobs in %s:\n", COLOR_BOLD, total, COLOR_OFF, message_end);
    char usedPartitons[200];
    for (size_t i = 0; i < queue->size; i++) {
      User *user = &queue->users[i];
      joinUserPartitions(user, usedPartitons);
      printf("-> %s%-12s%s: ", ANSI_COLOR_BLUE, user->name, ANSI_COLOR_RESET);
      printf("%s%4d%s running, ", ANSI_COLOR_GREEN COLOR_BOLD, user->running, ANSI_COLOR_RESET COLOR_OFF);
      printf("%s%4d%s pending  ", ANSI_COLOR_YELLOW COLOR_BOLD, user->pending, ANSI_COLOR_RESET COLOR_OFF);
      printf("(%s%s%s)\n", ANSI_COLOR_CYAN, usedPartitons, ANSI_COLOR_RESET);
    }
  }

  freeQueue(queue);
  return EXIT_SUCCESS;
}
