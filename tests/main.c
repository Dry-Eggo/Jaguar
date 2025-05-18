
#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
extern void jprintln (jaguar_str fmt, ...);
extern jaguar_str input (jaguar_i32 bytes,jaguar_str prompt);
#include "/home/dry/Documents/Eggo/jaguar/build/test.jr.h"

#define GENERIC_BLOCK_OPT(T) typedef struct opt_##T {\
	T value; \
} opt_##T;

GENERIC_FN_newopt(jaguar_i32);



#define GENERIC_FN_newopt(T) opt_##T newopt (T i) {\
	return {.value = i};\
}

jaguar_i32 main () {

GENERIC_FN_newopt(jaguar_i32)


	opt_jaguar_i32 some = ;
	opt_jaguar_str other = {.value = "hello"};
	jprintln("generic : {s}",other.value);
	jaguar_i32 sum = ( some.value+ 4);
	return 0;
	return 0;
}
