cmake_minimum_required(VERSION 2.8 FATAL_ERROR)
project(crfsuite)
enable_language(C)

include_directories(${PROJECT_SOURCE_DIR}/liblbfgs/include)
include_directories(${PROJECT_SOURCE_DIR}/crfsuite/include)
include_directories(${PROJECT_SOURCE_DIR}/crfsuite/lib/cqdb/include)

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -std=c99")

FILE (GLOB
      CQDB_SOURCES 
      ${PROJECT_SOURCE_DIR}/crfsuite/lib/cqdb/src/*.c)
add_library(cqdb STATIC ${CQDB_SOURCES})

FILE (GLOB
      LBFGS_SOURCES
      ${PROJECT_SOURCE_DIR}/liblbfgs/lib/*.c)
add_library(lbfgs STATIC ${LBFGS_SOURCES})

FILE (GLOB 
      SOURCES 
      ${PROJECT_SOURCE_DIR}/crfsuite/lib/crf/src/*.c)

add_library(crfsuite STATIC ${SOURCES})
target_link_libraries(crfsuite cqdb lbfgs m)
export(PACKAGE crfsuite)
