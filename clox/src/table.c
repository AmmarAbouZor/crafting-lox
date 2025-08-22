#include <stdlib.h>
#include <string.h>

#include "memory.h"
#include "object.h"
#include "table.h"
#include "value.h"

#define TABLE_MAX_LOAD 0.75

void initTable(Table *table) {
  table->count = 0;
  table->capacity = 0;
  table->entires = NULL;
}

void freeTable(Table *table) {
  FREE_ARRAY(Entry, table->entires, table->capacity);
  initTable(table);
}

static Entry *findEntry(Entry *entries, int capacity, ObjString *key) {
  uint32_t index = key->hash % capacity;

  for (;;) {
    Entry *entry = &entries[index];
    if (entry->key == key || entry->key == NULL) {
      return entry;
    }

    index = (index + 1) % capacity;
  }
}

bool tableGet(Table *table, ObjString *key, Value *value) {
  if (table->count == 0)
    return false;

  Entry *entry = findEntry(table->entires, table->capacity, key);
  if (entry->key == NULL)
    return false;

  *value = entry->value;
  return true;
}

static void adjustCapacity(Table *table, int capacity) {
  Entry *entries = ALLOCATE(Entry, capacity);
  for (int i = 0; i < capacity; i++) {
    entries[i].key = NULL;
    entries[i].value = NIL_VAL;
  }

  // Rebuild the table with the existing entries.
  for (int i = 0; i < table->capacity; i++) {
    Entry *entry = &table->entires[i];
    if (entry->key == NULL)
      continue;

    Entry *dest = findEntry(entries, capacity, entries->key);
    dest->key = entry->key;
    dest->value = entry->value;
  }

  FREE_ARRAY(Entry, table->entires, table->capacity);

  table->entires = entries;
  table->capacity = capacity;
}

bool tableSet(Table *table, ObjString *key, Value value) {
  if (table->count + 1 > table->capacity * TABLE_MAX_LOAD) {
    int capacity = GROW_CAPACITY(table->capacity);
    adjustCapacity(table, capacity);
  }

  Entry *entry = findEntry(table->entires, table->capacity, key);

  bool isNeyKey = entry->key == NULL;

  if (isNeyKey)
    table->count++;

  entry->key = key;
  entry->value = value;

  return isNeyKey;
}

void tableAddAll(Table *from, Table *to) {
  for (int i = 0; i < from->capacity; i++) {
    Entry *entry = &from->entires[i];
    if (entry->key != NULL) {
      tableSet(to, entry->key, entry->value);
    }
  }
}
