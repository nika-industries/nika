DEFINE TABLE store SCHEMALESS;

DEFINE FIELD name ON store TYPE string;
DEFINE FIELD config ON store TYPE object;

DEFINE INDEX uniqueStoreNameIndex ON TABLE store COLUMNS name UNIQUE;
