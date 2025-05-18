// claw : the jaguar runtime

#pragma once
#include "stdlib.h"
// compiler handled type
#include <stdint.h>
typedef signed long jaguar_int;
typedef int jaguar_i32;
typedef uint8_t jaguar_u8;
typedef uint16_t jaguar_u16;
typedef uint32_t jaguar_u32;
typedef uint64_t jaguar_u64;
typedef int8_t jaguar_i8;
typedef int16_t jaguar_i16;
typedef int32_t jaguar_i32;
typedef int64_t jaguar_i64;
typedef char *jaguar_str;
typedef float jaguar_float;
extern void print(jaguar_str msg);
extern void println(jaguar_str msg);
void panic(jaguar_str errmsg, jaguar_i64 LINE);
extern void jaguar_rest(jaguar_int err_code);
extern void write_int(jaguar_i32 i);
extern void *mem_get(jaguar_int bytes);
extern jaguar_int str_len(const char *);
extern char char_to_upper(char *self);
extern char char_to_lower(char *self);
extern jaguar_str jformat(jaguar_str fmt, ...);
void claw_itoa(jaguar_int i, char *buf);
void print_int(jaguar_int i);
void __panic(jaguar_str msg);
void write_ch(char c);

// for manual creation of jaguar list<T,N> types
// helper functions will be added for appending and indexing

#define jaguar_list(T, N)                                                      \
  typedef struct jaguar_list_##T {                                             \
    T data[N];                                                                 \
    jaguar_int len;                                                            \
  } jaguar_list_##T;
#define jaguar_bounds_check(list, N)                                           \
  if (N >= list.len) {                                                         \
    panic("Tixie runtime: Out of bounds error");                               \
    jaguar_rest(100);                                                          \
  }

#define jaguar_list_at(list, N)                                                \
  ({                                                                           \
    typeof(list) _lst = (list);                                                \
    int _idx = (N);                                                            \
    if (_idx < 0 || _idx >= _lst.len) {                                        \
      __panic("Tixie runtime: Index out of bounds");                           \
    }                                                                          \
    _lst.data[_idx];                                                           \
  })
#define jaguar_deftype(T) static const char *jaguar_type_##T = #T;
#define jaguar_str_at(str, N)                                                  \
  ({                                                                           \
    jaguar_str _s = (str);                                                     \
    int _idx = (N);                                                            \
    if (_idx < 0 || _idx >= str_len(_s)) {                                     \
      __panic("Tixie Runtime: Out of bounds error");                           \
    }                                                                          \
    _s[_idx];                                                                  \
  })
#define typename(T) jaguar_type_##T
// ToDo: Init argc, argv, envp
