#include <iostream>


extern "C" void dll_entrypoint_clarke()
{
    
    std::cout << "Loading Compute dll" << std::endl;
}