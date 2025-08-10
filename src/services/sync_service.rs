use crate::error::AppResult;
use crate::external::*;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct SyncService {
    pool: SqlitePool,
    sevencloud_api: std::sync::Arc<tokio::sync::Mutex<SevenCloudAPI>>,
}

impl SyncService {
    pub fn new(
        pool: SqlitePool,
        sevencloud_api: std::sync::Arc<tokio::sync::Mutex<SevenCloudAPI>>,
    ) -> Self {
        Self {
            pool,
            sevencloud_api,
        }
    }

    pub async fn sync_orders(&self, start_date: &str, end_date: &str) -> AppResult<usize> {
        let api = self.sevencloud_api.lock().await;
        let orders = api.get_orders(start_date, end_date).await?;

        let mut processed_count = 0;

        for order_record in orders {
            if let Err(e) = self.process_order(order_record).await {
                log::error!("Failed to process order: {:?}", e);
                continue;
            }
            processed_count += 1;
        }

        log::info!("Synchronization complete, processed orders: {}", processed_count);
        Ok(processed_count)
    }

    async fn process_order(&self, order_record: OrderRecord) -> AppResult<()> {
        // 检查订单是否已存在
        let existing = sqlx::query!("SELECT id FROM orders WHERE id = ?", order_record.id)
            .fetch_optional(&self.pool)
            .await?;

        if existing.is_some() {
            log::debug!("Order already exists, skipping: {}", order_record.id);
            return Ok(());
        }

        // 根据会员号查找用户
        let user = if let Some(member_code) = &order_record.member_code {
            sqlx::query!("SELECT id FROM users WHERE member_code = ?", member_code)
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
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                "UPDATE users SET stamps = COALESCE(stamps, 0) + 1 WHERE id = ?",
                user.id
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            log::info!(
                "Successfully processed order: {}, User: {:?}, Stamps reward: {}",
                order_record.id,
                user.id,
                1
            );
        } else {
            log::debug!("Order has no associated user, skipping: {}", order_record.id);
        }

        Ok(())
    }

    pub async fn sync_discount_codes(&self) -> AppResult<usize> {
        let api = self.sevencloud_api.lock().await;
        let coupons = api.get_discount_codes(None).await?;

        let mut processed_count = 0;

        for coupon_record in coupons {
            if let Err(e) = self.process_discount_code(coupon_record).await {
                log::error!("Failed to process discount code: {:?}", e);
                continue;
            }
            processed_count += 1;
        }

        log::info!("Synchronization complete, processed discount codes: {}", processed_count);
        Ok(processed_count)
    }

    async fn process_discount_code(&self, _coupon_record: CouponRecord) -> AppResult<()> {
        // 这里可以实现优惠码同步逻辑
        // 比如更新本地优惠码的使用状态等
        Ok(())
    }
}
