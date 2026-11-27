cmake_minimum_required(VERSION 3.16)
project(konvert-settings LANGUAGES CXX)

if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    set(CMAKE_BUILD_TYPE Release CACHE STRING "Build type" FORCE)
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../VERSION")
    file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/../VERSION" APP_VERSION_RAW)
    list(GET APP_VERSION_RAW 0 APP_VERSION)
    string(STRIP "${APP_VERSION}" APP_VERSION)
else()
    set(APP_VERSION "v0.5.0")
endif()

find_package(Qt6 REQUIRED COMPONENTS Widgets Core Gui Network)

add_executable(konvert-settings
    src/main.cpp
    src/mainwindow.cpp
    src/mainwindow.h
    src/configmanager.cpp
    src/configmanager.h
    src/translator.h
    resources.qrc
)

target_compile_definitions(konvert-settings PRIVATE
    KONVERT_APP_VERSION="${APP_VERSION}"
    $<$<CONFIG:Debug>:KONVERT_DEBUG>
)

target_compile_options(konvert-settings PRIVATE
    $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall -Wextra -Wpedantic>
)

target_link_libraries(konvert-settings PRIVATE Qt6::Widgets Qt6::Core Qt6::Gui Qt6::Network)

include(CheckIPOSupported)
check_ipo_supported(RESULT konvert_ipo_ok OUTPUT konvert_ipo_err)
if(konvert_ipo_ok)
    set_property(TARGET konvert-settings PROPERTY INTERPROCEDURAL_OPTIMIZATION_RELEASE TRUE)
endif()

include(GNUInstallDirs)
install(TARGETS konvert-settings RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})
