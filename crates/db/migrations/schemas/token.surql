DEFINE TABLE token SCHEMAFULL;

DEFINE FIELD nickname ON token TYPE string;
DEFINE FIELD secret ON token TYPE string;
DEFINE FIELD perms ON token TYPE array<array<record<store> | string, 2>>;
DEFINE FIELD owner ON token TYPE record<user>;
DEFINE FIELD org ON token TYPE record<org>;
