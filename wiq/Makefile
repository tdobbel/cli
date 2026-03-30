default_target: main

CC = gcc
CFLAGS = -std=gnu11 -Wall -Wextra -pedantic -O3
SRC_DIR = src 

main: src/string8.h src/arena.h src/collections.h src/main.c
	$(CC) $(CFLAGS) $^ -o cwiq
