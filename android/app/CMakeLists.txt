# This file exists to trick Gradle into including libc++_shared.so
cmake_minimum_required(VERSION 3.10)
project(cppshared_dummy)
set(SRC_DIR ./src/main/cpp)
add_library(dummy SHARED ${SRC_DIR}/dummy.cpp)
