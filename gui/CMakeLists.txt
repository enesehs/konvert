cmake_minimum_required(VERSION 3.16)
project(konvert-settings LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

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
