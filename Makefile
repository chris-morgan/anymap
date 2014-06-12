RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTFLAGS ?= -O
RUST_REPOSITORY ?= ../rust
RUST_CTAGS ?= $(RUST_REPOSITORY)/src/etc/ctags.rust

all: anymap docs test

# Recursive wildcard function
# http://blog.jgc.org/2011/07/gnu-make-recursive-wildcard-function.html
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
  $(filter $(subst *,%,$2),$d))

SRC := $(call rwildcard,src/,*.rs)
LIB := build/$(shell rustc --crate-file-name src/lib.rs --crate-type rlib)
ifeq ($(LIB),build/)
# We may not have rustc or the lib.rs file may be broken.
# But don't break the rules on that account.
LIB := build/libanymap.dummy
endif

anymap: $(LIB)

$(LIB): $(SRC)
	@mkdir -p build/
	$(RUSTC) $(RUSTFLAGS) src/lib.rs --out-dir=build -L build

doc/anymap/index.html: $(SRC)
	$(RUSTDOC) src/lib.rs -L build

build/test: $(SRC)
	$(RUSTC) $(RUSTFLAGS) --test -o build/test src/lib.rs -L build

build/quicktest: $(SRC)
	$(RUSTC) --test -o build/quicktest src/lib.rs -L build

# There are no tests to run this way at present. It’s all doctests.
# test: anymap doctest build/test
# 	build/test --test
test: anymap doctest

doctest: $(SRC) $(LIB)
	$(RUSTDOC) -L build --test src/lib.rs

# Can't wait for everything to build, optimised too? OK, you can save some time here.
quicktest: build/quicktest
	build/quicktest --test

docs: doc/anymap/index.html

clean:
	rm -rf build/ doc/

TAGS: $(SRC)
	ctags -f TAGS --options="$(RUST_CTAGS)" --language=rust -R src/

.PHONY: all docs clean test doctest quicktest anymap
