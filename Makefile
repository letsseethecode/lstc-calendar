.PHONY = build test

SRC_FILES = $(shell find . -name "*.rs")

target/release/lstc-calendar.rlib: $(SRC_FILES)
	cargo build --release

publish:
	echo "Not implemented"

build: target/release/lstc-calendar.rlib

tarpaulin-report.html: $(SRC_FILES)
	cargo tarpaulin --out HTML

test: tarpaulin-report.html

