# Copyright (c) 2016-present, Facebook, Inc.
# All rights reserved.
#
# This source code is licensed under the BSD-style license found in the
# LICENSE file in the root directory of this source tree. An additional grant
# of patent rights can be found in the PATENTS file in the same directory.
#
# Find libgmock
#
#  LIBGMOCK_DEFINES     - List of defines when using libgmock.
#  LIBGMOCK_INCLUDE_DIR - where to find gmock/gmock.h, etc.
#  LIBGMOCK_LIBRARIES   - List of libraries when using libgmock.
#  LIBGMOCK_FOUND       - True if libgmock found.
find_package(PkgConfig)

IF (LIBGMOCK_INCLUDE_DIR)
  # Already in cache, be silent
  SET(GMock_FIND_QUIETLY TRUE)
ENDIF ()

set(CMAKE_IMPORT_FILE_VERSION 1)

pkg_check_modules(GMOCK gmock QUIET)
if(GMOCK_FOUND)
  add_library(gmock UNKNOWN IMPORTED)
  set_target_properties(
    gmock PROPERTIES
    IMPORTED_CONFIGURATIONS NOCONFIG
    IMPORTED_LINK_INTERFACE_LANGUAGES_NOCONFIG "CXX"
    IMPORTED_LINK_INTERFACE_LIBRARIES_NOCONFIG "${GMOCK_LDFLAGS}"
    INTERFACE_COMPILE_OPTIONS "${GMOCK_CFLAGS_OTHER}"
    INTERFACE_INCLUDE_DIRECTORIES "${GMOCK_INCLUDE_DIRS}"
  )
  set(GMOCK_LIBRARY gmock)
endif()

pkg_check_modules(GMOCK_MAIN gmock_main QUIET)
if(GMOCK_MAIN_FOUND)
  add_library(gmock_main UNKNOWN IMPORTED)
  set_target_properties(
    gmock_main PROPERTIES
    IMPORTED_CONFIGURATIONS NOCONFIG
    IMPORTED_LINK_INTERFACE_LANGUAGES_NOCONFIG "CXX"
    IMPORTED_LINK_INTERFACE_LIBRARIES_NOCONFIG "${GMOCK_MAIN_LDFLAGS}"
    INTERFACE_COMPILE_OPTIONS "${GMOCK_MAIN_CFLAGS_OTHER}"
    INTERFACE_INCLUDE_DIRECTORIES "${GMOCK_MAIN_INCLUDE_DIRS}"
  )
  set(GMOCK_MAIN_LIBRARY gmock_main)
endif()

pkg_check_modules(GTEST gtest QUIET)
if(GTEST_FOUND)
  add_library(gtest UNKNOWN IMPORTED)
  set_target_properties(
    gtest PROPERTIES
    IMPORTED_CONFIGURATIONS NOCONFIG
    IMPORTED_LINK_INTERFACE_LANGUAGES_NOCONFIG "CXX"
    IMPORTED_LINK_INTERFACE_LIBRARIES_NOCONFIG "${GTEST_LDFLAGS}"
    INTERFACE_COMPILE_OPTIONS "${GTEST_CFLAGS_OTHER}"
    INTERFACE_INCLUDE_DIRECTORIES "${GTEST_INCLUDE_DIRS}"
  )
set(GTEST_LIBRARY gtest)
endif()

pkg_check_modules(GTEST_MAIN gtest_main QUIET)
if(GTEST_MAIN_FOUND)
  add_library(gtest_main UNKNOWN IMPORTED)
  set_target_properties(
    gtest_main PROPERTIES
    IMPORTED_CONFIGURATIONS NOCONFIG
    IMPORTED_LINK_INTERFACE_LANGUAGES_NOCONFIG "CXX"
    IMPORTED_LINK_INTERFACE_LIBRARIES_NOCONFIG "${GTEST_MAIN_LDFLAGS}"
    INTERFACE_COMPILE_OPTIONS "${GTEST_MAIN_CFLAGS_OTHER}"
    INTERFACE_INCLUDE_DIRECTORIES "${GTEST_MAIN_INCLUDE_DIRS}"
  )
  set(GTEST_MAIN_LIBRARY gtest_main)
endif()

set(CMAKE_IMPORT_FILE_VERSION)

FIND_PACKAGE_HANDLE_STANDARD_ARGS(
  LIBGMOCK
  DEFAULT_MSG
  GMOCK_INCLUDEDIR
  GMOCK_LIBRARY
  GMOCK_MAIN_LIBRARY
  GTEST_LIBRARY
  GTEST_MAIN_LIBRARY
)
return()

FIND_PATH(LIBGMOCK_INCLUDE_DIR gmock/gmock.h)

FIND_LIBRARY(LIBGMOCK_MAIN_LIBRARY gmock_main)
FIND_LIBRARY(LIBGMOCK_LIBRARY gmock)             
FIND_LIBRARY(LIBGTEST_LIBRARY gtest)
set(LIBGMOCK_LIBRARIES
  ${LIBGMOCK_MAIN_LIBRARY}
  ${LIBGMOCK_LIBRARY}
  ${LIBGTEST_LIBRARY}
)

if(CMAKE_SYSTEM_NAME STREQUAL "Windows")
  # The GTEST_LINKED_AS_SHARED_LIBRARY macro must be set properly on Windows.
  #
  # There isn't currently an easy way to determine if a library was compiled as
  # a shared library on Windows, so just assume we've been built against a
  # shared build of gmock for now.
  SET(LIBGMOCK_DEFINES "GTEST_LINKED_AS_SHARED_LIBRARY=1" CACHE STRING "")
endif()

# handle the QUIETLY and REQUIRED arguments and set LIBGMOCK_FOUND to TRUE if
# all listed variables are TRUE
INCLUDE(FindPackageHandleStandardArgs)
FIND_PACKAGE_HANDLE_STANDARD_ARGS(
  LIBGMOCK
  DEFAULT_MSG
  LIBGMOCK_MAIN_LIBRARY
  LIBGMOCK_LIBRARY
  LIBGTEST_LIBRARY
  LIBGMOCK_LIBRARIES
  LIBGMOCK_INCLUDE_DIR
)

MARK_AS_ADVANCED(
  LIBGMOCK_DEFINES
  LIBGMOCK_MAIN_LIBRARY
  LIBGMOCK_LIBRARY
  LIBGTEST_LIBRARY
  LIBGMOCK_LIBRARIES
  LIBGMOCK_INCLUDE_DIR
)
