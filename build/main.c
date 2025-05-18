
#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
#include "/home/dry/Documents/Eggo/jaguar/build/__generics__.h"
extern void jprintln (jaguar_str fmt, ...);
extern jaguar_str input (jaguar_i32 bytes,jaguar_str prompt);
extern void __panic (jaguar_str msg);
extern jaguar_str jformat (jaguar_str fmt, ...);
extern void println (jaguar_str msg);
extern void* malloc (jaguar_u64 bytes);
extern void* realloc (void* pointer,jaguar_u64 bytes);

jaguar_i32 main () {vec_jaguar_str temp = newVec_jaguar_str();({
	vec_jaguar_str* __gbval1 = &temp;
vec_jaguar_str_push(__gbval1,"My name is abdul");
});jaguar_str t = ({
	vec_jaguar_str* __gbval2 = &temp;
vec_jaguar_str_at(__gbval2,0);
});println(t);
	return (jaguar_i32)0;
	return 0;
}
