cmake_minimum_required(VERSION 3.10)
set(CMAKE_CXX_STANDARD 20)
project(wiqProject C)

set(EXEC_NAME cwiq)

# link the aforementioned libraries
add_executable(${EXEC_NAME} src/queue.c src/main.c)


target_compile_definitions(${EXEC_NAME} PRIVATE graphic)

# add a custom target so we can do make run directly
add_custom_target(run COMMAND ${EXEC_NAME} DEPENDS ${EXEC_NAME} COMMENT "./${EXEC_NAME}")
