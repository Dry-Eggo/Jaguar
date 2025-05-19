#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
#include "/home/dry/Documents/Eggo/jaguar/build/__generics__.h"
extern void* malloc (jaguar_u64 bytes);
extern void* realloc (void* pointer,jaguar_u64 bytes);
typedef struct Allocator {

	void* start;
	void* current;
	jaguar_u64 capacity;} Allocator;
extern inline void* Allocator_allocate(Allocator* self,jaguar_u64 bytes);
extern inline void Allocator_grow(Allocator* self,jaguar_u64 bytes);
extern inline void* Allocator_allocate(Allocator *self,jaguar_u64 bytes) {
jaguar_i32 new_current = ( (jaguar_u64)(self->current)+ bytes);\
if (( new_current> ( (jaguar_u64)(self->start)+ self->capacity))){
({
	Allocator* __gbval0 = self;
Allocator_grow(__gbval0,bytes);
});new_current = ( (jaguar_u64)(self->current)+ bytes)
;
};\
void* allocated_ptr = self->current;\
self->current = (void*)(new_current);\
return (void*) allocated_ptr;}
extern inline void Allocator_grow(Allocator *self,jaguar_u64 bytes) {
jaguar_i32 newcap = ( self->capacity* 2);\
if (( newcap< ( self->capacity+ bytes))){
newcap = ( self->capacity+ bytes)
;
};\
void* new_start = realloc(self->start,newcap);\
jaguar_i32 offset = ( (jaguar_u64)(self->current)- (jaguar_u64)(self->start));\
self->start = new_start;\
self->current = (void*)(( (jaguar_u64)(self->start)+ offset));\
self->capacity = newcap;}

extern inline Allocator new_allocator (jaguar_u64 capacity) {void* p = malloc(capacity);return (Allocator) {.start = p,.current = p,.capacity = capacity};
}
