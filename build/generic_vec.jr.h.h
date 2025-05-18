#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"


#define GENERIC_BLOCK_VEC(T) typedef struct vec_##T {\
	T* data; \
	jaguar_u64 len; \
	jaguar_u64 cap; \
} vec_##T; \

#define GENERIC_FN_newvec(T) extern inline vec_##T newvec_##T () {\
	return temp;\
}
