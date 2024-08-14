DEFINE TABLE store SCHEMAFULL;

DEFINE FIELD nickname ON store TYPE string;
DEFINE FIELD config ON store TYPE object;
DEFINE FIELD public ON store TYPE bool;
DEFINE FIELD org ON store TYPE record<org>;

DEFINE INDEX uniqueStoreNameIndex ON TABLE store COLUMNS nickname UNIQUE;
