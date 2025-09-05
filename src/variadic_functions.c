/* C implementations of variadic suite functions. These call back into
   non-variadic rust functions. */
#include <assert.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int param_value_count(void *);
int param_get_value_1(void *handle, void *value);
int param_get_value_2(void *handle, void *value1, void *value2);
int param_get_value_3(void *handle, void *value1, void *value2, void *value3);
int param_get_value_4(void *handle, void *value1, void *value2, void *value3, void *value4);
const char *param_get_type(void *handle);
void param_set_value_boolean(void *handle, int value);
void param_set_value_integer(void *handle, int value);
void param_set_value_choice(void *handle, int value);
void param_set_value_double(void *handle, double value);
void param_set_value_string(void *handle, const char *value);
int message_impl(void *handle, const char *messageType, const char *messageId,
                 const char *message);

int paramGetValue (void *paramHandle, ...) {
  int count = param_value_count(paramHandle);
  assert(count <= 4);
  void *vals[4];

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
  case 4:
    return param_get_value_4(paramHandle, vals[0], vals[1], vals[2], vals[3]);
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
  default:
    return 1;                   /* OfxStatus::Failed */
  }
}

int paramSetValue(void *paramHandle, ...) {
  va_list ap;
  va_start (ap, paramHandle);
  const char *type = param_get_type(paramHandle);
  if (!strcmp(type, "OfxParamTypeBoolean")) {
    param_set_value_boolean(paramHandle, va_arg(ap, int));
  } else if (!strcmp(type, "OfxParamTypeInteger")) {
    param_set_value_integer(paramHandle, va_arg(ap, int));
  } else if (!strcmp(type, "OfxParamTypeDouble")) {
    param_set_value_double(paramHandle, va_arg(ap, double));
  } else if (!strcmp(type, "OfxParamTypeString")) {
    param_set_value_string(paramHandle, va_arg(ap, char*));
  } else if (!strcmp(type, "OfxParamTypeChoice")) {
    param_set_value_choice(paramHandle, va_arg(ap, int));
  } else {
    return 1;                   /* OfxStatus::Failed */
  }

  return 0;
}

int paramSetValueAtTime(void *paramHandle, double time, ...) {
  va_list ap;
  va_start (ap, time);
  const char *type = param_get_type(paramHandle);
  if (!strcmp(type, "OfxParamTypeBoolean")) {
    param_set_value_boolean(paramHandle, va_arg(ap, int));
  } else if (!strcmp(type, "OfxParamTypeInteger")) {
    param_set_value_integer(paramHandle, va_arg(ap, int));
  } else if (!strcmp(type, "OfxParamTypeDouble")) {
    param_set_value_double(paramHandle, va_arg(ap, double));
  } else if (!strcmp(type, "OfxParamTypeString")) {
    param_set_value_string(paramHandle, va_arg(ap, char*));
  } else if (!strcmp(type, "OfxParamTypeChoice")) {
    param_set_value_choice(paramHandle, va_arg(ap, int));
  } else {
    return 1;                   /* OfxStatus::Failed */
  }

  return 0;
}

int message(void *handle, const char *messageType, const char *messageId,
            const char *format, ...) {
  va_list ap;

  /* Determine required size */

  va_start(ap, format);
  int size = vsnprintf(NULL, 0, format, ap);
  va_end(ap);

  char *p = NULL;
  if (size > 0) {
    size++;             /* For '\0' */
    p = malloc(size);

    if (p) {
      va_start(ap, format);
      vsnprintf(p, size, format, ap);
      va_end(ap);
    }
  }

  int stat = message_impl(handle, messageType, messageId, p);

  free(p);

  return stat;
}
