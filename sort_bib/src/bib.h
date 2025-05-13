#ifndef BIB_H
#define BIB_H

typedef struct {
  char name[100];
  int size;
  int capacity;
  char *content;
} Citation;

typedef struct {
  int capacity, size;
  Citation *citations;
} Biblio;

int compareCitations(const void *a, const void *b);
void addToCitation(Citation *c, char *line);
Biblio *createBib();
Biblio *readBibtex(char *filename);
void sortBib(Biblio *bib);
void writeBibtex(Biblio *bib, char *filename);
void freeBib(Biblio *bib);

#endif
