export MODE=$1
export LD_LIBRARY_PATH=$(find $(pwd)/target/$MODE -iname "*cuda.so" -exec dirname {} \; | sort -u)
