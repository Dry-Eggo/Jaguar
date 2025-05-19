#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"


#define GENERIC_BLOCK_VEC(T) typedef struct vec_##T {\
	T* data; \
	jaguar_u64 len; \
	jaguar_u64 cap; \
} vec_##T; \
extern inline void vec_##T##_push(vec_##T* self,_##T __n); \
extern inline T vec_##T##_at(vec_##T* self,jaguar_u64 __idx); \
extern inline T vec_##T##_pop(vec_##T* self); \
extern inline void vec_##T##_grow(vec_##T* self); \
extern inline jaguar_i32 vec_##T##_len(vec_##T* self); \
extern inline jaguar_i32 vec_##T##_capacity(vec_##T* self); \
extern inline void vec_##T##_push(vec_##T *self,_##T __n) { \
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
};\
return self->data[__idx];}\
extern inline T vec_##T##_pop(vec_##T *self) { \
if (( ( self->len- 1)<= 0)){\
__panic("pop called on empty vector: Vec<_>");\
};\
return self->data[( self->len- 1)];}\
extern inline void vec_##T##_grow(vec_##T *self) { \
self->data = (T*)(realloc((void*)(self->data),( self->cap* 2)));\
self->cap = ( self->cap* 2);}\
extern inline jaguar_i32 vec_##T##_len(vec_##T *self) { \
return self->len;}\
extern inline jaguar_i32 vec_##T##_capacity(vec_##T *self) { \
return self->cap;}\

#define GENERIC_FN_newVec(T) extern inline vec_##T newVec_##T () {\
return (vec_##T) {.data = (T*)(malloc(( 8* 8))),.len = 1,.cap = 8};\
}
