cmake_minimum_required(VERSION 2.8 FATAL_ERROR)
project(crfsuite)
enable_language(C)

include_directories(${PROJECT_SOURCE_DIR}/crfsuite/include)
include_directories(${PROJECT_SOURCE_DIR}/crfsuite/lib/cqdb/include)

if(CMAKE_COMPILER_IS_GNUCC)
    # Note: gcc does not implicitly set _POSIX_C_SOURCE or _XOPEN_SOURCE
    # when using -std=c99.
    # http://pubs.opengroup.org/onlinepubs/9699919799/functions/V2_chap02.html#tag_15_02_01_01
    # We specify that we are POSIX.1-2001 compliant and XSI-conforming. We only
    # need to specify _XOPEN_SOURCE as _POSIX_C_SOURCE will be set implicitly.
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -std=c99 -D_XOPEN_SOURCE=600")
else()
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -std=c99")
endif()

find_package(liblbfgs REQUIRED)
include_directories(${liblbfgs_INCLUDE_DIR})

set(CQDB_SOURCES
    ${PROJECT_SOURCE_DIR}/crfsuite/lib/cqdb/src/lookup3.c
    ${PROJECT_SOURCE_DIR}/crfsuite/lib/cqdb/src/cqdb.c)
add_library(cqdb STATIC ${CQDB_SOURCES})
install(TARGETS cqdb ARCHIVE DESTINATION lib)

FILE (GLOB
      SOURCES
      ${PROJECT_SOURCE_DIR}/crfsuite/lib/crf/src/*.c)

add_library(crfsuite STATIC ${SOURCES})
target_link_libraries(crfsuite cqdb ${liblbfgs_LIBRARIES} m)
install(TARGETS crfsuite ARCHIVE DESTINATION lib)
export(PACKAGE crfsuite)
