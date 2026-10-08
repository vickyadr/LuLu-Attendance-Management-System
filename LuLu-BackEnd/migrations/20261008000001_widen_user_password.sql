-- Widen password column for Argon2 PHC strings (~97 chars).
-- Legacy SHA-256 hex is 64 chars, Argon2id PHC needs more.
ALTER TABLE IF EXISTS users ALTER COLUMN user_password TYPE TEXT;
