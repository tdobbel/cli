default_target: main

CC = gcc
CFLAGS = -std=gnu11 -Wall -Wextra -pedantic
INCLUDE_FLAGS = -I/home/dobby/clib
SRC_DIR = src 

main: src/tiff_reader.c
	$(CC) $(INCLUDE_FLAGS) $(CFLAGS) $^ -o gtiff

