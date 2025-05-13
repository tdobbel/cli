#include "bib.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define BUFFER_SIZE 1024
#define CITATION_SIZE 512

int compareCitations(const void *a, const void *b) {
  return strcmp(((Citation *)a)->name, ((Citation *)b)->name);
}

void addToCitation(Citation *c, char *line) {
  int n = strcspn(line, "\n") + 1;
  int new_size = c->size + n;
  if (new_size > c->capacity) {
    c->capacity *= 2;
    c->content = (char *)realloc(c->content, c->capacity * sizeof(char));
  }
  strncpy(c->content + c->size, line, n);
  c->size = new_size;
}

Biblio *createBib() {
  Biblio *bib = malloc(sizeof(Biblio));
  bib->capacity = 100;
  bib->size = 0;
  bib->citations = malloc(bib->capacity * sizeof(Citation));
  return bib;
}

Citation *newCitation(Biblio *bib) {
  if (bib->size >= bib->capacity) {
    bib->capacity *= 2;
    bib->citations = realloc(bib->citations, bib->capacity * sizeof(Citation));
  }
  Citation *c = &bib->citations[bib->size++];
  c->size = 0;
  c->capacity = CITATION_SIZE;
  c->content = malloc(c->capacity * sizeof(char));
  return c;
}

void sortBib(Biblio *bib) {
  qsort(bib->citations, bib->size, sizeof(Citation), compareCitations);
}

Biblio *readBibtex(char *filename) {
  FILE *file = fopen(filename, "r");
  if (file == NULL) {
    fprintf(stderr, "Could not open file %s\n", filename);
    return NULL;
  }

  char buffer[BUFFER_SIZE];
  Biblio *bib = createBib();
  Citation *current = NULL;
  while (fgets(buffer, BUFFER_SIZE, file)) {
    int n = strcspn(buffer, "\n");
    if (n == 0) {
      continue;
    }
    if (buffer[0] == '@') {
      current = newCitation(bib);
      int start = strcspn(buffer, "{") + 1;
      strncpy(current->name, buffer + start, n - start - 1);
    }
    addToCitation(current, buffer);
  }
  fclose(file);
  return bib;
}

void writeBibtex(Biblio *bib, char *filename) {
  FILE *file = fopen(filename, "w");
  if (file == NULL) {
    fprintf(stderr, "Could not open file %s\n", filename);
    return;
  }
  for (int i = 0; i < bib->size; i++) {
    Citation *c = bib->citations + i;
    fwrite(c->content, sizeof(char), c->size, file);
  }
  fclose(file);
}

void freeBib(Biblio *bib) {
  for (int i = 0; i < bib->size; i++) {
    free(bib->citations[i].content);
  }
  free(bib->citations);
  free(bib);
}
