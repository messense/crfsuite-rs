cmake_minimum_required(VERSION 2.6)
project(crfsuite C)

include_directories(${PROJECT_SOURCE_DIR}/liblbfgs/include)
include_directories(${PROJECT_SOURCE_DIR}/crfsuite/include)
include_directories(${PROJECT_SOURCE_DIR}/crfsuite/lib/cqdb/include)

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -std=c99")
if(NOT CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -O3")
endif()

FILE (GLOB 
      SOURCES 
      ${PROJECT_SOURCE_DIR}/liblbfgs/lib/libfgs.c 
      ${PROJECT_SOURCE_DIR}/crfsuite/lib/cqdb/src/*.c 
      ${PROJECT_SOURCE_DIR}/crfsuite/lib/crf/src/*.c)

add_library(crfsuite STATIC ${SOURCES})
