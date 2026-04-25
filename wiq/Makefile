default_target: main

CC = gcc
CFLAGS = -std=gnu11 -Wall -Wextra -pedantic -O3
SRC_DIR = src
INC = -I/export/homes/dobbelaeret/clib

main: src/main.c
	$(CC) $(CFLAGS) $(INC) $^ -o cwiq
