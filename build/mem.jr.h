#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
#include "/home/dry/Documents/Eggo/jaguar/build/__generics__.h"
extern void* malloc (jaguar_u64 bytes);
extern void* realloc (void* pointer,jaguar_u64 bytes);
typedef struct Allocator {

	void** data;
	jaguar_u64 topi;
	void* top;
	void* bottom;
	jaguar_u64 cap;} Allocator;
extern inline void* Allocator_allocate(Allocator* self,jaguar_u64 bytes);
extern inline void Allocator_grow(Allocator* self);
extern inline void* Allocator_allocate(Allocator *self,jaguar_u64 bytes) { 
if (( self->topi== self->cap)){
({
	Allocator* __gbval0 = self;
Allocator_grow(__gbval0);
});
};}
extern inline void Allocator_grow(Allocator *self) { 
realloc((void*)(self->data),( self->cap* 2));\
self->cap = ( self->cap* 2);}