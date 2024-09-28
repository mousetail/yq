-- Add migration script here
CREATE TABLE challenges (
    id SERIAL NOT NULL PRIMARY KEY,
    name varchar(255) NOT NULL,
    description TEXT NOT NULL,
    judge TEXT NOT NULL
);