CC = gcc
CFLAGS = -Wall -flto=3 -g
OPENCL_INCLUDE = -I/opt/rocm/opencl/include
OPENCL_LIB = -L/opt/rocm/opencl/lib -l:libOpenCL.so.1
OPENCL_VERSION_FLAG = -DCL_TARGET_OPENCL_VERSION=220

SOURCES = main.c
OBJECTS = $(SOURCES:.c=.o)
TARGET = testbench

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CC) $(CFLAGS) $(OPENCL_VERSION_FLAG) $(OPENCL_INCLUDE) -o $@ $^ $(OPENCL_LIB)

%.o: %.c
	$(CC) $(CFLAGS) $(OPENCL_VERSION_FLAG) $(OPENCL_INCLUDE) -c $< -o $@

.PHONY: all clean fclean run

clean:
	rm -f $(OBJECTS)

fclean: clean
	rm -f $(TARGET)

run:
	./$(TARGET)