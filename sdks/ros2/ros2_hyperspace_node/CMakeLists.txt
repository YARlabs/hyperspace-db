cmake_minimum_required(VERSION 3.8)
project(ros2_hyperspace_node)

if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

find_package(ament_cmake REQUIRED)
find_package(rclcpp REQUIRED)
find_package(hyperspace_interfaces REQUIRED)
find_package(gRPC REQUIRED)

set(PROTO_SRC_DIR "../../cpp/src")
set(PROTO_INC_DIR "../../cpp/include")

include_directories(include ${PROTO_INC_DIR})

add_executable(tribunal_router_node 
  src/tribunal_router_node.cpp
  ../../cpp/src/client.cpp
)

# Link against gRPC and interfaces
ament_target_dependencies(tribunal_router_node 
  rclcpp 
  hyperspace_interfaces
)

target_link_libraries(tribunal_router_node
  gRPC::grpc++
  protobuf::libprotobuf
)

install(TARGETS
  tribunal_router_node
  DESTINATION lib/${PROJECT_NAME})

ament_package()
