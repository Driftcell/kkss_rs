-- Drop verification_codes table and its indexes if they exist, as we now rely on Twilio Verify
DROP TABLE IF EXISTS verification_codes;
