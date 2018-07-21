VULKAN_DIR=modules/vulkan-docs/src
CTS_DIR=../VK-GL-CTS
CHERRY_DIR=../cherry
BINDING=target/vulkan.rs
NATIVE_DIR=target/native
TARGET=$(NATIVE_DIR)/test
OBJECTS=$(NATIVE_DIR)/test.o $(NATIVE_DIR)/window.o
LIB_EXTENSION=
TEST_LIST=$(CURDIR)/conformance/deqp.txt
TEST_LIST_SOURCE=$(CTS_DIR)/external/vulkancts/mustpass/1.0.2/vk-default.txt
DEQP_DIR=$(CTS_DIR)/build/external/vulkancts/modules/vulkan/
DEQP=cd $(DEQP_DIR) && RUST_LOG=debug LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH) ./deqp-vk
DOTA_DIR=../dota2/bin/osx64
DOTA_EXE=$(DOTA_DIR)/dota2.app/Contents/MacOS/dota2
#DOTA_PARAMS=-vulkan_disable_occlusion_queries -vulkan_scene_system_job_cost 2
DOTA_PARAMS=-vulkan_disable_occlusion_queries

RUST_BACKTRACE:=1
BACKEND:=gl
DEBUGGER=rust-gdb --args

CC=g++
CFLAGS=-std=c++11 -ggdb -O0 -I$(VULKAN_DIR)
DEPS=
LDFLAGS=

SYMLINK_DEBUG=
SYMLINK_RELEASE=

ifeq ($(OS),Windows_NT)
	LDFLAGS=
	BACKEND=dx12
	LIB_EXTENSION=dll
else
	UNAME_S:=$(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		LDFLAGS=-lpthread -ldl -lm -lX11 -lxcb
		BACKEND=vulkan
		LIB_EXTENSION=so
		SYMLINK_DEBUG=target/debug/libvulkan.so.1
		SYMLINK_RELEASE=target/release/libvulkan.so.1
	endif
	ifeq ($(UNAME_S),Darwin)
		LDFLAGS=-lpthread -ldl -lm
		BACKEND=metal
		DEBUGGER=rust-lldb --
		LIB_EXTENSION=dylib
	endif
endif

FULL_LIBRARY_PATH=$(CURDIR)/target/debug
LIBRARY=target/debug/libportability.$(LIB_EXTENSION)
LIBRARY_FAST=target/release/libportability.$(LIB_EXTENSION)

.PHONY: all rebuild debug release version-debug version-release binding run cts clean cherry dota-debug dota-release dota-orig

all: $(TARGET)

rebuild: $(SYMLINK_DEBUG)
	cargo build --manifest-path libportability/Cargo.toml --features $(BACKEND)

debug: $(SYMLINK_DEBUG)
	cargo build --manifest-path libportability/Cargo.toml --features $(BACKEND),debug

release: $(LIBRARY_FAST) $(SYMLINK_RELEASE)

version-debug: $(SYMLINK_DEBUG)
	cargo rustc --manifest-path libportability/Cargo.toml --features $(BACKEND),portability-gfx/env_logger -- -Clink-arg="-current_version 1.0.0" -Clink-arg="-compatibility_version 1.0.0"

version-release: $(SYMLINK_RELEASE)
	cargo rustc --release --manifest-path libportability/Cargo.toml --features $(BACKEND) -- -Clink-arg="-current_version 1.0.0" -Clink-arg="-compatibility_version 1.0.0"

dota-debug: version-debug $(DOTA_EXE)
	echo "env DYLD_LIBRARY_PATH=$(CURDIR)/target/debug:$(CURDIR)/$(DOTA_DIR)" >.lldbinit
	DYLD_LIBRARY_PATH=$(CURDIR)/target/debug:$(CURDIR)/$(DOTA_DIR) $(DEBUGGER) $(DOTA_EXE) $(DOTA_PARAMS)

dota-release: version-release $(DOTA_EXE)
	DYLD_LIBRARY_PATH=$(CURDIR)/target/release:$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) $(DOTA_PARAMS)
dota-molten:
	DYLD_LIBRARY_PATH=$(CURDIR)/../MoltenVK/Package/Release/MoltenVK/macOS:$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE)
dota-orig:
	DYLD_LIBRARY_PATH=$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) $(DOTA_PARAMS)

binding: $(BINDING)

$(BINDING): $(VULKAN_DIR)/vulkan/*.h
	bindgen --no-layout-tests --rustfmt-bindings $(VULKAN_DIR)/vulkan/vulkan.h -o $(BINDING)

$(LIBRARY): libportability*/src/*.rs libportability*/Cargo.toml Cargo.lock
	cargo build --manifest-path libportability/Cargo.toml --features $(BACKEND)
	cargo build --manifest-path libportability-icd/Cargo.toml --features $(BACKEND)
	mkdir -p target/native

$(LIBRARY_FAST):  libportability*/src/*.rs libportability*/Cargo.toml Cargo.lock
	cargo build --release --manifest-path libportability/Cargo.toml --features $(BACKEND)
	cargo build --release --manifest-path libportability-icd/Cargo.toml --features $(BACKEND)

$(NATIVE_DIR)/%.o: native/%.cpp $(DEPS) Makefile
	$(CC) -c -o $@ $< $(CFLAGS)

$(TARGET): $(LIBRARY) $(OBJECTS) Makefile $(SYMLINK_DEBUG)
	$(CC) -o $(TARGET) $(OBJECTS) $(LIBRARY) $(LDFLAGS)

run: $(TARGET)
	$(TARGET)

$(TEST_LIST): $(TEST_LIST_SOURCE)
	cat $(TEST_LIST_SOURCE) | grep -v -e ".event" -e "query" >$(TEST_LIST)

ifdef pick
cts: $(SYMLINK_DEBUG)
	cargo build --manifest-path libportability/Cargo.toml --features $(BACKEND),portability-gfx/env_logger
	($(DEQP) -n $(pick))
else
ifdef debug
cts: $(LIBRARY) $(SYMLINK_DEBUG)
	echo "env LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH)" >.lldbinit
	#(cd $(DEQP_DIR) && LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH) $(DEBUGGER) ./deqp-vk -n $(debug))
	LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH) $(DEBUGGER) $(DEQP_DIR)/deqp-vk -n $(debug)
else
cts: $(LIBRARY) $(TEST_LIST) $(SYMLINK_DEBUG)
	($(DEQP) --deqp-caselist-file=$(TEST_LIST))
	python $(CTS_DIR)/scripts/log/log_to_xml.py TestResults.qpa conformance/last.xml
	mv TestResults.qpa conformance/last.qpa
	firefox conformance/last.xml
endif #debug
endif #pick

clean:
	rm -f $(OBJECTS) $(TARGET) $(BINDING)
	cargo clean

cherry: $(TARGET)
	cd $(CHERRY_DIR) && rm -f Cherry.db && RUST_LOG=warn LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH) go run server.go

$(SYMLINK_DEBUG): $(LIBRARY)
	ln -sf "libportability.so" "$(SYMLINK_DEBUG)"

$(SYMLINK_RELEASE): $(LIBRARY_FAST)
	ln -sf "libportability.so" "$(SYMLINK_RELEASE)"
