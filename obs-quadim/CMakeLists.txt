cmake_minimum_required(VERSION 3.28...3.30)

# 1. 创建 MODULE 库
add_library(quadim MODULE)
add_library(OBS::quadim ALIAS quadim)

# 2. 添加源码
target_sources(quadim
  PRIVATE
    obs-quadim.c
)

# 3. 链接 libobs
target_link_libraries(quadim
  PRIVATE OBS::libobs
)

# 4. Windows 下额外资源（可选）
if(OS_WINDOWS)
  configure_file(cmake/windows/obs-module.rc.in quadim.rc)
  target_sources(quadim PRIVATE quadim.rc)
endif()

# 5. 设置在 IDE 里的分组和去掉前缀“lib”
set_target_properties_obs(quadim PROPERTIES
  FOLDER plugins
  PREFIX ""
)