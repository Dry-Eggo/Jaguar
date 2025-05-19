#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
#include "/home/dry/Documents/Eggo/jaguar/build/__generics__.h"
extern void* malloc (jaguar_u64 bytes);
extern void* realloc (void* pointer,jaguar_u64 bytes);
extern void __panic (jaguar_str msg);