CC=gcc
CFLAGS=
DEPS=
LDFLAGS=-lpthread -ldl -lm

HEADER=native/vulkan/vulkan.h
BINDING=target/vulkan.rs
NATIVE_DIR=target/native
TARGET=$(NATIVE_DIR)/test
OBJECTS=$(NATIVE_DIR)/test.o
LIBRARY=target/debug/libportability.a

all: $(TARGET)

$(BINDING): $(HEADER)
	bindgen --no-layout-tests --rustfmt-bindings $(HEADER) -o $(BINDING)

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
