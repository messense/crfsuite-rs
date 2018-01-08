/* c bindings to the pycrfsuite library */

#ifndef PYCRFSUITE_H_INCLUDED
#define PYCRFSUITE_H_INCLUDED

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

enum CrfErrorCode {
  CRF_ERROR_CODE_NO_ERROR = 0,
  CRF_ERROR_CODE_PANIC = 1,
  CRF_ERROR_CODE_CRF_ERROR = 2,
};
typedef uint32_t CrfErrorCode;

typedef struct Model Model;

typedef struct Tagger Tagger;

/*
 * Represents a string.
 */
typedef struct {
  char *data;
  size_t len;
  bool owned;
} FfiStr;

typedef struct {
  FfiStr *data;
  size_t len;
} Tags;

typedef struct {
  const char *name;
  double value;
} Attribute;

typedef struct {
  Attribute *data;
  size_t len;
} AttributeList;

/*
 * Clears the last error.
 */
void pycrfsuite_err_clear();

CrfErrorCode pycrfsuite_err_get_last_code();

/*
 * Returns the last error message.
 *
 * If there is no error an empty string is returned.  This allocates new memory
 * that needs to be freed with `pycrfsuite_str_free`.
 */
FfiStr pycrfsuite_err_get_last_message();

/*
 * Initializes the library
 */
void pycrfsuite_init();

void pycrfsuite_model_destroy(Model *m);

Model *pycrfsuite_model_open(const char *s);

/*
 * Frees a ffi str.
 *
 * If the string is marked as not owned then this function does not
 * do anything.
 */
void pycrfsuite_str_free(FfiStr *s);

/*
 * Creates a ffi str from a c string.
 *
 * This sets the string to owned.  In case it's not owned you either have
 * to make sure you are not freeing the memory or you need to set the
 * owned flag to false.
 */
FfiStr pycrfsuite_str_from_cstr(const char *s);

Tagger *pycrfsuite_tagger_create(Model *m);

void pycrfsuite_tagger_destroy(Tagger *t);

Tags *pycrfsuite_tagger_tag(Tagger *t, const AttributeList *xseq, size_t xseq_len);

void pycrfsuite_tags_destroy(Tags *tags);

#endif /* PYCRFSUITE_H_INCLUDED */