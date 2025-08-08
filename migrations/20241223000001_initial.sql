-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    member_code TEXT UNIQUE NOT NULL,
    phone TEXT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    birthday DATE NOT NULL,
    member_type TEXT NOT NULL CHECK (member_type IN ('fan', 'sweet_shareholder', 'super_shareholder')),
    balance INTEGER DEFAULT 0,
    -- 将 sweet_cash 更名为 stamps 以与模型一致
    stamps INTEGER DEFAULT 0,
    referrer_id INTEGER NULL,
    referral_code TEXT UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (referrer_id) REFERENCES users(id)
);

-- 创建订单表
CREATE TABLE IF NOT EXISTS orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    member_code TEXT,
    price INTEGER NOT NULL,
    product_name TEXT NOT NULL,
    product_no TEXT,
    order_status INTEGER NOT NULL,
    pay_type INTEGER,
    -- 将 sweet_cash_earned 更名为 stamps_earned 以与模型一致
    stamps_earned INTEGER DEFAULT 0,
    external_created_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- 创建优惠码表
CREATE TABLE IF NOT EXISTS discount_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    code TEXT UNIQUE NOT NULL,
    discount_amount INTEGER NOT NULL,
    code_type TEXT NOT NULL CHECK (code_type IN ('welcome', 'referral', 'purchase_reward', 'redeemed')),
    is_used BOOLEAN DEFAULT FALSE,
    used_at DATETIME NULL,
    expires_at DATETIME NOT NULL,
    external_id INTEGER NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- 创建充值记录表
CREATE TABLE IF NOT EXISTS recharge_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    stripe_payment_intent_id TEXT UNIQUE NOT NULL,
    amount INTEGER NOT NULL,
    bonus_amount INTEGER NOT NULL,
    total_amount INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'succeeded', 'failed', 'canceled')),
    stripe_status TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- 创建甜品现金交易记录表
CREATE TABLE IF NOT EXISTS sweet_cash_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('earn', 'redeem')),
    amount INTEGER NOT NULL,
    balance_after INTEGER NOT NULL,
    related_order_id INTEGER NULL,
    related_discount_code_id INTEGER NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (related_order_id) REFERENCES orders(id),
    FOREIGN KEY (related_discount_code_id) REFERENCES discount_codes(id)
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
