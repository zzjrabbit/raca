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
vdso:
	@echo "Building vdso."
	@cargo build -p vdso_dylib --target $(TARGET_ARCH)-unknown-linux-musl -Zbuild-std $(RELEASE_OPTION)

.PHONY: vdso_test
vdso_test:
	@echo "Building vdso."
	@cargo build -p vdso_dylib --target $(TARGET_ARCH)-unknown-linux-musl -Zbuild-std $(RELEASE_OPTION) --features libos

.PHONY: user
user:
	cargo build -p user_boot --target $(TARGET_ARCH)-unknown-linux-musl -Zbuild-std $(RELEASE_OPTION)

.PHONY: run
run: vdso user
	cargo run -p builder $(RELEASE_OPTION) -- $(ROPT)

.PHONY: test
test:
	TARGET_ARCH="x86_64" make test_impl

.PHONY: test_impl
test_impl: vdso_test user
	cargo testo --features libos
	cargo testh --features libos

.PHONY: clean
clean:
	cargo clean
	rm racaOS.img
