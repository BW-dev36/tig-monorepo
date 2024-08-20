# >= 3.24 for "CUDA_ARCHITECTURES native"
cmake_minimum_required(VERSION 3.16 FATAL_ERROR)

enable_testing()

# Initialize CMAKE_CUDA_ARCHITECTURES when CMAKE_CUDA_COMPILER_ID is NVIDIA
cmake_policy(SET CMP0104 NEW)



if (ENABLE_CUDA)
    find_package(CUDAToolkit REQUIRED)

    # Shared library with CUDA
    add_library(knapsack_gpu STATIC
        dp_cuda.cu
        knapmaxxing.cpp
        knapmaxxing-v2.cu
        dp_cuda.h
        entrypoint.cu
    )
    target_compile_definitions(knapsack_gpu PUBLIC __GPU__)

    target_compile_options(
        knapsack_gpu
        PRIVATE $<$<COMPILE_LANGUAGE:CXX>:-O3 -mavx2 -march=native -fPIC>
                $<$<COMPILE_LANGUAGE:CUDA>:-O3 --generate-line-info -Xcompiler -fPIC -Xcompiler -mavx2 -Xcompiler -march=native>
    )

    target_compile_features(
        knapsack_gpu
        PUBLIC cxx_std_17
    )

    set_target_properties(
        knapsack_gpu
        PROPERTIES
            CUDA_SEPARABLE_COMPILATION ON
            CUDA_RESOLVE_DEVICE_SYMBOLS ON # Fixes linking for Windows
            CUDA_ARCHITECTURES native
    )

    target_link_libraries(
        knapsack_gpu
        PRIVATE CUDA::cudart
    )

   
else()
    # Shared library without CUDA
    add_library(knapsack_gpu STATIC
        knapmaxxing.cpp
        dp_cuda.h
    )

    target_compile_options(
        knapsack_gpu
        PRIVATE $<$<COMPILE_LANGUAGE:CXX>:-O3 -mavx2 -march=native -fPIC>
    )

    target_compile_features(
        knapsack_gpu
        PUBLIC cxx_std_17
    )
endif()
