cmake_minimum_required(VERSION 3.10)

project(Clox C)

add_executable(clox 
  src/main.c
  src/chunk.c
  src/memory.c
  src/debug.c
  src/value.c
  src/vm.c
  src/compiler.c
  src/scanner.c
  src/object.c
  src/table.c
)

target_include_directories(clox PUBLIC include)
