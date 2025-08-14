use crate::error::AppResult;
use crate::external::*;
use sqlx::PgPool;

#[derive(Clone)]
pub struct SyncService {
    pool: PgPool,
    sevencloud_api: std::sync::Arc<tokio::sync::Mutex<SevenCloudAPI>>,
}

impl SyncService {
    pub fn new(
        pool: PgPool,
        sevencloud_api: std::sync::Arc<tokio::sync::Mutex<SevenCloudAPI>>,
    ) -> Self {
        Self {
            pool,
            sevencloud_api,
        }
    }

    /// 同步七云订单到本地
    pub async fn sync_orders(&self, start_date: &str, end_date: &str) -> AppResult<usize> {
        let mut api = self.sevencloud_api.lock().await;
        let orders = api.get_orders(start_date, end_date).await?;

        let mut processed_count = 0;

        for order_record in orders {
            if let Err(e) = self.process_order(order_record).await {
                log::error!("Failed to process order: {:?}", e);
                continue;
            }
            processed_count += 1;
        }

        log::info!(
            "Synchronization complete, processed orders: {}",
            processed_count
        );
        Ok(processed_count)
    }

    /// 处理七云订单
    async fn process_order(&self, order_record: OrderRecord) -> AppResult<()> {
        // 检查订单是否已存在
        let existing = sqlx::query!("SELECT id FROM orders WHERE id = $1", order_record.id)
            .fetch_optional(&self.pool)
            .await?;

        if existing.is_some() {
            log::debug!("Order already exists, skipping: {}", order_record.id);
            return Ok(());
        }

        // 根据会员号查找用户
        let user = if let Some(member_code) = &order_record.member_code {
            sqlx::query!(
                "SELECT id, referrer_id, balance FROM users WHERE member_code = $1",
                member_code
            )
            .fetch_optional(&self.pool)
            .await?
        } else {
            None
        };

        if let Some(user) = user {
            // 开始事务
            let mut tx = self.pool.begin().await?;

            // 插入订单记录
            let created_at = chrono::DateTime::from_timestamp_millis(order_record.create_date)
                .unwrap_or_default();
            let price_cents: i64 = (order_record.price.unwrap_or(0.0) * 100.0) as i64;

            sqlx::query!(
                r#"
                INSERT INTO orders (
                    id, user_id, member_code, price, product_name, product_no,
                    order_status, pay_type, stamps_earned, external_created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                order_record.id,
                user.id,
                order_record.member_code,
                price_cents,
                order_record.product_name,
                order_record.product_no,
                order_record.status,
                order_record.pay_type,
                1,
                created_at
            )
            .execute(&mut *tx)
            .await?;

            // 新订单 +1 个 stamp
            sqlx::query!(
                "UPDATE users SET stamps = COALESCE(stamps, 0) + 1 WHERE id = $1",
                user.id
            )
            .execute(&mut *tx)
            .await?;

            // 订单返利：若用户存在推荐人，则用户与推荐人各获得订单金额的 10%
            // 只有存在推荐人时才发放双方各 10% 返利
            if let Some(referrer_id) = user.referrer_id {
                if price_cents > 0 {
                    let rebate = price_cents / 10; // 向下取整
                    if rebate > 0 {
                        // 下单用户返利
                        let user_new_balance_row = sqlx::query!(
                            "UPDATE users SET balance = COALESCE(balance,0) + $1 WHERE id = $2 RETURNING balance",
                            rebate,
                            user.id
                        )
                        .fetch_one(&mut *tx)
                        .await?;
                        let user_new_balance = user_new_balance_row.balance.unwrap_or(0);
                        sqlx::query!(
                            r#"INSERT INTO sweet_cash_transactions (
                                user_id, transaction_type, amount, balance_after, related_order_id, description
                            ) VALUES ($1, 'earn', $2, $3, $4, $5)"#,
                            user.id,
                            rebate,
                            user_new_balance,
                            order_record.id,
                            format!("Order rebate 10% for order {}", order_record.id)
                        )
                        .execute(&mut *tx)
                        .await?;

                        // 推荐人返利
                        let referrer_new_balance_row = sqlx::query!(
                            "UPDATE users SET balance = COALESCE(balance,0) + $1 WHERE id = $2 RETURNING balance",
                            rebate,
                            referrer_id
                        )
                        .fetch_one(&mut *tx)
                        .await?;
                        let referrer_new_balance = referrer_new_balance_row.balance.unwrap_or(0);
                        sqlx::query!(
                            r#"INSERT INTO sweet_cash_transactions (
                                user_id, transaction_type, amount, balance_after, related_order_id, description
                            ) VALUES ($1, 'earn', $2, $3, $4, $5)"#,
                            referrer_id,
                            rebate,
                            referrer_new_balance,
                            order_record.id,
                            format!("Referral order rebate 10% from user {} order {}", user.id, order_record.id)
                        )
                        .execute(&mut *tx)
                        .await?;
                        log::info!(
                            "Order {} rebate distributed: user {} +{} cents & referrer {} +{} cents",
                            order_record.id,
                            user.id,
                            rebate,
                            referrer_id,
                            rebate
                        );
                    }
                }
            }

            tx.commit().await?;

            log::info!(
                "Successfully processed order: {}, User: {:?}, Stamps reward: {}",
                order_record.id,
                user.id,
                1
            );
        } else {
            log::debug!(
                "Order has no associated user, skipping: {}",
                order_record.id
            );
        }

        Ok(())
    }

    /// 同步七云优惠码
    pub async fn sync_discount_codes(&self) -> AppResult<usize> {
        let mut api = self.sevencloud_api.lock().await;
        let coupons = api.get_discount_codes(None).await?;

        let mut processed_count = 0;

        for coupon_record in coupons {
            if let Err(e) = self.process_discount_code(coupon_record).await {
                log::error!("Failed to process discount code: {:?}", e);
                continue;
            }
            processed_count += 1;
        }

        log::info!(
            "Synchronization complete, processed discount codes: {}",
            processed_count
        );
        Ok(processed_count)
    }

    /// 处理七云优惠码
    async fn process_discount_code(&self, coupon_record: CouponRecord) -> AppResult<()> {
        // 同步逻辑：依据外部优惠码 code 字段（不使用 external_id），更新本地 is_used/used_at
        // _coupon_record.is_use: "0" 未使用, "1" 已使用
        let code_str = coupon_record.code.to_string();

        // 查询本地是否存在该优惠码
        let local = sqlx::query!(
            r#"SELECT id, is_used FROM discount_codes WHERE code = $1"#,
            code_str
        )
        .fetch_optional(&self.pool)
        .await?;

        if local.is_none() {
            log::debug!(
                "Discount code not found locally, skipping sync: external_code={}",
                coupon_record.code
            );
            return Ok(());
        }
        let local = local.unwrap();

        let external_used = match coupon_record.is_use.as_str() {
            "0" => false,
            "1" => true,
            other => {
                log::warn!(
                    "Unknown is_use value from external coupon: code={}, value={}",
                    coupon_record.code,
                    other
                );
                false
            }
        };

        // 若外部已使用而本地未标记，则更新
        if external_used && !local.is_used.unwrap_or(false) {
            // 转换 use_date (七云时间戳假定为毫秒)；若不存在则使用当前时间
            let used_at = coupon_record
                .use_date
                .and_then(|ts| chrono::DateTime::from_timestamp_millis(ts))
                .unwrap_or_else(|| chrono::Utc::now());

            sqlx::query!(
                r#"UPDATE discount_codes SET is_used = TRUE, used_at = $1, updated_at = NOW() WHERE id = $2"#,
                used_at,
                local.id
            )
            .execute(&self.pool)
            .await?;

            log::info!(
                "Discount code marked as used via sync: code={}, id={:?}",
                coupon_record.code,
                local.id
            );
        } else if !external_used && local.is_used.unwrap_or(false) {
            // 外部显示未使用但本地已使用——通常不回滚，记录冲突
            log::warn!(
                "Usage state mismatch (local used, external unused), keeping local: code={}, id={:?}",
                coupon_record.code,
                local.id
            );
        } else {
            log::debug!(
                "Discount code already in sync: code={}, used={}",
                coupon_record.code,
                external_used
            );
        }

        Ok(())
    }
}
