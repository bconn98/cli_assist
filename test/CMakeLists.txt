cmake_minimum_required(VERSION 3.25)
project(my_project_ut)

# GoogleTest requires at least C++14
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)
FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/refs/tags/v1.14.0.zip
)
# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)
enable_testing()
include(CTest)

add_executable(
  ${PROJECT_NAME}
  src/MyTestUt.cpp
)

# set(CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/CMakeModules)
if(ENABLE_COVERAGE AND test)
    check_library_exists(gcov __gcov_init "" HAVE_LIB_GCOV)
    if (HAVE_LIB_GCOV)
      add_compile_options("--coverage" "-O0" "-fprofile-arcs" "-ftest-coverage")
      link_libraries("gcov")
      message(STATUS "DONE")
    else()
      message(STATUS "DONER")
    endif()
endif()

target_link_libraries(
  ${PROJECT_NAME}
  GTest::gtest_main
)

include(GoogleTest)
gtest_discover_tests(${PROJECT_NAME})
