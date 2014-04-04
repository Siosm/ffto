.PHONY: clean all

all: ffto

clean:
	rm ffto

ffto: ffto.rs
	rustc -O -C prefer-dynamic --cfg ndebug ffto.rs
