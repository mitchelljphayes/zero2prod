-- migrations/rename_password_column
ALTER TABLE users RENAME password TO password_hash;
