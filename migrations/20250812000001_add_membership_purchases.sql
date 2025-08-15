-- Membership purchases table
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'membership_purchase_status') THEN
        CREATE TYPE membership_purchase_status AS ENUM ('pending', 'succeeded', 'failed', 'canceled');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS membership_purchases (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    stripe_payment_intent_id VARCHAR(255) UNIQUE NOT NULL,
    target_member_type member_type NOT NULL,
    amount BIGINT NOT NULL, -- cents
    status membership_purchase_status NOT NULL DEFAULT 'pending',
    stripe_status VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_membership_purchases_user_id ON membership_purchases(user_id);
CREATE INDEX IF NOT EXISTS idx_membership_purchases_payment_intent ON membership_purchases(stripe_payment_intent_id);
