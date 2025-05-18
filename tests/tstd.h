
#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
extern void println (jaguar_str n);
jaguar_deftype(String);
typedef struct String {
	jaguar_str d;
} String;
jaguar_deftype(tester_type);
typedef struct tester_type {
	jaguar_i32 x;
} tester_type;

	jaguar_i32 x = 3;


jaguar_str String_to_str(String *self) {
	return self->d;
}char String_at(String *self,jaguar_i32 n) {
	return jaguar_str_at(self->d, n);
}
extern inline String new_str (jaguar_str n) {
	String tmp = {.d = n};
	return tmp;
}

extern inline void foo() {
	println("nested bundles");
}

extern inline void bundle_tester() {
	foo();
}

extern inline tester_type bundle_tester_type (jaguar_i32 cz) {
	tester_type tmp = {.x = cz};
	return tmp;
}
