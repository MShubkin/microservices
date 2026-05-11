DROP TABLE IF EXISTS hedgehogs;
DROP TABLE IF EXISTS hedgehog_versions;
DROP TABLE IF EXISTS special_hedgehogs;

CREATE TABLE hedgehogs(uuid uuid NOT NULL,
    favourite_food TEXT NOT NULL,
    born_on DATE NOT NULL,
    died_on DATE,
    changed_by INTEGER NOT NULL
);

CREATE TABLE special_hedgehogs(
    id BIGINT NOT NULL,
    uuid uuid NOT NULL,
    favourite_food TEXT NOT NULL,
    born_on DATE NOT NULL,
    died_on DATE,
    changed_by INTEGER NOT NULL
);

CREATE TABLE hedgehog_versions(
    pricing_version SMALLINT NOT NULL,
    id BIGINT NOT NULL,
    uuid uuid NOT NULL,
    favourite_food TEXT NOT NULL,
    born_on DATE NOT NULL,
    died_on DATE,
    changed_by INTEGER NOT NULL
);
