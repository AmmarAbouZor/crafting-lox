#ifndef clox_table_h
#define clox_table_h

#include "common.h"
#include "value.h"

typedef struct {
  ObjString *key;
  Value value;
} Entry;

typedef struct {
  int count;
  int capacity;
  Entry *entries;
} Table;

void initTable(Table *table);
void freeTable(Table *table);
bool tableGet(Table *table, ObjString *key, Value *value);
bool tableSet(Table *table, ObjString *key, Value value);
bool tableDelete(Table *table, ObjString *key);
void tableAddAll(Table *from, Table *to);
// We need this separate function instead of tableGet because this function will
// check for string equality by comparing the chars and not their memory address
ObjString *tableFindString(Table *table, const char *chars, int length,
                           uint32_t hash);

void tableRemoveWhite(Table *table);
void markTable(Table *table);

#endif
