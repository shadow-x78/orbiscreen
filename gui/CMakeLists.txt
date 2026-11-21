cmake_minimum_required(VERSION 3.21)

project(orbiscreen-gui VERSION 0.44.6 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_SKIP_RPATH ON)
set(CMAKE_SKIP_INSTALL_RPATH ON)

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
    src/qrview.cpp
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

# Theme and app icon rendering: vector glyphs/logo at several DPRs plus the committed PNG set.
qt_add_executable(orbiscreen-theme-tests
    tests/tst_theme.cpp
    src/theme.cpp
    src/uikit.cpp
)
target_include_directories(orbiscreen-theme-tests PRIVATE src)
target_compile_options(orbiscreen-theme-tests PRIVATE -Wall -Wextra -Wpedantic)
target_compile_definitions(orbiscreen-theme-tests PRIVATE
    ORBISCREEN_ICON_DIR="${CMAKE_CURRENT_SOURCE_DIR}/icons"
    ORBISCREEN_SOURCE_DIR="${CMAKE_CURRENT_SOURCE_DIR}/.."
)
target_link_libraries(orbiscreen-theme-tests PRIVATE Qt6::Test Qt6::Widgets)
add_test(NAME theme-tests COMMAND orbiscreen-theme-tests)
set_tests_properties(theme-tests PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
