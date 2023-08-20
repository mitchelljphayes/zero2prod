-- migrations/seed_user.sql
INSERT INTO users (user_id, username, password_hash)
VALUES (
	'dba8fc02-7b9d-4e16-ab89-3a501f16205a',
	'admin',
	'$argon2id$v=19$m=15000,t=2,p=2$fHCVZ699nUFOe/l6MSUXsg$gzAOd0KssCubbwzKnvjuoD/qVj3/WN2aNoD3Tw6+S7U'
)

