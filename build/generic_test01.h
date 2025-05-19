#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"


#define GENERIC_BLOCK_TESTER_STRUCT(T) typedef struct tester_struct_##T {\
} tester_struct_##T; \
extern inline void tester_struct_##T##_push(tester_struct_##T* self,_##T __n); \
extern inline void tester_struct_##T##_push(tester_struct_##T *self,_##T __n) { \
}\