.PHONY = build test

SRC_FILES = $(shell find . -name "*.rs")
CFG_FILES = $(shell find . -name "*.toml")

target/release/lstc-calendar.rlib: $(SRC_FILES) $(CFG_FILES)
	cargo build --release

publish:
	echo "Not implemented"

build: target/release/liblstc-calendar.rlib

tarpaulin-report.html: $(SRC_FILES)
	cargo tarpaulin --out HTML

test: tarpaulin-report.html

clean:
	rm -rf target
