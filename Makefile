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
LIB := target/$(shell rustc --crate-file-name src/lib.rs --crate-type rlib)
ifeq ($(LIB),target/)
# We may not have rustc or the lib.rs file may be broken.
# But don't break the rules on that account.
LIB := target/libanymap.dummy
endif

anymap: $(LIB)

$(LIB): $(SRC)
	@mkdir -p target/
	$(RUSTC) $(RUSTFLAGS) src/lib.rs --out-dir=target -L target

doc/anymap/index.html: $(SRC)
	$(RUSTDOC) src/lib.rs -L target

target/test: $(SRC)
	$(RUSTC) $(RUSTFLAGS) --test -o target/test src/lib.rs -L target

target/quicktest: $(SRC)
	$(RUSTC) --test -o target/quicktest src/lib.rs -L target

# There are no tests to run this way at present. It’s all doctests.
# test: anymap doctest target/test
# 	target/test --test
test: anymap doctest

bench: anymap target/test
	target/test --bench

doctest: $(SRC) $(LIB)
	$(RUSTDOC) -L target --test src/lib.rs

# Can't wait for everything to target, optimised too? OK, you can save some time here.
quicktest: target/quicktest
	target/quicktest --test

docs: doc/anymap/index.html

clean:
	rm -rf target/ doc/

TAGS: $(SRC)
	ctags -f TAGS --options="$(RUST_CTAGS)" --language=rust -R src/

.PHONY: all docs clean test doctest quicktest anymap
