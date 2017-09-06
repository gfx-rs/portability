HEADER=native/vulkan/vulkan.h
BINDING=src/original.rs
TARGET=native/test
OBJECTS=native/test.o
OUTPUT_DIR=target/debug
OUTPUT=${OUTPUT_DIR}/libvulkan.a

all: ${TARGET}

${BINDING}: ${HEADER}
	bindgen --no-layout-tests --rustfmt-bindings ${HEADER} -o ${BINDING}

portability: ${BINDING}
	cargo build

${TARGET}: portability ${OBJECTS}
	gcc -o ${TARGET} -L${OUTPUT_DIR} -lvulkan ${OBJECTS}

run: ${TARGET}
	${TARGET}

clean:
	rm -f ${OBJECTS} ${TARGET} ${BINDING}
