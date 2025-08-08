-- Add stamps to users and stamps_earned to orders
ALTER TABLE users ADD COLUMN stamps INTEGER DEFAULT 0;
ALTER TABLE orders ADD COLUMN stamps_earned INTEGER DEFAULT 0;

-- Optional: backfill stamps_earned for existing orders to 0 (default already 0)
-- Optional: you can migrate sweet_cash to balance here if needed; current code stops using sweet_cash
