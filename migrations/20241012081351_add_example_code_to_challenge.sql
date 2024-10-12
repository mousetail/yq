-- Add migration script here
ALTER TABLE challenges ADD
    example_code TEXT NOT NULL DEFAULT '';
