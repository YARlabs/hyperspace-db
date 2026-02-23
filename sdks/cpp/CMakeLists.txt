cmake_minimum_required(VERSION 3.14)
project(hyperspace_cpp_sdk)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find gRPC (which internally finds Protobuf)
find_package(gRPC REQUIRED)

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_BINARY_DIR}/proto
)

# Protobuf and gRPC generation commands go here
# ...

# Math components for Cognitive SDK (Lorentz geometry)
add_library(hyperspace_math src/math.cpp)

# Core SDK Library
add_library(hyperspace_sdk src/client.cpp)
# target_link_libraries(hyperspace_sdk PUBLIC grpc++ protobuf)
