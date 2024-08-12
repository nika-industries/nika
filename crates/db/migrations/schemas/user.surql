DEFINE TABLE user SCHEMALESS;

DEFINE FIELD name ON user TYPE string;
DEFINE FIELD org ON user TYPE record<org>;
