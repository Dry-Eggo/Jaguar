
#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
#include "/home/dry/Documents/Eggo/jaguar/build/__generics__.h"
extern void jprintln (jaguar_str fmt, ...);
#include "/home/dry/Documents/Eggo/jaguar/build/mem.jr.h"

jaguar_i32 main () {Allocator allocator = 
	new_allocator(64);jaguar_i32* test = (jaguar_i32*)(({
	Allocator* __gbval0 = &allocator;
Allocator_allocate(__gbval0,8);
}));*test = 5
;jprintln("{d}",*test);
	return 0;
}
