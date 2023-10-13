.PHONY: build check clean clippy cocomo extra full install install-dev-deps prebuild uninstall \
update

NAME=$(shell basename $(shell realpath .))

COCOMO_PATHS=Cargo.toml Makefile t/README.md src

build: target/release/${NAME}

full: update check build clippy extra

target/release/${NAME}: README.md $(shell fd '\.rs$$') $(shell fd '^Cargo.toml$$')
	cargo build --release

README.md: t/README.md
	cargo build --release
	kapow $< >$@

clippy:
	cargo clippy -- -D clippy::all

update:
	cargo upgrade --incompatible
	cargo update

check:
	cargo outdated --exit-code 1
	cargo audit

clean: extra-clean
	cargo clean
	git clean -dxf

install-dev-deps:
	cargo install cargo-audit cargo-edit cargo-outdated cocomo fd-find kapow tokei toml-cli trunk \
exa

install:
	cargo install --path .

uninstall:
	cargo uninstall $(shell toml get -r Cargo.toml package.name)

cocomo:
	@tokei ${COCOMO_PATHS}
	@echo "\n---\n"
	@cocomo -o sloccount ${COCOMO_PATHS}
	@echo "---\n"
	@cocomo ${COCOMO_PATHS}

###

extra: data.json data-pretty.json extended.json extended-pretty.json

extra-clean:
	rm -f data.json data-pretty.json extended.json extended-pretty.json

data.json: $(shell fd -t f . data src) Cargo.toml Makefile
	cargo run --release -- >$@

data-pretty.json: $(shell fd -t f . data src) Cargo.toml Makefile
	cargo run --release -- -f json-pretty >$@

extended.json: $(shell fd -t f . data src) Cargo.toml Makefile
	cargo run --release -- --extended >$@

extended-pretty.json: $(shell fd -t f . data src) Cargo.toml Makefile
	cargo run --release -- --extended -f json-pretty >$@

