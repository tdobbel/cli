#include "bib.h"
#include <stdio.h>
#include <stdlib.h>

#define BUFFER_SIZE 1024

int main(int argc, char *argv[]) {
  if (argc < 2) {
    fprintf(stderr, "Missing input file\n");
    return EXIT_FAILURE;
  }

  Biblio *bib = readBibtex(argv[1]);
  sortBib(bib);
  writeBibtex(bib, "sorted.bib");

  freeBib(bib);

  return EXIT_SUCCESS;
}
