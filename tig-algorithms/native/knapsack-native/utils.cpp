#include "utils.h"

#ifdef __GPU__

#include <cuda.h>
#endif

int get_nb_gpu()
{
    int nb_gpu = 0;
#ifdef __GPU__
    cudaGetDeviceCount(&nb_gpu);
#endif
    return nb_gpu;
}

