#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
#include "/home/dry/Documents/Eggo/jaguar/build/__generics__.h"
extern void* malloc (jaguar_u64 bytes);
	vec_##T temp = (vec_##T) {.data = (T*)(malloc(24)),.len = 0,.cap = 0};