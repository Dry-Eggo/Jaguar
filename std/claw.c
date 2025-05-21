#include "claw.h"
#include <stdarg.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
extern void jaguar_rest(jaguar_int exit_code);
extern char char_to_upper(char *self) {
  if (*self >= 'a' && *self <= 'z')
    return *self - 32;
  return *self;
}
extern char char_to_lower(char *self) {
  if (*self >= 'A' && *self <= 'Z')
    return *self + 32;
  return *self;
}
extern void rzapp(char *dst, const char *src) {
  char *tmp = dst;
  dst = malloc(1024);
  strcat(dst, tmp);
  strcat(dst, src);
}
void claw_itoa(jaguar_int val, char *buffer) {
  int i = 0;
  int is_negatve = 0;

  if (val < 0) {
    is_negatve = 1;
    val = -val;
  }

  do {
    buffer[i++] = (val % 10) + '0';
    val /= 10;
  } while (val > 0);
  if (is_negatve) {
    buffer[i++] = '-';
  }

  int left = 0;
  int right = i - 1;
  while (left < right) {
    char tmp = buffer[left];
    buffer[left] = buffer[right];
    buffer[right] = tmp;
    left++;
    right--;
  }
  buffer[i] = '\0';
}
void print_int(jaguar_int i) {
  char *buf = mem_get(8);
  claw_itoa(i, buf);
  jaguar_str d = buf;
  print(d);
}

extern int str_eq(jaguar_str s1, jaguar_str s2) { return (strcmp(s1, s2)); }

void __panic(jaguar_str errmsg) {
  jaguar_str d = "[Jaguar panicked]: ";
  print(d);
  jaguar_str e = errmsg;
  println(e);
  jaguar_rest(1);
}
/*
 * String formatter for the jaguar backend.
 * e.g let foo:str = jformat("Hello {s}\n", "World");
 * asser_eq(foo, "Hello World");
 */
extern jaguar_str jformat(jaguar_str fmt, ...) {
  char buf[1024];
  va_list args;
  va_start(args, fmt);
  int len = 0;
  for (int i = 0; fmt[i]; i++) {
    if (fmt[i] == '{') {
      i++;
      if (fmt[i] == '{') {
        i++;
        buf[len++] = '{';
        continue;
      }
      if (fmt[i] == 's' && fmt[i + 1] == '}') {
        jaguar_str arg = va_arg(args, jaguar_str);
        len += snprintf(buf + len, sizeof(buf) - len, "%s", arg);
        i += 1;
      } else if (fmt[i] == 'd' && fmt[i + 1] == '}') {
        jaguar_int arg = va_arg(args, jaguar_int);
        len += snprintf(buf + len, sizeof(buf) - len, "%ld", arg);
        i += 1;
      } else if (fmt[i] == 'p' && fmt[i + 1] == '}') {
        void *arg = va_arg(args, void *);
        len += snprintf(buf + len, sizeof(buf) - len, "%p", arg);
        i += 1;
      } else if (fmt[i] == 'c' && fmt[i + 1] == '}') {
        char arg = va_arg(args, int);
        len += snprintf(buf + len, sizeof(buf) - len, "%c", arg);
        i += 1;
      } else {
        len += snprintf(buf + len, sizeof(buf) - len, "{?");
      }
    } else {
      buf[len++] = fmt[i];
    }
  }
  buf[len] = '\0';
  va_end(args);
  return strdup(buf);
}
/*
 * jprintln:
 *    fmt is passed to jformat for formatting and then printed to stdout with a
 * newline
 */
void jprintln(jaguar_str fmt, ...) {
  jaguar_str p = jformat(fmt);
  println(p);
}
extern jaguar_str jinput(jaguar_str prompt) {
  print(prompt);
  char buffer[1024];
  scanf("%s", buffer);
  return strdup(buffer);
}
void panic(jaguar_str errmsg, jaguar_int LINE) {
  print(jformat("[Tixie Panicked][line: {d}]: ", LINE));
  println(errmsg);
  jaguar_rest(100);
}
void write_ch(char c) {
  jaguar_str d = &c;
  println(d);
}
// string merger
jaguar_str strmrg(jaguar_str c, jaguar_str d) {
  char dup[1024] = {0};
  strcat(dup, c);
  strcat(dup, d);
  return strdup(dup);
}
jaguar_str strslice(jaguar_str sub, int n, int pos) {
  char buf[1024] = {0};
  int len = 0;
  for (int i = n; i < pos; i++) {
    buf[len++] = jaguar_str_at(sub, i);
  }
  return strdup(buf);
}
