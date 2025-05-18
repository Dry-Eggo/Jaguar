#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
extern void* malloc (jaguar_u64 bytes);
extern void* realloc (void* __n,jaguar_u64 bytes);
extern void jprintln (jaguar_str fmt, ...);
typedef struct Array {

	void** data;
	jaguar_u64 len;
	jaguar_u64 cap;} Array;
extern inline jaguar_i32 Array_push(Array* self,void* n);
extern inline jaguar_i32 Array_grow(Array* self);
extern inline void* Array_at(Array* self,jaguar_u64 n);

#define GENERIC_BLOCK_VEC(T) typedef struct vec_##T {\
	T* data; \
} vec_##T;
GENERIC_BLOCK_VEC(T);



extern inline jaguar_i32 Array_push(Array *self,void* n) {
if (( self->len== self->cap)){

	({
	Array* __gbval0 = self;
Array_grow(__gbval0);
});
}
	self->data[( self->len- 1)] = n;
	self->len = ( self->len+ 1);
	return 0;
}


extern inline jaguar_i32 Array_grow(Array *self) {
	self->data = (void**)(realloc((void*)(self->data),( self->cap* 2)));
	self->cap = ( self->cap* 2);
}


extern inline void* Array_at(Array *self,jaguar_u64 n) {
	return self->data[n];
}

extern inline Array new_array () {
	Array* a = (Array*)(malloc(24));
	a->data = (void**)(malloc(( 8* 8)));
	a->cap = ( 8* 8);
	a->len = 1;
	return *a;
}


#define GENERIC_FN_newvec(T) extern inline vec_##T newvec () {
	vec_T tmp = {.data = (T*)(malloc(64))};\
	return tmp;\
}
