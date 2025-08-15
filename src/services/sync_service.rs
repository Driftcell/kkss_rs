use crate::error::AppResult;
use crate::external::*;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};

#[derive(Clone)]
pub struct SyncService {
    pool: DatabaseConnection,
    sevencloud_api: std::sync::Arc<tokio::sync::Mutex<SevenCloudAPI>>,
}

impl SyncService {
    pub fn new(
        pool: DatabaseConnection,
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
                log::error!("Failed to process order: {e:?}");
                continue;
            }
            processed_count += 1;
        }

        log::info!("Synchronization complete, processed orders: {processed_count}");
        Ok(processed_count)
    }

    /// 处理七云订单
    async fn process_order(&self, order_record: OrderRecord) -> AppResult<()> {
        // 检查订单是否已存在
        let existing = self
            .pool
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT id FROM orders WHERE id = $1",
                vec![order_record.id.into()],
            ))
            .await?;

        if existing.is_some() {
            log::debug!("Order already exists, skipping: {}", order_record.id);
            return Ok(());
        }

        // 根据会员号查找用户
        let user_row = if let Some(member_code) = &order_record.member_code {
            self.pool
                .query_one(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    "SELECT id, referrer_id, balance FROM users WHERE member_code = $1",
                    vec![member_code.clone().into()],
                ))
                .await?
        } else {
            None
        };

        if let Some(user) = user_row {
            let user_id_db: i64 = user.try_get_by("id")?;
            let referrer_id_opt: Option<i64> = user.try_get_by("referrer_id").ok();
            // 开始事务
            let txn = self.pool.begin().await?;

            // 插入订单记录
            let created_at = chrono::DateTime::from_timestamp_millis(order_record.create_date)
                .unwrap_or_default();
            let price_cents: i64 = (order_record.price.unwrap_or(0.0) * 100.0) as i64;

            txn.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"
                INSERT INTO orders (
                    id, user_id, member_code, price, product_name, product_no,
                    order_status, pay_type, stamps_earned, external_created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                vec![
                    order_record.id.into(),
                    user_id_db.into(),
                    order_record.member_code.clone().into(),
                    price_cents.into(),
                    order_record.product_name.clone().into(),
                    order_record.product_no.clone().into(),
                    (order_record.status as i64).into(),
                    order_record.pay_type.unwrap_or_default().into(),
                    1i64.into(),
                    created_at.into(),
                ],
            ))
            .await?;

            // 新订单 +1 个 stamp
            txn.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE users SET stamps = COALESCE(stamps, 0) + 1 WHERE id = $1",
                vec![user_id_db.into()],
            ))
            .await?;

            // 订单返利：若用户存在推荐人，则用户与推荐人各获得订单金额的 10%
            // 只有存在推荐人时才发放双方各 10% 返利
            if let Some(referrer_id) = referrer_id_opt {
                if price_cents > 0 {
                    let rebate = price_cents / 10; // 向下取整
                    if rebate > 0 {
                        // 下单用户返利
                        let user_row = txn.query_one(Statement::from_sql_and_values(
                            DatabaseBackend::Postgres,
                            "UPDATE users SET balance = COALESCE(balance,0) + $1 WHERE id = $2 RETURNING balance",
                            vec![rebate.into(), user_id_db.into()],
                        )).await?;
                        let user_new_balance: i64 = user_row
                            .and_then(|r| r.try_get_by("balance").ok())
                            .unwrap_or(0);
                        txn.execute(Statement::from_sql_and_values(
                            DatabaseBackend::Postgres,
                            r#"INSERT INTO sweet_cash_transactions (
                                user_id, transaction_type, amount, balance_after, related_order_id, description
                            ) VALUES ($1, 'earn', $2, $3, $4, $5)"#,
                            vec![
                                user_id_db.into(),
                                rebate.into(),
                                user_new_balance.into(),
                                order_record.id.into(),
                                format!("Order rebate 10% for order {}", order_record.id).into(),
                            ],
                        )).await?;

                        // 推荐人返利
                        let ref_row = txn.query_one(Statement::from_sql_and_values(
                            DatabaseBackend::Postgres,
                            "UPDATE users SET balance = COALESCE(balance,0) + $1 WHERE id = $2 RETURNING balance",
                            vec![rebate.into(), referrer_id.into()],
                        )).await?;
                        let referrer_new_balance: i64 = ref_row
                            .and_then(|r| r.try_get_by("balance").ok())
                            .unwrap_or(0);
                        txn.execute(Statement::from_sql_and_values(
                            DatabaseBackend::Postgres,
                            r#"INSERT INTO sweet_cash_transactions (
                                user_id, transaction_type, amount, balance_after, related_order_id, description
                            ) VALUES ($1, 'earn', $2, $3, $4, $5)"#,
                            vec![
                                referrer_id.into(),
                                rebate.into(),
                                referrer_new_balance.into(),
                                order_record.id.into(),
                                format!("Referral order rebate 10% from user {} order {}", user_id_db, order_record.id).into(),
                            ],
                        )).await?;
                        log::info!(
                            "Order {} rebate distributed: user {} +{} cents & referrer {} +{} cents",
                            order_record.id,
                            user_id_db,
                            rebate,
                            referrer_id,
                            rebate
                        );
                    }
                }
            }

            txn.commit().await?;

            log::info!(
                "Successfully processed order: {}, User: {}, Stamps reward: {}",
                order_record.id,
                user_id_db,
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
                log::error!("Failed to process discount code: {e:?}");
                continue;
            }
            processed_count += 1;
        }

        log::info!("Synchronization complete, processed discount codes: {processed_count}");
        Ok(processed_count)
    }

    /// 处理七云优惠码
    async fn process_discount_code(&self, coupon_record: CouponRecord) -> AppResult<()> {
        // 同步逻辑：依据外部优惠码 code 字段（不使用 external_id），更新本地 is_used/used_at
        // _coupon_record.is_use: "0" 未使用, "1" 已使用
        let code_str = coupon_record.code.to_string();

        // 查询本地是否存在该优惠码
        let local = self
            .pool
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT id, is_used FROM discount_codes WHERE code = $1"#,
                vec![code_str.clone().into()],
            ))
            .await?;

        if local.is_none() {
            log::debug!(
                "Discount code not found locally, skipping sync: external_code={}",
                coupon_record.code
            );
            return Ok(());
        }
        let local = local.unwrap();
        let local_id: i64 = local.try_get_by("id")?;
        let local_is_used: bool = local.try_get_by("is_used").unwrap_or(false);

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
        if external_used && !local_is_used {
            // 转换 use_date (七云时间戳假定为毫秒)；若不存在则使用当前时间
            let used_at = coupon_record
                .use_date
                .and_then(chrono::DateTime::from_timestamp_millis)
                .unwrap_or_else(chrono::Utc::now);

            self.pool.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"UPDATE discount_codes SET is_used = TRUE, used_at = $1, updated_at = NOW() WHERE id = $2"#,
                vec![used_at.into(), local_id.into()],
            )).await?;

            log::info!(
                "Discount code marked as used via sync: code={}, id={:?}",
                coupon_record.code,
                local_id
            );
        } else if !external_used && local_is_used {
            // 外部显示未使用但本地已使用——通常不回滚，记录冲突
            log::warn!(
                "Usage state mismatch (local used, external unused), keeping local: code={}, id={:?}",
                coupon_record.code,
                local_id
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
