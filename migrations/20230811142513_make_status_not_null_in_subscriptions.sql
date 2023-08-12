-- make_status_not_null_in_subscriptions
BEGIN;
	UPDATE subscriptions
		SET status = 'confirmed'
		WHERE status IS NULL; 
		-- Make `status` mandatory
	ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
