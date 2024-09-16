# >= 3.24 for "CUDA_ARCHITECTURES native"
cmake_minimum_required(VERSION 3.16 FATAL_ERROR)

enable_testing()

# Initialize CMAKE_CUDA_ARCHITECTURES when CMAKE_CUDA_COMPILER_ID is NVIDIA
cmake_policy(SET CMP0104 NEW)

if (ENABLE_CUDA)
find_package(CUDAToolkit)
endif()

# set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)

set(PROJECT_NAME challenges)
# Shared library
add_library(${PROJECT_NAME} OBJECT
    RngArray.cpp
    knapsack.cpp
    vector_search.cpp
)

# target_compile_options(
#     ${PROJECT_NAME}
#     PRIVATE $<$<COMPILE_LANGUAGE:CXX>:-O3 -march=native -fPIC>
#             $<$<COMPILE_LANGUAGE:CUDA>:-O3 --generate-line-info -Xcompiler -fPIC -Xcompiler -Xcompiler -march=native>
# )

target_compile_options(
    ${PROJECT_NAME}
    PRIVATE
        # C++ options
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CONFIG:Debug>>:-O0 -march=native -fPIC>  # Options pour le mode Debug
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CONFIG:Release>>:-O3 -march=native -fPIC>  # Options pour le mode Release

        # CUDA options
        $<$<AND:$<COMPILE_LANGUAGE:CUDA>,$<CONFIG:Debug>>:-O0 --generate-line-info -Xcompiler -fPIC -Xcompiler -march=native>  # Options pour le mode Debug
        $<$<AND:$<COMPILE_LANGUAGE:CUDA>,$<CONFIG:Release>>:-O3 --generate-line-info -Xcompiler -fPIC -Xcompiler -march=native>  # Options pour le mode Release
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
endif()
