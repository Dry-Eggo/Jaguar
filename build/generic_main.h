#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"


#define GENERIC_BLOCK_OPT(T) typedef struct opt_##T {\
	T value; \
	jaguar_i32 is_good; \
} opt_##T; \
extern inline T opt_##T##_unwrap(opt_##T* self); \
extern inline jaguar_i32 opt_##T##_is_some(opt_##T* self); \
extern inline T opt_##T##_unwrap(opt_##T *self) { \
if (( self->is_good!= 1)){\
__panic(jformat("Unwrapping a bad option"));\
};	return self->value;}\
extern inline jaguar_i32 opt_##T##_is_some(opt_##T *self) { \
	return self->is_good;}\

#define GENERIC_BLOCK_RESULT(U,E) typedef struct result_##U##_##E {\
	U ok; \
	E err; \
} result_##U##_##E; \

#define GENERIC_FN_newopt(T) opt_##T newopt_##T (T i) {\
	return (opt_##T) {.value = i,.is_good = 1};\
}


#define GENERIC_BLOCK_VEC(T) typedef struct vec_##T {\
	T* data; \
	jaguar_u64 len; \
	jaguar_u64 cap; \
} vec_##T; \
extern inline void vec_##T##_push(vec_##T* self,T __n); \
extern inline T vec_##T##_at(vec_##T* self,jaguar_u64 __idx); \
extern inline T vec_##T##_pop(vec_##T* self); \
extern inline void vec_##T##_grow(vec_##T* self); \
extern inline jaguar_i32 vec_##T##_len(vec_##T* self); \
extern inline jaguar_i32 vec_##T##_capacity(vec_##T* self); \
GENERIC_BLOCK_OPT(T)\
extern inline opt_##T vec_##T##_get(vec_##T* self,jaguar_i32 __idx); \
extern inline void vec_##T##_push(vec_##T *self,T __n) { \
if (( self->len== self->cap)){\
({\
	vec_##T* __gbval0 = self;\
vec_##T##_grow(__gbval0);\
});\
};\
self->data[( self->len- 1)] = __n;\
self->len = ( self->len+ 1);}\
extern inline T vec_##T##_at(vec_##T *self,jaguar_u64 __idx) { \
if (( __idx>= ( self->len- 1))){\
__panic("Out of bounds: Vec<_>");\
};	return self->data[__idx];}\
extern inline T vec_##T##_pop(vec_##T *self) { \
if (( ( self->len- 1)<= 0)){\
__panic("pop called on empty vector: Vec<_>");\
};	return self->data[( self->len- 1)];}\
extern inline void vec_##T##_grow(vec_##T *self) { \
self->data = (T*)(realloc((void*)(self->data),( self->cap* 2)));\
self->cap = ( self->cap* 2);}\
extern inline jaguar_i32 vec_##T##_len(vec_##T *self) { \
	return self->len;}\
extern inline jaguar_i32 vec_##T##_capacity(vec_##T *self) { \
	return self->cap;}\
extern inline opt_##T vec_##T##_get(vec_##T *self,jaguar_i32 __idx) { \
	return (opt_##T) {.value = self->data[__idx]};}\

#define GENERIC_FN_newVec(T) vec_##T newVec_##T () {\
	return (vec_##T) {.data = (T*)(malloc(( 8* 8))),.len = 1,.cap = 8};\
}
