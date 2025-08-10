-- 创建用户表 (Postgres)
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    member_code TEXT UNIQUE NOT NULL,
    phone TEXT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    birthday DATE NOT NULL,
    member_type TEXT NOT NULL CHECK (member_type IN ('fan', 'sweet_shareholder', 'super_shareholder')),
    balance BIGINT DEFAULT 0,
    stamps BIGINT DEFAULT 0,
    referrer_id BIGINT NULL REFERENCES users(id),
    referral_code TEXT UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创建订单表
CREATE TABLE IF NOT EXISTS orders (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    member_code TEXT,
    price BIGINT NOT NULL,
    product_name TEXT NOT NULL,
    product_no TEXT,
    order_status INTEGER NOT NULL,
    pay_type INTEGER,
    stamps_earned BIGINT DEFAULT 0,
    external_created_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创建优惠码表
CREATE TABLE IF NOT EXISTS discount_codes (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    code TEXT UNIQUE NOT NULL,
    discount_amount BIGINT NOT NULL,
    code_type TEXT NOT NULL CHECK (code_type IN ('welcome', 'referral', 'purchase_reward', 'redeemed')),
    is_used BOOLEAN DEFAULT FALSE,
    used_at TIMESTAMPTZ NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    external_id BIGINT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创建充值记录表
CREATE TABLE IF NOT EXISTS recharge_records (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    stripe_payment_intent_id TEXT UNIQUE NOT NULL,
    amount BIGINT NOT NULL,
    bonus_amount BIGINT NOT NULL,
    total_amount BIGINT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'succeeded', 'failed', 'canceled')),
    stripe_status TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创建甜品现金交易记录表
CREATE TABLE IF NOT EXISTS sweet_cash_transactions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('earn', 'redeem')),
    amount BIGINT NOT NULL,
    balance_after BIGINT NOT NULL,
    related_order_id BIGINT NULL REFERENCES orders(id),
    related_discount_code_id BIGINT NULL REFERENCES discount_codes(id),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_users_member_code ON users(member_code);
CREATE INDEX IF NOT EXISTS idx_users_phone ON users(phone);
CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_member_code ON orders(member_code);
CREATE INDEX IF NOT EXISTS idx_orders_external_created_at ON orders(external_created_at);
CREATE INDEX IF NOT EXISTS idx_discount_codes_user_id ON discount_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_discount_codes_code ON discount_codes(code);
CREATE INDEX IF NOT EXISTS idx_discount_codes_expires_at ON discount_codes(expires_at);
CREATE INDEX IF NOT EXISTS idx_recharge_records_user_id ON recharge_records(user_id);
CREATE INDEX IF NOT EXISTS idx_recharge_records_stripe_payment_intent_id ON recharge_records(stripe_payment_intent_id);
CREATE INDEX IF NOT EXISTS idx_sweet_cash_transactions_user_id ON sweet_cash_transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_sweet_cash_transactions_transaction_type ON sweet_cash_transactions(transaction_type);
