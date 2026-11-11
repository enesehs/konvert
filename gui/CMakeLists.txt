cmake_minimum_required(VERSION 3.16)
project(konvert-settings LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../VERSION")
    file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/../VERSION" APP_VERSION_RAW)
    list(GET APP_VERSION_RAW 0 APP_VERSION)
    string(STRIP "${APP_VERSION}" APP_VERSION)
else()
    set(APP_VERSION "v0.1-beta")
endif()

add_compile_definitions(KONVERT_APP_VERSION="${APP_VERSION}")

find_package(Qt6 REQUIRED COMPONENTS Widgets Core Gui Svg)

add_executable(konvert-settings
    src/main.cpp
    src/mainwindow.cpp
    src/mainwindow.h
    src/configmanager.cpp
    src/configmanager.h
    src/translator.h
    resources.qrc
)

target_link_libraries(konvert-settings PRIVATE Qt6::Widgets Qt6::Core Qt6::Gui Qt6::Svg)
