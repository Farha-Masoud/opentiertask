cmake_minimum_required(VERSION 3.10)
project(MultithreadedServer)

set(CMAKE_CXX_STANDARD 17)

# Include Boost
find_package(Boost REQUIRED COMPONENTS system thread)

# Include Protobuf
find_package(Protobuf REQUIRED)

# Source files
set(SOURCES
    src/main.cpp
    src/server.cpp
)

# Add executable
add_executable(server ${SOURCES})

# Link libraries
target_include_directories(server PRIVATE ${Protobuf_INCLUDE_DIRS})
target_link_libraries(server PRIVATE ${Protobuf_LIBRARIES} Boost::system Boost::thread)

