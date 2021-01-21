cmake_minimum_required(VERSION 2.8 FATAL_ERROR)
project(crfsuite)
enable_language(C)

include_directories(${PROJECT_SOURCE_DIR}/crfsuite/include)

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

find_package(cqdb REQUIRED)
include_directories(${cqdb_INCLUDE_DIR})

FILE (GLOB
      SOURCES
      ${PROJECT_SOURCE_DIR}/crfsuite/lib/crf/src/*.c)

add_library(crfsuite STATIC ${SOURCES})
set_target_properties(crfsuite PROPERTIES PUBLIC_HEADER "crfsuite/include/crfsuite.h;crfsuite/include/crfsuite.hpp;crfsuite/include/crfsuite_api.hpp")
target_link_libraries(crfsuite ${cqdb_LIBRARIES} ${liblbfgs_LIBRARIES} m)
install(TARGETS crfsuite ARCHIVE DESTINATION lib PUBLIC_HEADER DESTINATION include)
export(PACKAGE crfsuite)

include(CMakePackageConfigHelpers)
configure_package_config_file(
	"${PROJECT_SOURCE_DIR}/Config.cmake.in"
	"${PROJECT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
	INSTALL_DESTINATION "lib/crfsuite/cmake"
	NO_SET_AND_CHECK_MACRO
	NO_CHECK_REQUIRED_COMPONENTS_MACRO
)
install(FILES "${PROJECT_BINARY_DIR}/${PROJECT_NAME}Config.cmake" DESTINATION "lib/crfsuite/cmake" COMPONENT Development)
