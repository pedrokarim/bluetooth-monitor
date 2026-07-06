# Bluetooth Monitor — user-space install
#
# Common targets:
#   make            # release build
#   make run        # build + launch
#   make install    # install to ~/.local (delegates to install.sh)
#   make uninstall  # remove installed files
#   make clean      # cargo clean

APP := bluetooth-monitor
CARGO ?= cargo

.PHONY: all build release run install uninstall clean fmt lint check help

all: release

build:
	$(CARGO) build --release --locked

release: build

run: release
	./target/release/$(APP)

install:
	./install.sh --no-build

install-with-build:
	./install.sh

uninstall:
	./install.sh --uninstall

clean:
	$(CARGO) clean

fmt:
	$(CARGO) fmt --all

lint:
	$(CARGO) clippy --release --all-targets

check:
	$(CARGO) fmt --all --check
	$(CARGO) build --release --locked

help:
	@echo "Targets:"
	@echo "  make               # release build"
	@echo "  make run           # build then launch"
	@echo "  make install       # install (assumes binary already built)"
	@echo "  make install-with-build  # build + install"
	@echo "  make uninstall     # remove all installed files"
	@echo "  make clean         # cargo clean"
	@echo "  make fmt / lint    # formatting / clippy"
	@echo "  make check         # what CI runs"
