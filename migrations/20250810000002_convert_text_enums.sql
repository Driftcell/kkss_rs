-- Convert TEXT + CHECK constrained columns to proper Postgres ENUM types
-- Simplified without dynamic loops (uses default constraint naming pattern <table>_<column>_check)

-- member_type for users table
-- users.member_type
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'member_type') THEN
        CREATE TYPE member_type AS ENUM ('fan','sweet_shareholder','super_shareholder');
    END IF;
    -- Drop default check constraint if present
    EXECUTE 'ALTER TABLE users DROP CONSTRAINT IF EXISTS users_member_type_check';
    -- Convert column
    ALTER TABLE users ALTER COLUMN member_type TYPE member_type USING member_type::member_type;
END $$;

-- code_type for discount_codes table
-- discount_codes.code_type
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'code_type') THEN
        CREATE TYPE code_type AS ENUM ('welcome','referral','purchase_reward','redeemed');
    END IF;
    EXECUTE 'ALTER TABLE discount_codes DROP CONSTRAINT IF EXISTS discount_codes_code_type_check';
    ALTER TABLE discount_codes ALTER COLUMN code_type TYPE code_type USING code_type::code_type;
END $$;

-- recharge_status for recharge_records table
-- recharge_records.status
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'recharge_status') THEN
        CREATE TYPE recharge_status AS ENUM ('pending','succeeded','failed','canceled');
    END IF;
    EXECUTE 'ALTER TABLE recharge_records DROP CONSTRAINT IF EXISTS recharge_records_status_check';
    ALTER TABLE recharge_records ALTER COLUMN status TYPE recharge_status USING status::recharge_status;
END $$;
