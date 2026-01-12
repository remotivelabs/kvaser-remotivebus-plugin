#pragma once

#include <stdlib.h>

// Include paths work with both local prebuilt and Docker-built libraries
// The build.rs script sets the correct -I path
#include "linlib.h"
#include "canlib.h"
