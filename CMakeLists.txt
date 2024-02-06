cmake_minimum_required(VERSION 3.25)

set(CMAKE_CXX_STANDARD 20)

project(hello_world LANGUAGES CXX)

option(test "cfg tests?" OFF)
option(ENABLE_COVERAGE "Coverage of tests?" OFF)

add_executable(${PROJECT_NAME} src-cpp/main.cpp)

if (test)
    add_subdirectory(test)
else ()
    message("No GTest")
endif ()
