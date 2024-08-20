#pragma once

#ifdef __GPU__
    #define CUDA_CHECK(call) \
        do { \
            cudaError_t error = call; \
            if (error != cudaSuccess) { \
                std::cout << "CUDA error at " << __FILE__ << ":" << __LINE__ \
                        << ": " << cudaGetErrorString(error) << std::endl; \
                exit(1); \
            } \
        } while(0)


int get_nb_gpu();

#endif
