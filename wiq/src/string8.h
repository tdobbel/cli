#ifndef _STRING8_H_
#define _STRING8_H_

#include <ctype.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef uint64_t u64;
typedef uint8_t u8;
typedef u8 b8;

typedef struct {
  u8 *str;
  u64 size;
} string8;

#define STR8_LIT(s) ((string8){.str = (u8 *)(s), .size = strlen((s))})
#define STR8_FMT "%.*s"
#define STR8_UNWRAP(s) (int)(s).size, (char *)(s).str

string8 str_trim_left(string8 s);
string8 str_trim_right(string8 s);
string8 str_trim(string8 s);
string8 to_string8(const char *s);
b8 str_equal(string8 s1, string8 s2);

#ifdef STRING_IMPLEMENTATION

string8 str_trim_left(string8 s) {
  u64 i = 0;
  while (i < s.size && isspace(s.str[i])) {
    i++;
  }
  return (string8){.str = s.str + i, .size = s.size - i};
}

string8 str_trim_right(string8 s) {
  u64 i = s.size;
  while (i > 0 && isspace(s.str[i - 1])) {
    i--;
  }
  return (string8){.str = s.str, .size = i};
}

string8 str_trim(string8 s) { return str_trim_right(str_trim_left(s)); }

string8 to_string8(const char *s) {
  u64 size = strlen(s);
  u8 *str = (u8 *)malloc(strlen(s));
  memcpy(str, s, size);
  return (string8){.str = str, .size = size};
}

b8 str_equal(string8 s1, string8 s2) {
  if (s1.size != s2.size)
    return 0;
  for (u64 i = 0; i < s1.size; ++i) {
    if (s1.str[i] != s2.str[i])
      return 0;
  }
  return 1;
}

#endif
#endif
