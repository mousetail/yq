-- Add migration script here

CREATE TABLE solutions (
    id SERIAL PRIMARY KEY NOT NULL,
    author INTEGER NOT NULL REFERENCES accounts(id),
    score INTEGER NOT NULL,
    language varchar(32) NOT NULL,
    version varchar(32) NOT NULL,
    challenge INTEGER NOT NULL REFERENCES challenges(id),
    code TEXT NOT NULL,
    submitted_date TIMESTAMP WITH TIME ZONE DEFAULT now()
)