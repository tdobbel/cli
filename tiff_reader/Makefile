default_target: main

CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -pedantic
SRC_DIR = src 

main: src/arena.h src/tiff_reader.c
	$(CC) $(CFLAGS) $^ -o gtiff

