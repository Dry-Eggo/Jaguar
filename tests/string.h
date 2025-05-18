
#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
extern jaguar_i32 strlen (jaguar_str n);
jaguar_deftype(string);
typedef struct string {
	jaguar_str data;
	jaguar_i32 len;
} string;

extern inline string new_string (jaguar_str m) {
	string tmp = {.data = m,.len = strlen(m)};
	return tmp;
}
jaguar_i32 string_size(string *self) {
	return self->len;
}jaguar_str string_to_str(string *self) {
	return self->data;
}char string_at(string *self,jaguar_i32 idx) {
	return jaguar_str_at(self->data, idx);
}