#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"


#define GENERIC_FN_newopt(T) opt_##T newopt (T i) {\
	return {.value = i};\
}
