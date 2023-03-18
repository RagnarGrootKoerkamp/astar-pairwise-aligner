CC=clang
CPP=clang++
CFLAGS=-L../target/release -lastarpa_c

.PHONY: all bindings astarpa

all: example

bindings:
	cbindgen --lang c --cpp-compat --crate astarpa-c -o astarpa.h

astarpa:
	cargo build --release --offline

example: example.c astarpa bindings
	$(CC) $(CFLAGS) example.c -o example

example_cpp: example.c astarpa bindings
	$(CPP) $(CFLAGS) example.cpp -o example_cpp
