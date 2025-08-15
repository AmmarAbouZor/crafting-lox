#include <stdlib.h>

#include "memory.h"

/**
 * Function for all memory operations (Allocate, Free, Extend , Shrink)
 */
void *reallocate(void *pointer, size_t oldSize, size_t newSize) {
  if (newSize == 0) {
    free(pointer);
    return NULL;
  }

  void *result = realloc(pointer, newSize);

  // Exit the whole VM in case we can allocate memory.
  if (result == NULL)
    exit(1);

  return result;
}
