/*
 * NanoSVG implementation wrapper. Keeping vendor implementation in a separate
 * translation unit lets project sources use strict warnings as errors.
 */
#define NANOSVG_IMPLEMENTATION
#include "third_party/nanosvg/nanosvg.h"
#define NANOSVGRAST_IMPLEMENTATION
#include "third_party/nanosvg/nanosvgrast.h"