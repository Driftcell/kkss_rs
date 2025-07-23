use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use crate::config::SevenCloudConfig;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: String,
    pub message: String,
    pub data: Option<T>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrdersData {
    pub records: Vec<OrderRecord>,
    pub total: i64,
    pub size: i64,
    pub current: i64,
    pub pages: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRecord {
    pub id: i64,
    #[serde(rename = "createDate")]
    pub create_date: i64,
    #[serde(rename = "memberCode")]
    pub member_code: Option<String>,
    pub price: Option<f64>,
    #[serde(rename = "productName")]
    pub product_name: String,
    #[serde(rename = "productNo")]
    pub product_no: Option<String>,
    pub status: i32,
    #[serde(rename = "payType")]
    pub pay_type: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CouponsData {
    pub records: Vec<CouponRecord>,
    pub total: i64,
    pub size: i64,
    pub current: i64,
    pub pages: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CouponRecord {
    pub id: i64,
    #[serde(rename = "createDate")]
    pub create_date: i64,
    pub code: i64,
    #[serde(rename = "isUse")]
    pub is_use: String,
    #[serde(rename = "useDate")]
    pub use_date: Option<i64>,
    #[serde(rename = "useBy")]
    pub use_by: Option<String>,
    pub discount: f64,
}

pub struct SevenCloudAPI {
    client: Client,
    config: SevenCloudConfig,
    token: Option<String>,
    admin_id: Option<i64>,
    username: Option<String>,
}

impl SevenCloudAPI {
    pub fn new(config: SevenCloudConfig) -> Self {
        Self {
            client: Client::new(),
            config,
            token: None,
            admin_id: None,
            username: None,
        }
    }

    pub async fn login(&mut self) -> AppResult<()> {
        let url = format!("{}/SZWL-SERVER/tAdmin/loginSys", self.config.base_url);
        let password_hash = format!("{:x}", md5::compute(&self.config.password));

        let data = serde_json::json!({
            "username": self.config.username,
            "password": password_hash,
        });

        let response = self.client
            .post(&url)
            .json(&data)
            .send()
            .await?;

        let result: ApiResponse<serde_json::Value> = response.json().await?;

        if !result.success {
            return Err(AppError::ExternalApiError(
                format!("七云登录失败: {}", result.message)
            ));
        }

        let data = result.data.ok_or_else(|| {
            AppError::ExternalApiError("七云登录响应数据为空".to_string())
        })?;

        self.admin_id = data["id"].as_i64();
        self.username = data["name"].as_str().map(|s| s.to_string());
        self.token = data["currentToken"].as_str().map(|s| s.to_string());

        log::info!("七云API登录成功, admin_id: {:?}", self.admin_id);

        Ok(())
    }

    pub async fn get_orders(&self, start_date: &str, end_date: &str) -> AppResult<Vec<OrderRecord>> {
        self.ensure_logged_in()?;

        let url = format!("{}/ORDER-SERVER/tOrder/pageOrder", self.config.base_url);
        let mut all_orders = Vec::new();
        let mut current_page = 1;

        loop {
            let mut params = HashMap::new();
            params.insert("adminId", self.admin_id.unwrap().to_string());
            params.insert("userName", self.username.as_ref().unwrap().clone());
            params.insert("adminType", "".to_string());
            params.insert("type", "".to_string());
            params.insert("payType", "".to_string());
            params.insert("productNo", "".to_string());
            params.insert("clientId", "".to_string());
            params.insert("dateType", "0".to_string());
            params.insert("startDate", start_date.to_string());
            params.insert("endDate", end_date.to_string());
            params.insert("current", current_page.to_string());
            params.insert("size", "100".to_string());
            params.insert("status", "1".to_string());
            params.insert("companyType", "".to_string());
            params.insert("machineType", "".to_string());
            params.insert("ifForeign", "".to_string());
            params.insert("chartType", "day".to_string());

            let response = self.client
                .get(&url)
                .query(&params)
                .header("Authorization", self.token.as_ref().unwrap())
                .send()
                .await?;

            let result: ApiResponse<OrdersData> = response.json().await?;

            if !result.success {
                return Err(AppError::ExternalApiError(
                    format!("获取订单失败: {}", result.message)
                ));
            }

            let page_data = result.data.ok_or_else(|| {
                AppError::ExternalApiError("订单数据为空".to_string())
            })?;

            all_orders.extend(page_data.records);

            if current_page >= page_data.pages {
                break;
            }

            current_page += 1;
        }

        Ok(all_orders)
    }

    pub async fn get_discount_codes(&self, is_use: Option<bool>) -> AppResult<Vec<CouponRecord>> {
        self.ensure_logged_in()?;

        let url = format!("{}/SZWL-SERVER/tPromoCode/list", self.config.base_url);
        let mut all_coupons = Vec::new();
        let mut current_page = 1;

        loop {
            let mut data = serde_json::json!({
                "adminId": self.admin_id.unwrap(),
                "current": current_page,
                "size": 20,
            });

            if let Some(is_use) = is_use {
                data["isUse"] = serde_json::Value::String(if is_use { "1" } else { "0" }.to_string());
            }

            let response = self.client
                .post(&url)
                .json(&data)
                .header("Authorization", self.token.as_ref().unwrap())
                .send()
                .await?;

            let result: ApiResponse<CouponsData> = response.json().await?;

            if !result.success {
                return Err(AppError::ExternalApiError(
                    format!("获取优惠码失败: {}", result.message)
                ));
            }

            let page_data = result.data.ok_or_else(|| {
                AppError::ExternalApiError("优惠码数据为空".to_string())
            })?;

            all_coupons.extend(page_data.records);

            if current_page >= page_data.pages {
                break;
            }

            current_page += 1;
        }

        Ok(all_coupons)
    }

    pub async fn generate_discount_code(
        &self,
        code: &str,
        discount: f64,
        expire_months: u32,
    ) -> AppResult<bool> {
        self.ensure_logged_in()?;

        if code.len() != 6 || !code.chars().all(|c| c.is_digit(10)) {
            return Err(AppError::ValidationError("优惠码必须是6位数字".to_string()));
        }

        if discount <= 0.0 {
            return Err(AppError::ValidationError("折扣金额必须大于0".to_string()));
        }

        if expire_months == 0 || expire_months > 3 {
            return Err(AppError::ValidationError("有效期必须在1-3个月之间".to_string()));
        }

        let url = format!("{}/SZWL-SERVER/tPromoCode/add", self.config.base_url);

        let mut params = HashMap::new();
        params.insert("addMode", "2".to_string());
        params.insert("codeNum", code.to_string());
        params.insert("number", "1".to_string());
        params.insert("month", expire_months.to_string());
        params.insert("type", "1".to_string());
        params.insert("discount", discount.to_string());
        params.insert("frpCode", "WEIXIN_NATIVE".to_string());
        params.insert("adminId", self.admin_id.unwrap().to_string());

        let response = self.client
            .get(&url)
            .query(&params)
            .header("Authorization", self.token.as_ref().unwrap())
            .send()
            .await?;

        let result: ApiResponse<String> = response.json().await?;

        if !result.success {
            return Err(AppError::ExternalApiError(
                format!("生成优惠码失败: {}", result.message)
            ));
        }

        log::info!("成功生成优惠码: {}, 金额: {}, 有效期: {}个月", code, discount, expire_months);

        Ok(true)
    }

    fn ensure_logged_in(&self) -> AppResult<()> {
        if self.token.is_none() || self.admin_id.is_none() {
            return Err(AppError::ExternalApiError("未登录七云API".to_string()));
        }
        Ok(())
    }
}
