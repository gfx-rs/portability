VULKAN_DIR=modules/vulkan-docs/src
BINDING=target/vulkan.rs
NATIVE_DIR=target/native
TARGET=$(NATIVE_DIR)/test
OBJECTS=$(NATIVE_DIR)/test.o
LIBRARY=target/debug/libportability.a

CC=gcc
CFLAGS=-I$(VULKAN_DIR)
DEPS=
LDFLAGS=-lpthread -ldl -lm

all: $(TARGET)

$(BINDING): $(VULKAN_DIR)/vulkan/*.h
	bindgen --no-layout-tests --rustfmt-bindings $(VULKAN_DIR)/vulkan/vulkan.h -o $(BINDING)

$(LIBRARY): $(BINDING) src/*.rs
	cargo build
	mkdir -p target/native

$(NATIVE_DIR)/%.o: native/%.c $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

$(TARGET): $(LIBRARY) $(OBJECTS)
	$(CC) -o $(TARGET) $(LDFLAGS) $(OBJECTS) $(LIBRARY)

run: $(TARGET)
	$(TARGET)

clean:
	rm -f $(OBJECTS) $(TARGET) $(BINDING)
	cargo clean
