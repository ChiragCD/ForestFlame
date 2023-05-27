UNAME := $(shell uname)

ifeq ($(UNAME), Linux)
ARCH := elf64
endif
ifeq ($(UNAME), Darwin)
ARCH := macho64
endif

tests/%.s: tests/%.snek src/main.rs src/compiler.rs src/parser.rs src/types.rs
	cargo run -- $< tests/$*.s

tests/%.run: tests/%.s runtime/start.rs lib/*.o
	nasm -f $(ARCH) tests/$*.s -o tests/intermediate_$*.o
	ld -r -o tests/$*.o tests/intermediate_$*.o lib/*.o
	ar rcs tests/lib$*.a tests/$*.o
	rustc -L lib/ -L tests/ -lour_code:$* runtime/start.rs -o tests/$*.run

%.run: %.s runtime/start.rs
	nasm -f $(ARCH) $*.s -o $*.o
	ar rcs lib$*.a $*.o
	rustc -L lib/ -L ./ -lour_code:$* runtime/start.rs -o $*.run

.PHONY: test
test:
	cargo build
	cargo test

clean:
	rm -f tests/*.a tests/*.s tests/*.run tests/*.o

submit:
	rm -f ./*.zip 2>/dev/null
	rm -f tests/*.run 2>/dev/null
	zip -r submit.zip ./lib ./runtime ./src ./tests ./Makefile ./Cargo.* ./design.pdf

lib/%.a: lib/%.s
	nasm -f $(ARCH) lib/$*.s -o lib/$*.o
	ar rcs lib/$*.a lib/$*.o

glib: lib/lib_memory_manager.a lib/lib_error.a lib/lib_heap.a lib/lib_link.a lib/lib_array.a lib/lib_io.a
