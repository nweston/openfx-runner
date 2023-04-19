#include <assert.h>
#include <stdarg.h>

int param_value_count(void *);
int param_get_value_1(void *handle, void *value);
int param_get_value_2(void *handle, void *value1, void *value2);
int param_get_value_3(void *handle, void *value1, void *value2, void *value3);

int paramGetValue (void *paramHandle, ...) {
  int count = param_value_count(paramHandle);
  assert(count <= 3);
  void *vals[3];

  va_list ap;
  va_start (ap, paramHandle);
  for (int i = 0; i < count; i++) {
    vals[i] = va_arg (ap, void*);
  }

  switch (count) {
  case 1:
    return param_get_value_1(paramHandle, vals[0]);
    break;
  case 2:
    return param_get_value_2(paramHandle, vals[0], vals[1]);
    break;
  case 3:
    return param_get_value_3(paramHandle, vals[0], vals[1], vals[2]);
    break;
  default:
    return 1;                   /* OfxStatus::Failed */
  }
}

int paramGetValueAtTime (void *paramHandle, double time, ...) {
  int count = param_value_count(paramHandle);
  assert(count <= 3);
  void *vals[3];

  va_list ap;
  va_start (ap, time);
  for (int i = 0; i < count; i++) {
    vals[i] = va_arg (ap, void*);
  }

  switch (count) {
  case 1:
    return param_get_value_1(paramHandle, vals[0]);
    break;
  case 2:
    return param_get_value_2(paramHandle, vals[0], vals[1]);
    break;
  case 3:
    return param_get_value_3(paramHandle, vals[0], vals[1], vals[2]);
    break;
  default:
    return 1;                   /* OfxStatus::Failed */
  }
}
