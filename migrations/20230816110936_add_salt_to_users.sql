-- migrations/add_salt_to_users
ALTER TABLE users ADD COLUMN salt TEXT NOT NULL;
