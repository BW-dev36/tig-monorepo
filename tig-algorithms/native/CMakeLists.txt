# >= 3.24 for "CUDA_ARCHITECTURES native"
cmake_minimum_required(VERSION 3.16 FATAL_ERROR)

# Option to enable or disable CUDA
option(ENABLE_CUDA "Enable CUDA support" ON)

if (ENABLE_CUDA)

project(
    cpp_cuda
    VERSION 0.1.0
    LANGUAGES CXX CUDA
)
else()

project(
    cpp_cuda
    VERSION 0.1.0
    LANGUAGES CXX
)

endif()

enable_testing()

# Initialize CMAKE_CUDA_ARCHITECTURES when CMAKE_CUDA_COMPILER_ID is NVIDIA
cmake_policy(SET CMP0104 NEW)

if(BUILD_SHARED_LIBS)
    set(CMAKE_POSITION_INDEPENDENT_CODE ON)
endif()

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -static-libstdc++ -static-libgcc")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -static-libstdc++ -static-libgcc")

if (ENABLE_CUDA)
    find_package(CUDAToolkit)
endif()

add_subdirectory(knapsack-native)
add_subdirectory(sat-native)
add_subdirectory(clarke-native)
add_subdirectory(vector-native)

#set(CMAKE_SHARED_LINKER_FLAGS "-Wl,--export-all-symbols")

# Add Rust library
set(RUST_LIB_DIR ${RUST_LIB_DIR})
add_library(rust_lib STATIC IMPORTED)
set_target_properties(rust_lib PROPERTIES
    IMPORTED_LOCATION ${RUST_LIB_DIR}/librust_lib.a
)



# Shared library
add_library(cpp_cuda STATIC 
                $<TARGET_OBJECTS:knapsack_gpu>
                $<TARGET_OBJECTS:sat_gpu>
                $<TARGET_OBJECTS:clarke_gpu>
                $<TARGET_OBJECTS:vector_search>)

target_compile_options(
    cpp_cuda
    PRIVATE $<$<COMPILE_LANGUAGE:CXX>:-O3 -mavx2 -march=native>
            $<$<COMPILE_LANGUAGE:CUDA>:-O3 --generate-line-info -Xcompiler -mavx2 -Xcompiler -march=native>
)

target_compile_features(
    cpp_cuda
    PUBLIC cxx_std_17
)
# Créez un fichier source spécifique pour forcer l'inclusion de libstdc++
add_library(libstdcxx_object OBJECT entry.cpp)

# Forcer la liaison statique des bibliothèques standard C++ et GCC
set_target_properties(libstdcxx_object PROPERTIES COMPILE_FLAGS "-static-libstdc++ -static-libgcc")


if (ENABLE_CUDA)
    set_target_properties(
        cpp_cuda
        PROPERTIES
            CUDA_SEPARABLE_COMPILATION ON
            CUDA_RESOLVE_DEVICE_SYMBOLS ON # Fixes linking for Windows
            CUDA_ARCHITECTURES native
    )
    target_link_libraries(
        cpp_cuda
        PRIVATE libstdcxx_object
        PRIVATE CUDA::cudart
        PUBLIC knapsack_gpu
        PUBLIC rust_lib
    )

else()
    target_link_libraries(
        cpp_cuda
        PRIVATE libstdcxx_object
        PUBLIC knapsack_gpu
        PUBLIC rust_lib
    )
endif()
#add_executable(main main.cpp)
#target_link_libraries(main PRIVATE dp_cuda PRIVATE CUDA::cudart)

#add_executable(simple_test simple_test.cpp)
#target_link_libraries(simple_test PRIVATE dp_cuda  PRIVATE CUDA::cudart)
#add_test(
#    NAME simple_test
#    COMMAND simple_test
#)