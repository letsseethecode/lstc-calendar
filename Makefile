.PHONY = build test

SRC_FILES = $(shell find . -name "*.rs")
CFG_FILES = $(shell find . -name "*.toml")

target/release/lstc-calendar.rlib: $(SRC_FILES) $(CFG_FILES)
	cargo build --release

build: target/release/liblstc-calendar.rlib

publish: build
	$(eval VERSION := $(shell grep -Eoi "^version = \"([^\"]+)\"" Cargo.toml | grep -Eo "(\d+\.\d+\.\d+)"))
	git tag $(VERSION)
	cargo publish

tarpaulin-report.html: $(SRC_FILES)
	cargo tarpaulin --out HTML

test: tarpaulin-report.html

clean:
	rm -rf target
