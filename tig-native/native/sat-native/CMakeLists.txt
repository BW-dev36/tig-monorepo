# >= 3.24 for "CUDA_ARCHITECTURES native"
cmake_minimum_required(VERSION 3.16 FATAL_ERROR)

enable_testing()

# Initialize CMAKE_CUDA_ARCHITECTURES when CMAKE_CUDA_COMPILER_ID is NVIDIA
cmake_policy(SET CMP0104 NEW)

if (ENABLE_CUDA)
find_package(CUDAToolkit)
endif()

# set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)

set(PROJECT_NAME sat_gpu)
# Shared library
add_library(sat_gpu OBJECT
    sprint_sat.cpp
    sprint_sat_v2.cpp
)

target_compile_options(
    ${PROJECT_NAME}
    PRIVATE $<$<COMPILE_LANGUAGE:CXX>:-O3 -march=native -fPIC>
            $<$<COMPILE_LANGUAGE:CUDA>:-O3 --generate-line-info -Xcompiler -fPIC -Xcompiler -Xcompiler -march=native>
)


target_compile_features(
    ${PROJECT_NAME}
    PUBLIC cxx_std_17
)

if (ENABLE_CUDA)
set_target_properties(
    ${PROJECT_NAME}
    PROPERTIES
        CUDA_SEPARABLE_COMPILATION ON
        CUDA_RESOLVE_DEVICE_SYMBOLS ON # Fixes linking for Windows
        CUDA_ARCHITECTURES native
)
endif()

if (ENABLE_CUDA)
    target_link_libraries(
        ${PROJECT_NAME}
        PRIVATE CUDA::cudart
    )
else()
    target_link_libraries(
        ${PROJECT_NAME}
    )
endif()
