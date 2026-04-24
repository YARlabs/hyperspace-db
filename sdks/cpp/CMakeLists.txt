cmake_minimum_required(VERSION 3.14)
project(hyperspace_cpp_sdk)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(gRPC REQUIRED)
find_package(Protobuf REQUIRED)

include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Core SDK Library
add_library(hyperspace_sdk src/client.cpp)
target_link_libraries(hyperspace_sdk PUBLIC gRPC::grpc++ protobuf::libprotobuf)
