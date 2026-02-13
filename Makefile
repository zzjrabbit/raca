MAKEFLAGS += --no-print-directory

TARGET_ARCH := loongarch64
MODE := debug
ROPT := -s

USER_FILE_PREFIX := target/$(TARGET_ARCH)-unknown-linux-musl/$(MODE)
VDSO_DYLIB_PATH := $(USER_FILE_PREFIX)/libvdso_dylib.so
USER_BOOT_PATH := $(USER_FILE_PREFIX)/user_boot

export VDSO_DYLIB_PATH
export USER_BOOT_PATH

ifeq ($(MODE), release)
	RELEASE_OPTION := --release
endif

.PHONY: vdso
vdso: ## Build vsdo.
	@cargo build -p vdso_dylib --target $(TARGET_ARCH)-unknown-linux-musl -Zbuild-std $(RELEASE_OPTION)

.PHONY: vdso_test
vdso_test: ## Build vdso for testing.
	@cargo build -p vdso_dylib --target $(TARGET_ARCH)-unknown-linux-musl -Zbuild-std $(RELEASE_OPTION) --features libos

.PHONY: user
user: ## Build user programs.
	@cargo build -p user_boot --target $(TARGET_ARCH)-unknown-linux-musl -Zbuild-std $(RELEASE_OPTION)

.DEFAULT_GOAL := run

.PHONY: run
run: vdso user ## Build image and run it.
	@cargo run -p builder $(RELEASE_OPTION) -- $(ROPT)

.PHONY: test
test: ## Test object and kernel_hal
	@TARGET_ARCH="x86_64" make test_impl

.PHONY: test_impl
test_impl: vdso_test user
	@cargo testo --features libos
	@cargo testh --features libos

.PHONY: clean
clean: ## Clean up the image and target directory.
	@cargo clean
	@rm racaOS.img

.PHONY: help
help: ## Show help message.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n make \033[36m\033[0m\n"} /^[$$()% 0-9a-zA-Z_-]+:.*?##/ { printf " \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
