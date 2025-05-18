#include <stdlib.h>
typedef struct Vec2 {
	int x;
	int y;
} Vec2;

typedef struct Mat2 {
	Vec2* x;
	Vec2* y;
} Mat2;

int main() {
	Mat2* v = (Mat2*)malloc(sizeof(Mat2));
	v->x = (Vec2*)malloc(sizeof(Vec2));
	v->y = (Vec2*)malloc(sizeof(Vec2));
	v->x->y = 20;
}

