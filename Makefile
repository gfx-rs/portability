VULKAN_DIR=modules/vulkan-docs/src
CTS_DIR=../VK-GL-CTS
CHERRY_DIR=../cherry
BINDING=target/vulkan.rs
NATIVE_DIR=target/native
NATIVE_TARGET=$(NATIVE_DIR)/test
NATIVE_OBJECTS=$(NATIVE_DIR)/test.o $(NATIVE_DIR)/window.o
LIB_EXTENSION=
TEST_LIST=$(CURDIR)/conformance/deqp.txt
TEST_LIST_SOURCE=$(CTS_DIR)/external/vulkancts/mustpass/1.0.2/vk-default.txt
DEQP_DIR=$(CTS_DIR)/build/external/vulkancts/modules/vulkan/
DEQP=cd $(DEQP_DIR) && RUST_LOG=debug LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH) ./deqp-vk
CLINK_ARGS=
GIT_TAG=$(shell git describe --abbrev=0 --tags)
GIT_TAG_FULL=$(shell git describe --tags)
OS_NAME=

DOTA_DIR=../dota2/bin/osx64
DOTA_EXE=$(DOTA_DIR)/dota2.app/Contents/MacOS/dota2
#possible command lines are : -vulkan_disable_occlusion_queries -vulkan_scene_system_job_cost 2 +vulkan_batch_submits 1 +vulkan_batch_size 500
DOTA_PARAMS:=-vulkan_disable_occlusion_queries
DOTA_DEMO_PHORONIX= "$(CURDIR)/../dota2/demos/dota2-pts-1971360796.dem"
DOTA_BENCHMARK=+timedemoquit +timedemo $(DOTA_DEMO_PHORONIX) +timedemo_start 40000 +timedemo_end 50000 +fps_max 0 -novconsole -high -autoconfig_level 3
DOTA_BENCH_RESULTS=../dota2/dota/Source2Bench.csv

RUST_BACKTRACE:=1
BACKEND:=gl
DEBUGGER=rust-gdb --args
GFX_METAL_RECORDING:=immediate

CC=g++
CFLAGS=-std=c++11 -ggdb -O0 -I$(VULKAN_DIR)
DEPS=
LDFLAGS=

ifeq ($(OS),Windows_NT)
	LDFLAGS=
	BACKEND=dx12
	LIB_EXTENSION=dll
	OS_NAME=windows
else
	UNAME_S:=$(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		LDFLAGS=-lpthread -ldl -lm -lX11 -lxcb
		BACKEND=vulkan
		LIB_EXTENSION=so
		OS_NAME=linux
	endif
	ifeq ($(UNAME_S),Darwin)
		LDFLAGS=-lpthread -ldl -lm
		BACKEND=metal
		DEBUGGER=rust-lldb --
		LIB_EXTENSION=dylib
		CLINK_ARGS=-- -Clink-arg="-current_version 1.0.0" -Clink-arg="-compatibility_version 1.0.0"
		OS_NAME=macos
	endif
endif

FULL_LIBRARY_PATH=$(CURDIR)/target/debug
LIBRARY=target/debug/libportability.$(LIB_EXTENSION)
LIBRARY_FAST=target/release/libportability.$(LIB_EXTENSION)

.PHONY: all dummy rebuild debug release version-debug version-release binding run-native cts clean cherry dota-debug dota-release dota-orig dota-bench-gfx dota-bench-orig dota-bench-gl package memcpy-report

all: $(NATIVE_TARGET)

dummy:

rebuild:
	cargo build --manifest-path libportability/Cargo.toml --features $(BACKEND)

debug:
	cargo build --manifest-path libportability/Cargo.toml --features $(BACKEND),debug

release: $(LIBRARY_FAST)

version-debug:
	cargo rustc --manifest-path libportability/Cargo.toml --features $(BACKEND),portability-gfx/env_logger $(CLINK_ARGS)

version-release:
	cargo rustc --release --manifest-path libportability/Cargo.toml --features $(BACKEND) $(CLINK_ARGS)


dota-debug: version-debug $(DOTA_EXE)
	DYLD_LIBRARY_PATH=$(CURDIR)/target/debug:$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) $(DOTA_PARAMS)

dota-debugger: version-debug $(DOTA_EXE)
	echo "env DYLD_LIBRARY_PATH=$(CURDIR)/target/debug:$(CURDIR)/$(DOTA_DIR)" >.lldbinit
	DYLD_LIBRARY_PATH=$(CURDIR)/target/debug:$(CURDIR)/$(DOTA_DIR) $(DEBUGGER) $(DOTA_EXE) $(DOTA_PARAMS)

dota-release: version-release $(DOTA_EXE)
	DYLD_LIBRARY_PATH=$(CURDIR)/target/release:$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) $(DOTA_PARAMS)
dota-molten: $(DOTA_EXE)
	DYLD_LIBRARY_PATH=$(CURDIR)/../MoltenVK/Package/Release/MoltenVK/macOS:$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) $(DOTA_PARAMS)
dota-orig: $(DOTA_EXE)
	DYLD_LIBRARY_PATH=$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) $(DOTA_PARAMS)
dota-orig-gl: $(DOTA_EXE)
	DYLD_LIBRARY_PATH=$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) -gl

dota-bench-gfx: version-release $(DOTA_EXE)
	DYLD_LIBRARY_PATH=$(CURDIR)/target/release:$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) $(DOTA_BENCHMARK) $(DOTA_PARAMS)

dota-bench-orig: $(DOTA_EXE)
	DYLD_LIBRARY_PATH=$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) $(DOTA_BENCHMARK) $(DOTA_PARAMS)

dota-bench-gl: $(DOTA_EXE)
	DYLD_LIBRARY_PATH=$(CURDIR)/target/release:$(CURDIR)/$(DOTA_DIR) $(DOTA_EXE) $(DOTA_BENCHMARK) -gl

ifeq ($(UNAME_S),Darwin)
target/debug/libMoltenVK.dylib: version-debug
	cd target/debug && ln -sf libportability.dylib libMoltenVK.dylib
target/release/libMoltenVK.dylib: version-release
	cd target/release && ln -sf libportability.dylib libMoltenVK.dylib
molten-links: target/debug/libMoltenVK.dylib target/release/libMoltenVK.dylib
endif

binding: $(BINDING)

$(BINDING): $(VULKAN_DIR)/vulkan/*.h
	bindgen --no-layout-tests --rustfmt-bindings $(VULKAN_DIR)/vulkan/vulkan.h -o $(BINDING)

$(LIBRARY): dummy
	cargo build --manifest-path libportability/Cargo.toml --features $(BACKEND)
	cargo build --manifest-path libportability-icd/Cargo.toml --features $(BACKEND),portability-gfx/env_logger
	mkdir -p target/native

$(LIBRARY_FAST): dummy
	cargo build --release --manifest-path libportability/Cargo.toml --features $(BACKEND)
	cargo build --release --manifest-path libportability-icd/Cargo.toml --features $(BACKEND)

$(NATIVE_DIR)/%.o: native/%.cpp $(DEPS) Makefile
	$(CC) -c -o $@ $< $(CFLAGS)

$(NATIVE_TARGET): $(LIBRARY) $(NATIVE_OBJECTS) Makefile
	$(CC) -o $(NATIVE_TARGET) $(NATIVE_OBJECTS) $(LIBRARY) $(LDFLAGS)

run-native: $(NATIVE_TARGET)
	$(NATIVE_TARGET)

$(TEST_LIST): $(TEST_LIST_SOURCE)
	cat $(TEST_LIST_SOURCE) | grep -v -e ".event" -e "query" >$(TEST_LIST)

ifdef pick
cts:
	cargo build --manifest-path libportability/Cargo.toml --features $(BACKEND),portability-gfx/env_logger
	($(DEQP) -n $(pick))
else
ifdef debug
cts: $(LIBRARY)
	echo "env LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH)" >.lldbinit
	#(cd $(DEQP_DIR) && LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH) $(DEBUGGER) ./deqp-vk -n $(debug))
	LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH) $(DEBUGGER) $(DEQP_DIR)/deqp-vk -n $(debug)
else
cts: $(LIBRARY) $(TEST_LIST)
	($(DEQP) --deqp-caselist-file=$(TEST_LIST))
	python $(CTS_DIR)/scripts/log/log_to_xml.py TestResults.qpa conformance/last.xml
	mv TestResults.qpa conformance/last.qpa
	firefox conformance/last.xml
endif #debug
endif #pick

clean:
	rm -f $(NATIVE_OBJECTS) $(NATIVE_TARGET) $(BINDING)
	cargo clean

package: version-debug version-release
	cargo build --manifest-path libportability-icd/Cargo.toml --features $(BACKEND)
	cargo build --manifest-path libportability-icd/Cargo.toml --features $(BACKEND) --release
	echo "$(GIT_TAG_FULL)" > commit-sha
	zip gfx-portability-$(OS_NAME)-$(GIT_TAG).zip target/*/libportability*.$(LIB_EXTENSION) libportability-icd/portability-$(OS_NAME)-*.json commit-sha

target/debug/libvulkan.$(LIB_EXTENSION):
	cd target/debug && ln -sf libportability.$(LIB_EXTENSION) libvulkan.$(LIB_EXTENSION)

cherry: $(LIBRARY) target/debug/libvulkan.$(LIB_EXTENSION)
	cd $(CHERRY_DIR) && rm -f Cherry.db && RUST_LOG=warn LD_LIBRARY_PATH=$(FULL_LIBRARY_PATH) go run server.go

memcpy-report:
	RUSTFLAGS='-g --emit=llvm-ir' cd libportability && cargo build --release --features $(BACKEND)
	../memcpy-find/memcpy-find target/release/deps/portability.ll | rustfilt >etc/portability-memcpy.txt
