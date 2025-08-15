-- Update discount_codes.code_type to new ENUM values
-- Previous migration converted code_type to Postgres ENUM type "code_type"

BEGIN;

-- 1) Create new enum type
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'code_type_new') THEN
        CREATE TYPE code_type_new AS ENUM (
            'shareholder_reward',
            'super_shareholder_reward',
            'sweets_credits_reward'
        );
    END IF;
END $$;

-- 2) Alter column to the new enum with value mapping
ALTER TABLE discount_codes
    ALTER COLUMN code_type TYPE code_type_new
    USING (
        CASE code_type::text
            WHEN 'welcome' THEN 'shareholder_reward'
            WHEN 'referral' THEN 'sweets_credits_reward'
            WHEN 'purchase_reward' THEN 'shareholder_reward'
            WHEN 'redeemed' THEN 'sweets_credits_reward'
            ELSE 'sweets_credits_reward'
        END
    )::code_type_new;

-- 3) Replace old enum type name
DO $$ BEGIN
    -- Drop any legacy CHECK constraint if present (should have been removed already)
    EXECUTE 'ALTER TABLE discount_codes DROP CONSTRAINT IF EXISTS discount_codes_code_type_check';

    -- Drop old type and rename new
    IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'code_type') THEN
        DROP TYPE code_type;
    END IF;
    ALTER TYPE code_type_new RENAME TO code_type;
END $$;

COMMIT;
