
#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
#include "/home/dry/Documents/Eggo/jaguar/build/__generics__.h"
extern void jprintln (jaguar_str fmt, ...);
extern jaguar_str input (jaguar_i32 bytes,jaguar_str prompt);
extern void __panic (jaguar_str msg);
extern jaguar_str jformat (jaguar_str fmt, ...);
extern void println (jaguar_str msg);
extern void* malloc (jaguar_u64 bytes);
extern void* realloc (void* pointer,jaguar_u64 bytes);
extern void write_int (jaguar_i32 i);
#include "/home/dry/Documents/Eggo/jaguar/build/string.jr.h"
typedef struct Foo {
} Foo;
extern inline jaguar_str Foo_f();
extern inline jaguar_str Foo_f() { 
return "hey";}

jaguar_i32 main () {vec_jaguar_i32 t = newVec_jaguar_i32();({
	vec_jaguar_i32* __gbval1 = &t;
vec_jaguar_i32_push(__gbval1,2);
});write_int(({
	vec_jaguar_i32* __gbval2 = &t;
vec_jaguar_i32_at(__gbval2,0);
}));return 0;
	return 0;
}
