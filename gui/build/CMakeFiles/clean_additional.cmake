# Additional clean files
cmake_minimum_required(VERSION 3.16)

if("${CONFIG}" STREQUAL "" OR "${CONFIG}" STREQUAL "")
  file(REMOVE_RECURSE
  "CMakeFiles/konvert-settings_autogen.dir/AutogenUsed.txt"
  "CMakeFiles/konvert-settings_autogen.dir/ParseCache.txt"
  "konvert-settings_autogen"
  )
endif()
