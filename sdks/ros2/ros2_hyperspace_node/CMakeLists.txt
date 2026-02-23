cmake_minimum_required(VERSION 3.8)
project(ros2_hyperspace_node)

if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

find_package(ament_cmake REQUIRED)
find_package(rclcpp REQUIRED)
find_package(hyperspace_interfaces REQUIRED)

# Ideally link against the installed cpp sdk, mock paths here:
include_directories(include ../../cpp/include)

add_executable(tribunal_router_node src/tribunal_router_node.cpp)
ament_target_dependencies(tribunal_router_node rclcpp hyperspace_interfaces)

install(TARGETS
  tribunal_router_node
  DESTINATION lib/${PROJECT_NAME})

ament_package()
