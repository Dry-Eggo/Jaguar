#pragma once

#include "/home/dry/Documents/Eggo/jaguar/std/claw.h"
#include "/home/dry/Documents/Eggo/jaguar/build/__generics__.h"
extern jaguar_i32 strlen (jaguar_str n);
extern void strcat (void* d,jaguar_str s);
extern void* malloc (jaguar_u64 s);
extern void* realloc (void* p,jaguar_u64 n);
extern jaguar_str rzapp (jaguar_str d,jaguar_str m);
extern jaguar_str strslice (jaguar_str s,jaguar_i32 n,jaguar_i32 pos);
extern jaguar_str strmrg (jaguar_str m,jaguar_str n);
extern jaguar_str jformat (jaguar_str fmt, ...);
extern void jprintln (jaguar_str fmt, ...);
extern jaguar_str strdup (jaguar_str m);
extern void* mem_get (jaguar_i64 bytes);
extern void memset (jaguar_str d,jaguar_i32 v,jaguar_i32 size);
typedef struct string {

	jaguar_str data;
	jaguar_i32 len;} string;
extern inline jaguar_i32 string_size(string* self);
extern inline jaguar_str string_to_str(string* self);
extern inline char string_at(string* self,jaguar_i32 idx);
extern inline jaguar_i32 string_append(string* self,jaguar_str m);
extern inline jaguar_str string_slice(string* self,jaguar_i32 n,jaguar_i32 p);
extern inline jaguar_str string_substr(string* self,jaguar_i32 n,jaguar_i32 p);
extern inline jaguar_str string_rev(string* self);
extern inline jaguar_i32 string_find(string* self,char c);
extern inline jaguar_i32 string_eq(string* self,jaguar_str sub);
extern inline jaguar_i32 string_is_empty(string* self);
extern inline jaguar_str string_to_upper(string* self);
extern inline jaguar_str string_to_lower(string* self);
extern inline void string_clear(string* self);
extern inline jaguar_i32 string_size(string *self) { 
return self->len;}
extern inline jaguar_str string_to_str(string *self) { 
return strdup(self->data);}
extern inline char string_at(string *self,jaguar_i32 idx) { 
return self->data[idx];}
extern inline jaguar_i32 string_append(string *self,jaguar_str m) { 
self->data = strdup(self->data);\
jaguar_i32 oldlen = strlen(self->data);\
jaguar_i32 addlen = strlen(m);\
self->data = (jaguar_str)(realloc((void*)(self->data),( ( oldlen+ addlen)+ 1)));\
strcat((void*)(self->data),m);\
return 0;}
extern inline jaguar_str string_slice(string *self,jaguar_i32 n,jaguar_i32 p) { 
return strslice(self->data,n,p);}
extern inline jaguar_str string_substr(string *self,jaguar_i32 n,jaguar_i32 p) { 
return ({
	string* __gbval0 = self;
string_slice(__gbval0,n,( n+ p));
});}
extern inline jaguar_str string_rev(string *self) { 
string buffer = {.data = "",.len = 0};\

for (jaguar_int i = ( ({
	string* __gbval1 = self;
string_size(__gbval1);
})- 1)
;( i>= 0);(i = ( i- 1)
)) {
({
	string* __gbval2 = &buffer;
string_append(__gbval2,jformat("{c}",({
	string* __gbval3 = self;
string_at(__gbval3,i);
})));
});}
;\
return strdup(({
	string* __gbval4 = &buffer;
string_to_str(__gbval4);
}));}
extern inline jaguar_i32 string_find(string *self,char c) { 
jaguar_i32 i = ({
	string* __gbval5 = self;
string_size(__gbval5);
});\
jaguar_i32 ret_val = ( i+ i);\

for (jaguar_int n = 0
;( n< i);(n = ( n+ 1)
)) {
char t = ({
	string* __gbval6 = self;
string_at(__gbval6,n);
});
if (( t== c)){
ret_val = n
;break;;
};}
;\
return ret_val;}
extern inline jaguar_i32 string_eq(string *self,jaguar_str sub) { 
if (( self->len!= strlen(sub))){
return 0;
};\

for (jaguar_int i = 0
;( i< self->len);(i = ( i+ 1)
)) {
if (( ({
	string* __gbval7 = self;
string_at(__gbval7,i);
})!= sub[i])){
return 0;
};}
;\
return 1;}
extern inline jaguar_i32 string_is_empty(string *self) { 
return ( self->len== 0);}
extern inline jaguar_str string_to_upper(string *self) { 
string buffer = {.data = "",.len = 0};\

for (jaguar_int i = 0
;( i< self->len);(i = ( i+ 1)
)) {
char t = ({
	string* __gbval8 = self;
string_at(__gbval8,i);
});
({
	string* __gbval9 = &buffer;
string_append(__gbval9,jformat("{c}",({
	char* __gbval10 = &t;
char_to_upper(__gbval10);
})));
});}
;\
return ({
	string* __gbval11 = &buffer;
string_to_str(__gbval11);
});}
extern inline jaguar_str string_to_lower(string *self) { 
string buffer = {.data = "",.len = 0};\

for (jaguar_int i = 0
;( i< self->len);(i = ( i+ 1)
)) {
char t = ({
	string* __gbval12 = self;
string_at(__gbval12,i);
});
({
	string* __gbval13 = &buffer;
string_append(__gbval13,jformat("{c}",({
	char* __gbval14 = &t;
char_to_lower(__gbval14);
})));
});}
;\
return ({
	string* __gbval15 = &buffer;
string_to_str(__gbval15);
});}
extern inline void string_clear(string *self) { 
self->data = "";}

extern inline string new_string (jaguar_str m) {string tmp = {.data = m,.len = strlen(m)};return tmp;
}
