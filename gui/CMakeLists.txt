cmake_minimum_required(VERSION 3.21)

project(orbiscreen-gui VERSION 0.41.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(Qt6 6.4 REQUIRED COMPONENTS Widgets DBus Network Test)

qt_standard_project_setup()

set(ORBISCREEN_GUI_SOURCES
    src/main.cpp
    src/mainwindow.cpp
    src/daemonclient.cpp
    src/report.cpp
    src/i18n.cpp
    src/theme.cpp
    src/uikit.cpp
    src/trayicon.cpp
    src/pages/connectpage.cpp
    src/pages/displaypage.cpp
    src/pages/doctorpage.cpp
)

qt_add_executable(orbiscreen-gui ${ORBISCREEN_GUI_SOURCES})

target_include_directories(orbiscreen-gui PRIVATE src)
target_compile_definitions(orbiscreen-gui PRIVATE ORBISCREEN_VERSION="${PROJECT_VERSION}")
target_compile_options(orbiscreen-gui PRIVATE -Wall -Wextra -Wpedantic)
target_link_libraries(orbiscreen-gui PRIVATE Qt6::Widgets Qt6::DBus Qt6::Network)


install(TARGETS orbiscreen-gui RUNTIME DESTINATION bin)

enable_testing()

qt_add_executable(orbiscreen-gui-tests
    tests/tst_gui.cpp
    src/report.cpp
    src/i18n.cpp
    src/daemonclient.cpp
)
target_include_directories(orbiscreen-gui-tests PRIVATE src)
target_compile_options(orbiscreen-gui-tests PRIVATE -Wall -Wextra -Wpedantic)
target_link_libraries(orbiscreen-gui-tests PRIVATE Qt6::Test Qt6::Widgets Qt6::DBus Qt6::Network)

add_test(NAME gui-tests COMMAND orbiscreen-gui-tests)
