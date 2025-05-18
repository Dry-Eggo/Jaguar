all : round

round:
	cargo build
	clear
	@./target/debug/jagc ./tests/main.jr -o ./tests/main
	@nasm -f elf64 ./std/trn0/stdio.asm -o ./std/stdjr.o
	@gcc ./std/claw.c  -l./std/stdjr.o -o ./std/claw.o -c -static -g
	@gcc -no-pie -static -o ./tests/main ./build/main.c ./std/stdjr.o ./std/claw.o -g -w
	@./tests/main

.Phony : clean build
clean:
	echo "Cleaning artifacts..."
	rm -rf ./tests/main.tsm ./tests/main.o ./tests/main
	echo "Done"

buildc:
	@clear
	@./target/debug/jagc ./tests/main.jr -o ./tests/main
run:
	@make buildc
	@make compile
	@clear
	@./tests/main
compile:
	@nasm -f elf64 ./std/trn0/stdio.asm -o ./std/stdjr.o
	@gcc ./std/claw.c -l./std/stdjr.o -o ./std/claw.o -c -static
	@gcc -no-pie -static -o ./tests/main ./build/main.c ./std/stdjr.o ./std/claw.o
debug:
	@gdb ./tests/main

