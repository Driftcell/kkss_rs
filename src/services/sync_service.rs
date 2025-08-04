use crate::error::AppResult;
use crate::external::*;
use crate::models::*;
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
                log::error!("处理订单失败: {:?}", e);
                continue;
            }
            processed_count += 1;
        }

        log::info!("同步完成，处理订单数: {}", processed_count);
        Ok(processed_count)
    }

    async fn process_order(&self, order_record: OrderRecord) -> AppResult<()> {
        // 检查订单是否已存在
        let existing = sqlx::query!("SELECT id FROM orders WHERE id = ?", order_record.id)
            .fetch_optional(&self.pool)
            .await?;

        if existing.is_some() {
            log::debug!("订单已存在，跳过: {}", order_record.id);
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
            let price_cents = (order_record.price.unwrap_or(0.0) * 100.0) as i64;
            let sweet_cash_earned = price_cents / 2; // 每消费1美元获得0.5美元甜品现金

            // 开始事务
            let mut tx = self.pool.begin().await?;

            // 插入订单记录
            let created_at = chrono::DateTime::from_timestamp_millis(order_record.create_date)
                .unwrap_or_default();

            sqlx::query!(
                r#"
                INSERT INTO orders (
                    id, user_id, member_code, price, product_name, product_no,
                    order_status, pay_type, sweet_cash_earned, external_created_at
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
                sweet_cash_earned,
                created_at
            )
            .execute(&mut *tx)
            .await?;

            // 更新用户甜品现金
            sqlx::query!(
                "UPDATE users SET sweet_cash = sweet_cash + ? WHERE id = ?",
                sweet_cash_earned,
                user.id
            )
            .execute(&mut *tx)
            .await?;

            // 记录甜品现金交易
            let balance_after = sqlx::query!("SELECT sweet_cash FROM users WHERE id = ?", user.id)
                .fetch_one(&mut *tx)
                .await?
                .sweet_cash;

            let transaction_type_str = TransactionType::Earn.to_string();
            let description = format!("订单 {} 奖励", order_record.id);

            sqlx::query!(
                r#"
                INSERT INTO sweet_cash_transactions (
                    user_id, transaction_type, amount, balance_after,
                    related_order_id, description
                ) VALUES (?, ?, ?, ?, ?, ?)
                "#,
                user.id,
                transaction_type_str,
                sweet_cash_earned,
                balance_after,
                order_record.id,
                description
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            log::info!(
                "处理订单成功: {}, 用户: {:?}, 甜品现金奖励: {}",
                order_record.id,
                user.id,
                sweet_cash_earned
            );
        } else {
            log::debug!("订单无关联用户，跳过: {}", order_record.id);
        }

        Ok(())
    }

    pub async fn sync_discount_codes(&self) -> AppResult<usize> {
        let api = self.sevencloud_api.lock().await;
        let coupons = api.get_discount_codes(None).await?;

        let mut processed_count = 0;

        for coupon_record in coupons {
            if let Err(e) = self.process_discount_code(coupon_record).await {
                log::error!("处理优惠码失败: {:?}", e);
                continue;
            }
            processed_count += 1;
        }

        log::info!("同步优惠码完成，处理数量: {}", processed_count);
        Ok(processed_count)
    }

    async fn process_discount_code(&self, _coupon_record: CouponRecord) -> AppResult<()> {
        // 这里可以实现优惠码同步逻辑
        // 比如更新本地优惠码的使用状态等
        Ok(())
    }
}
