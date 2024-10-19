-- Add migration script here
ALTER TABLE accounts
    ADD COLUMN created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now();

ALTER TABLE challenges
    ADD COLUMN created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now();

ALTER TABLE solutions
    ADD COLUMN created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now();

CREATE OR REPLACE FUNCTION update_modified_column()
    RETURNS TRIGGER AS $$
    BEGIN
        NEW.updated_at = now();
        RETURN NEW;
    END;
$$ language 'plpgsql';

CREATE TRIGGER update_accounts_changetimestamp BEFORE UPDATE
    ON accounts FOR EACH ROW EXECUTE PROCEDURE 
    update_modified_column();

CREATE TRIGGER update_challenges_changetimestamp BEFORE UPDATE
    ON challenges FOR EACH ROW EXECUTE PROCEDURE 
    update_modified_column();

CREATE TRIGGER update_solutions_changetimestamp BEFORE UPDATE
    ON solutions FOR EACH ROW EXECUTE PROCEDURE 
    update_modified_column();
