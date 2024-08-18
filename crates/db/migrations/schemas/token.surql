DEFINE TABLE token SCHEMAFULL;

DEFINE FIELD nickname ON token TYPE string;
DEFINE FIELD secret ON token TYPE string;
DEFINE FIELD perms ON token TYPE array<object>;
DEFINE FIELD owner ON token TYPE record<user>;
DEFINE FIELD org ON token TYPE record<org>;

DEFINE INDEX uniqueTokenSecretIndex ON TABLE token COLUMNS secret UNIQUE;
