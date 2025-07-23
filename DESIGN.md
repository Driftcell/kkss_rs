## 项目介绍

基于Rust actix-web框架的冰淇凌推广网站后端系统，主要为消费者提供会员管理、优惠码兑换、充值等功能的API服务。

## 技术架构

### 核心技术栈
- **Web框架**: actix-web 4.x
- **数据库ORM**: sqlx 
- **数据库**: SQLite (开发阶段) → PostgreSQL (生产环境)
- **日志**: env_logger + structured logging
- **配置管理**: TOML配置文件
- **认证**: JWT (jsonwebtoken)

### 第三方服务集成
- **短信服务**: Twilio (仅支持美国手机号格式)
- **支付服务**: Stripe (充值功能)
- **第三方API**: 七云API (获取订单和优惠码数据)

## 项目需求

### 基础要求

#### 代码质量
- 良好的代码结构和模块化设计
- 完善的注释和文档
- 统一的错误处理机制
- 结构化日志记录

#### 配置管理
- 支持TOML配置文件
- 环境变量覆盖配置
- 敏感信息加密存储
- 配置项包括:
  - 数据库连接字符串
  - JWT密钥和过期时间
  - Twilio API凭证
  - Stripe API凭证
  - 七云API凭证
  - 服务器端口和地址

#### API设计
- RESTful API设计规范
- 统一的JSON响应格式
- 完善的错误码定义
- CORS跨域支持
- API版本控制 (v1)

#### 数据处理
- **金额存储**: 使用i64类型，单位为美分 (cents)
- **时间处理**: 统一使用UTC时间戳 (毫秒)
- **手机号验证**: 仅支持美国号码格式 (+1xxxxxxxxxx)
- **数据校验**: 输入参数严格校验

### 业务需求

#### 会员体系

**会员类型**:
- **甜品股东**: 支付$8加入，获得基础股东权益
- **超级股东**: 支付$30加入，获得高级股东权益  
- **粉丝**: 通过推荐链接免费注册，获得基础权益

**会员标识**:
- 10位数字会员号，系统自动生成
- 格式: 1000000001 - 9999999999
- 唯一性保证，不可重复

**甜品股东福利**:
- $8优惠码（可兑换任意配料冰淇淋一份）
- 15分钟免费营养咨询券（代码/链接形式）
- 专属推荐二维码/链接，用于邀请粉丝

**超级股东福利**:
- 10个$3一次性优惠码（有效期30天）
- 优惠码可分享给朋友使用
- 每次订单仅可使用一个优惠码
- 专属推荐二维码/链接

**粉丝福利**:
- 一个免费配料优惠码（价值$0.5）
- 可升级为股东获得更多权益

#### 奖励机制

**购买奖励**:
- 股东/粉丝每次购买获得$0.5甜品现金
- 甜品现金与实际金额1:1等值
- 甜品现金不可提现，仅用于兑换优惠码

**甜品现金兑换**:
- $5优惠码：需要1000个甜品现金积分（$5等值）
- $10优惠码：需要2000个甜品现金积分（$10等值）
- $20优惠码：需要4000个甜品现金积分（$20等值）
- $25优惠码：需要5000个甜品现金积分（$25等值）

**积分展示**:
- 兑换页面采用集章形式设计
- 每200个甜品现金积分 = 1个印章
- 10个印章 = $5优惠码（显示满章可兑换）

#### 充值奖励

充值奖励采用阶梯式优惠:
- **$100充值**: 获得$115余额 (15%奖励)
- **$200充值**: 获得$235余额 (17.5%奖励)  
- **$300充值**: 获得$375余额 (25%奖励)
- **$500充值**: 获得$650余额 (30%奖励)

**充值说明**:
- 充值通过Stripe支付系统处理
- 奖励金额立即到账
- 余额可用于购买商品或兑换优惠码

## 功能模块

### 用户认证模块

#### 注册流程
1. **手机号验证**: 
   - 验证美国手机号格式 (+1xxxxxxxxxx)
   - 通过Twilio发送6位数字验证码
   - 验证码有效期5分钟，支持重发 (限制：60秒内最多1次)

2. **用户信息收集**:
   - 手机号 (必需，作为唯一标识)
   - 用户名 (必需，2-20字符，支持中英文数字)
   - 密码 (必需，8-128字符，包含大小写字母和数字)
   - 生日 (必需，YYYY-MM-DD格式)
   - 推荐人 (可选，10位会员号或推荐链接)

3. **注册完成**:
   - 生成10位唯一会员号
   - 根据推荐情况确定会员类型
   - 发放对应会员福利
   - 返回JWT令牌

#### 登录流程
1. **凭证验证**:
   - 手机号 + 密码验证
   - 支持记住登录状态 (30天)
   - 失败5次后账户临时锁定 (15分钟)

2. **JWT令牌**:
   - 包含用户ID、会员号、权限等信息
   - 访问令牌有效期: 2小时
   - 刷新令牌有效期: 30天
   - 支持令牌续期

### 第三方数据同步模块

#### 七云API集成
基于原有`sevencloud.py`文件逻辑，使用Rust重新实现:

**认证机制**:
- 用户名/密码MD5加密认证
- 获取访问令牌 (token)
- 令牌自动续期机制

**数据获取功能**:
1. **订单数据同步**:
   - 支持按时间范围查询
   - 分页获取所有订单 (每页100条)
   - 包含订单详情和支付信息
   - 自动去重和增量同步

2. **优惠码数据同步**:
   - 获取所有优惠码状态
   - 支持按使用状态过滤
   - 实时同步使用情况

3. **优惠码生成**:
   - 即时调用API生成6位数字优惠码
   - 设置金额和有效期 (最长3个月)
   - 返回生成结果确认

#### 数据处理规则

**新订单处理**:
1. 检查订单是否已存在本地数据库
2. 如果是新订单:
   - 插入订单记录到本地数据库
   - 根据订单金额计算甜品现金奖励
   - 更新用户甜品现金余额
   - 记录奖励发放日志
3. 如果订单已存在:
   - 检查状态是否有更新
   - 更新本地记录状态

**数据同步策略**:
- 定时任务每15分钟同步一次新订单
- 每日凌晨全量同步前一天所有数据
- 异常时支持手动触发同步
- 同步失败时自动重试机制 (最多3次)

```python
import hashlib
from contextlib import asynccontextmanager
from typing import Annotated, List, Optional

import aiohttp
import structlog
from fastapi import Depends
from pydantic import BaseModel, Field, field_validator

from config import Config

logger: structlog.stdlib.BoundLogger = structlog.get_logger(__name__)

config = Config.from_toml()


class Order(BaseModel):
    """订单数据模型"""

    id: int = Field(..., description="订单ID")
    create_date: int = Field(..., description="创建时间戳（毫秒）", alias="createDate")
    member_code: Optional[str] = Field(None, description="会员ID", alias="memberCode")
    price: float = Field(..., description="订单价格")
    product_name: str = Field(..., description="产品名称", alias="productName")

    class Config:
        populate_by_name = True


class DiscountCode(BaseModel):
    """优惠码数据模型"""

    id: int = Field(..., description="优惠码ID")
    code: int = Field(..., description="优惠码")
    create_date: int = Field(..., description="创建时间戳（毫秒）", alias="createDate")
    use_by: Optional[str] = Field(None, description="使用者", alias="useBy")
    use_date: Optional[int] = Field(
        None, description="使用时间戳（毫秒）", alias="useDate"
    )
    is_use: bool = Field(..., description="是否已使用", alias="isUse")

    @field_validator("is_use", mode="before")
    @classmethod
    def validate_is_use(cls, v):
        """将字符串转换为布尔值：'0' -> False, '1' -> True"""
        if isinstance(v, str):
            return v == "1"
        return bool(v)


class SevenCloudAPI:
    def __init__(self, token: str, admin_id: int, username: str):
        """Do not instantiate this class directly, use the `login` method instead."""
        logger.debug("Initializing SevenCloudAPI", token=token, admin_id=admin_id)
        self._token = token
        self._admin_id = admin_id
        self._username = username

    async def get_orders(self, start_date: str, end_date: str) -> List[Order]:
        """
        获取订单数据（支持分页获取所有数据）

        Args:
            start_date: 开始日期，格式: 'YYYY-MM-DD HH:MM:SS'
            end_date: 结束日期，格式: 'YYYY-MM-DD HH:MM:SS'

        Returns:
            List[Order]: 订单列表
        """
        url = "https://sz.sunzee.com.cn/ORDER-SERVER/tOrder/pageOrder"
        all_orders = []
        current_page = 1

        while True:
            # 构建查询参数
            params = {
                "adminId": self._admin_id,
                "userName": self._username,
                "adminType": "",
                "type": "",
                "payType": "",
                "productNo": "",
                "clientId": "",
                "dateType": "0",
                "startDate": start_date,
                "endDate": end_date,
                "current": str(current_page),
                "size": "100",
                "status": "1",
                "companyType": "",
                "machineType": "",
                "ifForeign": "",
                "chartType": "day",
            }

            async with self._session() as session:
                try:
                    async with session.get(url, params=params) as response:
                        if response.status != 200:
                            logger.error(
                                "Get orders request failed",
                                status=response.status,
                                reason=response.reason,
                                page=current_page,
                            )
                            break

                        data = await response.json()
                        logger.debug(
                            "Get orders response", response=data, page=current_page
                        )

                        if not data.get("success") or data.get("code") != "00000":
                            logger.error(
                                "Get orders failed", response=data, page=current_page
                            )
                            break

                        page_data = data.get("data", {})
                        records = page_data.get("records", [])
                        total_pages = page_data.get("pages", 1)
                        total_records = page_data.get("total", 0)

                        logger.debug(
                            "Processing page",
                            page=current_page,
                            total_pages=total_pages,
                            records_in_page=len(records),
                            total_records=total_records,
                        )

                        # 解析当前页的订单记录
                        page_orders = []
                        for record in records:
                            try:
                                order = Order(**record)
                                page_orders.append(order)
                            except Exception as e:
                                logger.warning(
                                    "Failed to parse order record",
                                    record=record,
                                    error=str(e),
                                    page=current_page,
                                )
                                continue

                        all_orders.extend(page_orders)

                        # 检查是否还有更多页
                        if current_page >= total_pages:
                            logger.debug(
                                "All pages processed",
                                total_pages=total_pages,
                                total_orders=len(all_orders),
                            )
                            break

                        current_page += 1

                except Exception as e:
                    logger.error(
                        "Get orders request exception", exc_info=e, page=current_page
                    )
                    break

        return all_orders

    async def generate_discount_code(
        self, code: str, discount: float, expire_in: int
    ) -> bool:
        """
        生成优惠码

        Args:
            code: 优惠码内容，六位数字
            discount: 折扣金额，单位：元
            expire_in: 过期时间（单位：月），不超过3个月

        Returns:
            bool: 是否成功生成优惠码
        """
        # 验证输入参数
        if not isinstance(code, str) or len(code) != 6 or not code.isdigit():
            logger.error("Invalid discount code format", code=code)
            return False

        if not isinstance(discount, (int, float)) or discount <= 0:
            logger.error("Invalid discount amount", discount=discount)
            return False

        if not isinstance(expire_in, int) or expire_in <= 0 or expire_in > 3:
            logger.error("Invalid expire_in value", expire_in=expire_in)
            return False

        url = "https://sz.sunzee.com.cn/SZWL-SERVER/tPromoCode/add"

        # 构建查询参数
        params = {
            "addMode": "2",
            "codeNum": code,
            "number": "1",
            "month": str(expire_in),
            "type": "1",
            "discount": str(discount),
            "frpCode": "WEIXIN_NATIVE",
            "adminId": str(self._admin_id),
        }

        async with self._session() as session:
            try:
                async with session.get(url, params=params) as response:
                    if response.status != 200:
                        logger.error(
                            "Generate discount code request failed",
                            status=response.status,
                            reason=response.reason,
                            code=code,
                        )
                        return False

                    data = await response.json()
                    logger.debug(
                        "Generate discount code response",
                        response=data,
                        code=code,
                    )

                    if not data.get("success") or data.get("code") != "00000":
                        logger.error(
                            "Generate discount code failed",
                            response=data,
                            code=code,
                        )
                        return False

                    logger.info(
                        "Discount code generated successfully",
                        code=code,
                        discount=discount,
                        expire_in=expire_in,
                        message=data.get("data", ""),
                    )
                    return True

            except Exception as e:
                logger.error(
                    "Generate discount code request exception",
                    exc_info=e,
                    code=code,
                )
                return False

    async def get_discount_codes(
        self, is_use: Optional[bool] = False
    ) -> List[DiscountCode]:
        """
        获取优惠码数据（支持分页获取所有数据）

        Args:
            is_use: 是否已使用，True表示已使用，False表示未使用，None表示获取所有，默认False

        Returns:
            List[DiscountCode]: 优惠码列表
        """
        url = "https://sz.sunzee.com.cn/SZWL-SERVER/tPromoCode/list"
        all_discount_codes = []
        current_page = 1

        while True:
            # 构建查询参数
            data = {
                "adminId": self._admin_id,
                "current": current_page,
                "size": 20,
            }

            # 如果指定了 is_use 参数，则添加到查询中
            if is_use is not None:
                data["isUse"] = "1" if is_use else "0"

            async with self._session() as session:
                try:
                    async with session.post(url, json=data) as response:
                        if response.status != 200:
                            logger.error(
                                "Get discount codes request failed",
                                status=response.status,
                                reason=response.reason,
                                page=current_page,
                            )
                            break

                        data = await response.json()
                        logger.debug(
                            "Get discount codes response",
                            response=data,
                            page=current_page,
                        )

                        if not data.get("success") or data.get("code") != "00000":
                            logger.error(
                                "Get discount codes failed",
                                response=data,
                                page=current_page,
                            )
                            break

                        page_data = data.get("data", {})
                        records = page_data.get("records", [])
                        total_pages = page_data.get("pages", 1)
                        total_records = page_data.get("total", 0)

                        logger.debug(
                            "Processing discount codes page",
                            page=current_page,
                            total_pages=total_pages,
                            records_in_page=len(records),
                            total_records=total_records,
                        )

                        # 解析当前页的优惠码记录
                        page_discount_codes = []
                        for record in records:
                            try:
                                discount_code = DiscountCode(**record)
                                page_discount_codes.append(discount_code)
                            except Exception as e:
                                logger.warning(
                                    "Failed to parse discount code record",
                                    record=record,
                                    error=str(e),
                                    page=current_page,
                                )
                                continue

                        all_discount_codes.extend(page_discount_codes)

                        # 检查是否还有更多页
                        if current_page >= total_pages:
                            logger.debug(
                                "All discount code pages processed",
                                total_pages=total_pages,
                                total_discount_codes=len(all_discount_codes),
                            )
                            break

                        current_page += 1

                except Exception as e:
                    logger.error(
                        "Get discount codes request exception",
                        exc_info=e,
                        page=current_page,
                    )
                    break

        return all_discount_codes

    @asynccontextmanager
    async def _session(self):
        headers = {"Authorization": self._token}
        async with aiohttp.ClientSession(headers=headers) as session:
            yield session

    @classmethod
    async def login(cls, username: str, password: str) -> Optional["SevenCloudAPI"]:
        url = "https://sz.sunzee.com.cn/SZWL-SERVER/tAdmin/loginSys"
        password_hash = hashlib.md5(password.encode()).hexdigest()

        data = {
            "username": username,
            "password": password_hash,
        }
        async with aiohttp.ClientSession() as session:
            try:
                async with session.post(url, json=data) as response:
                    if response.status != 200:
                        logger.debug(
                            "Login request failed",
                            status=response.status,
                            reason=response.reason,
                        )
                        return None

                    data = await response.json()
                    logger.debug("Login response", response=data)

                    if not data.get("success"):
                        logger.debug("Login failed", response=data)
                        return None

                    admin_id = data["data"]["id"]
                    user_name = data["data"]["name"]
                    token = data["data"]["currentToken"]

                    return cls(token=token, admin_id=admin_id, username=user_name)

            except Exception as e:
                logger.error("Login request exception", exc_info=e)
                return None
```

返回订单详细API的参考Struct定义:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub code: String,
    pub message: String,
    pub data: Option<Data>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub records: Vec<Record>,
    pub total: i64,
    pub size: i64,
    pub current: i64,
    pub orders: Vec<Order>, // 内容未知，暂定义为空
    pub optimizeCountSql: bool,
    pub hitCount: bool,
    pub countId: Option<String>,
    pub maxLimit: Option<i64>,
    pub searchCount: bool,
    pub pages: i64,
}

// 如果 orders 字段永远为空数组，可以用 ()
pub type Order = ();

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
    pub createDate: i64,
    pub modifyDate: i64,
    pub adminId: i64,
    pub clientId: String,
    pub payType: i64,
    pub price: Option<f64>,
    pub tax: Option<f64>,
    pub sn: String,
    #[serde(rename = "type")]
    pub type_field: Option<i64>,
    pub productName: String,
    pub es: String,
    pub payDate: i64,
    pub status: i64,
    pub productNo: Option<String>,
    pub productNumber: i64,
    pub note: Option<String>,
    pub adminProportion: Option<f64>,
    pub agencyId: Option<i64>,
    pub agencyProportion: Option<f64>,
    pub altInfo: Option<String>,
    pub equipmentId: i64,
    pub frpCode: Option<String>,
    pub merchantId: Option<i64>,
    pub merchantProportion: Option<f64>,
    pub personageId: Option<i64>,
    pub personageProportion: Option<f64>,
    pub productId: Option<i64>,
    pub refundDate: Option<i64>,
    pub productDesc: Option<String>,
    pub trxNo: Option<String>,
    pub refundId: Option<i64>,
    pub refundAmount: Option<f64>,
    pub proportionDesc: Option<String>,
    pub marketingAmount: Option<f64>,
    pub refundMarketingAmount: Option<f64>,
    pub orderStatus: Option<i64>,
    pub currency: Option<String>,
    pub merchantOrderId: Option<String>,
    pub requestId: Option<String>,
    pub paymentIntentId: Option<String>,
    pub companyType: String,
    pub refundQuantity: Option<i64>,
    pub isAir: String,
    pub amount: Option<f64>,
    pub machineType: String,
    pub memberCode: Option<String>,
    pub orderDetails: Vec<OrderDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderDetail {
    pub id: String,
    pub adminId: i64,
    pub equipmentId: i64,
    pub createDate: i64,
    pub orderSn: String,
    pub productNo: String,
    pub productName: String,
    pub productNumber: i64,
    pub price: Option<f64>,
    pub amount: Option<f64>,
    pub refundQuantity: Option<i64>,
    pub refundAmount: Option<f64>,
    pub refundStatus: String,
    pub companyType: String,
    pub machineType: String,
}
```

创建优惠码的API参考Struct定义:

```rust
use serde::{Deserialize, Serialize};

// 顶层结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponseSimple {
    pub code: String,
    pub message: String,
    pub data: String,
    pub success: bool,
}
```

列出优惠码的API参考Struct定义:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CouponApiResponse {
    pub code: String,
    pub message: String,
    pub data: Option<CouponData>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CouponData {
    pub records: Vec<CouponRecord>,
    pub total: i64,
    pub size: i64,
    pub current: i64,
    pub orders: Vec<serde_json::Value>, // 结构未知，暂用Value
    pub optimizeCountSql: bool,
    pub hitCount: bool,
    pub countId: Option<String>,
    pub maxLimit: Option<i64>,
    pub searchCount: bool,
    pub pages: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CouponRecord {
    pub id: i64,
    pub adminId: String,
    pub createDate: i64,
    pub userName: String,
    pub modifyDate: Option<i64>,
    pub code: i64,
    pub isUse: String,
    pub useDate: Option<i64>,
    pub useBy: Option<String>,
    pub lastUseDate: Option<i64>,
    pub discount: f64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub wxId: Option<String>,
}
```

### 优惠码管理模块

#### 优惠码兑换
**兑换类型**:
1. **甜品现金兑换**:
   - 用户使用甜品现金兑换优惠码
   - 扣除对应甜品现金余额
   - 即时调用七云API生成优惠码
   - 记录兑换历史和优惠码信息

2. **福利优惠码发放**:
   - 新会员注册时自动发放
   - 系统自动生成并分配
   - 设置对应有效期

**兑换流程**:
1. 验证用户余额是否充足
2. 扣除对应甜品现金
3. 调用七云API生成优惠码
4. 保存优惠码到本地数据库
5. 返回优惠码信息给用户

#### 优惠码查询
- 分页查询用户所有优惠码
- 按状态筛选 (未使用/已使用/已过期)
- 显示优惠码详情 (金额、有效期、使用状态)

### 充值支付模块

#### Stripe集成
**支付流程**:
1. 用户选择充值金额
2. 创建Stripe PaymentIntent
3. 前端调用Stripe支付组件
4. 支付成功后webhook回调
5. 验证支付结果并更新用户余额
6. 应用充值奖励规则

**安全措施**:
- Webhook签名验证
- 防重复支付处理
- 支付金额双重验证
- 敏感信息加密存储

## 数据库设计

### 用户表 (users)
```sql
CREATE TABLE users (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    member_code VARCHAR(10) UNIQUE NOT NULL,      -- 10位会员号
    phone VARCHAR(20) UNIQUE NOT NULL,            -- 手机号
    username VARCHAR(50) NOT NULL,                -- 用户名
    password_hash VARCHAR(255) NOT NULL,          -- 密码哈希
    birthday DATE NOT NULL,                       -- 生日
    member_type ENUM('fan', 'sweet_shareholder', 'super_shareholder') NOT NULL,
    balance BIGINT DEFAULT 0,                     -- 余额(美分)
    sweet_cash BIGINT DEFAULT 0,                  -- 甜品现金(美分)
    referrer_id BIGINT NULL,                      -- 推荐人ID
    referral_code VARCHAR(32) UNIQUE,             -- 推荐码
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (referrer_id) REFERENCES users(id)
);
```

### 订单表 (orders)
```sql
CREATE TABLE orders (
    id BIGINT PRIMARY KEY,                        -- 七云订单ID
    user_id BIGINT NOT NULL,                      -- 用户ID
    member_code VARCHAR(10),                      -- 会员号
    price BIGINT NOT NULL,                        -- 订单金额(美分)
    product_name VARCHAR(255) NOT NULL,           -- 产品名称
    product_no VARCHAR(100),                      -- 产品编号
    order_status INT NOT NULL,                    -- 订单状态
    pay_type INT,                                 -- 支付方式
    sweet_cash_earned BIGINT DEFAULT 0,           -- 获得的甜品现金
    external_created_at TIMESTAMP NOT NULL,       -- 七云订单创建时间
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    INDEX idx_member_code (member_code),
    INDEX idx_external_created_at (external_created_at)
);
```

### 优惠码表 (discount_codes)
```sql
CREATE TABLE discount_codes (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id BIGINT NOT NULL,                      -- 所有者用户ID
    code VARCHAR(20) UNIQUE NOT NULL,             -- 优惠码
    discount_amount BIGINT NOT NULL,              -- 优惠金额(美分)
    code_type ENUM('welcome', 'referral', 'purchase_reward', 'redeemed') NOT NULL,
    is_used BOOLEAN DEFAULT FALSE,                -- 是否已使用
    used_at TIMESTAMP NULL,                       -- 使用时间
    expires_at TIMESTAMP NOT NULL,                -- 过期时间
    external_id BIGINT NULL,                      -- 七云优惠码ID
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    INDEX idx_user_id (user_id),
    INDEX idx_code (code),
    INDEX idx_expires_at (expires_at)
);
```

### 充值记录表 (recharge_records)
```sql
CREATE TABLE recharge_records (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id BIGINT NOT NULL,                      -- 用户ID
    stripe_payment_intent_id VARCHAR(255) UNIQUE NOT NULL,
    amount BIGINT NOT NULL,                       -- 充值金额(美分)
    bonus_amount BIGINT NOT NULL,                 -- 奖励金额(美分)
    total_amount BIGINT NOT NULL,                 -- 实际到账金额(美分)
    status ENUM('pending', 'succeeded', 'failed', 'canceled') NOT NULL,
    stripe_status VARCHAR(50),                    -- Stripe状态
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    INDEX idx_user_id (user_id),
    INDEX idx_stripe_payment_intent_id (stripe_payment_intent_id)
);
```

### 甜品现金交易记录表 (sweet_cash_transactions)
```sql
CREATE TABLE sweet_cash_transactions (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id BIGINT NOT NULL,                      -- 用户ID
    transaction_type ENUM('earn', 'redeem') NOT NULL,
    amount BIGINT NOT NULL,                       -- 交易金额(美分)
    balance_after BIGINT NOT NULL,                -- 交易后余额
    related_order_id BIGINT NULL,                 -- 关联订单ID
    related_discount_code_id BIGINT NULL,         -- 关联优惠码ID
    description TEXT,                             -- 交易描述
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (related_order_id) REFERENCES orders(id),
    FOREIGN KEY (related_discount_code_id) REFERENCES discount_codes(id),
    INDEX idx_user_id (user_id),
    INDEX idx_transaction_type (transaction_type)
);
```

## API接口设计

### 响应格式规范

**成功响应**:
```json
{
    "success": true,
    "data": {}, 
    "message": "操作成功"
}
```

**错误响应**:
```json
{
    "success": false,
    "error": {
        "code": "ERROR_CODE",
        "message": "错误描述",
        "details": {}
    }
}
```

**分页响应**:
```json
{
    "success": true,
    "data": {
        "items": [],
        "pagination": {
            "current_page": 1,
            "per_page": 20,
            "total": 100,
            "total_pages": 5
        }
    }
}
```

### 认证模块 (Auth)

#### POST `/api/v1/auth/send-code`
发送手机验证码

**请求参数**:
```json
{
    "phone": "+1234567890"  // 美国手机号格式
}
```

**响应**:
```json
{
    "success": true,
    "data": {
        "expires_in": 300  // 验证码有效期(秒)
    },
    "message": "验证码已发送"
}
```

#### POST `/api/v1/auth/register`
用户注册

**请求参数**:
```json
{
    "phone": "+1234567890",
    "verification_code": "123456",
    "username": "用户名",
    "password": "password123",
    "birthday": "1990-01-01",
    "referrer_code": "1000000001"  // 可选，推荐人会员号
}
```

**响应**:
```json
{
    "success": true,
    "data": {
        "user": {
            "id": 1,
            "member_code": "1000000001",
            "username": "用户名",
            "phone": "+1234567890",
            "member_type": "fan",
            "balance": 0,
            "sweet_cash": 50  // 注册奖励50美分甜品现金
        },
        "access_token": "jwt_token",
        "refresh_token": "refresh_token",
        "expires_in": 7200
    }
}
```

#### POST `/api/v1/auth/login`
用户登录

**请求参数**:
```json
{
    "phone": "+1234567890",
    "password": "password123",
    "remember_me": true  // 可选，是否记住登录
}
```

**响应**:
```json
{
    "success": true,
    "data": {
        "user": {
            "id": 1,
            "member_code": "1000000001",
            "username": "用户名",
            "member_type": "sweet_shareholder",
            "balance": 1500,
            "sweet_cash": 250
        },
        "access_token": "jwt_token",
        "refresh_token": "refresh_token",
        "expires_in": 7200
    }
}
```

#### POST `/api/v1/auth/refresh`
刷新访问令牌

**请求Header**: `Authorization: Bearer refresh_token`

**响应**: 同登录响应格式

#### POST `/api/v1/auth/logout`
用户登出

**请求Header**: `Authorization: Bearer access_token`

### 用户模块 (User)

#### GET `/api/v1/user/profile`
获取用户个人信息

**请求Header**: `Authorization: Bearer access_token`

**响应**:
```json
{
    "success": true,
    "data": {
        "user": {
            "id": 1,
            "member_code": "1000000001",
            "username": "用户名",
            "phone": "+1234567890",
            "birthday": "1990-01-01",
            "member_type": "sweet_shareholder",
            "balance": 1500,
            "sweet_cash": 250,
            "referral_code": "REF123456",
            "total_referrals": 5,
            "created_at": "2024-01-01T00:00:00Z"
        },
        "statistics": {
            "total_orders": 10,
            "total_spent": 5000,
            "total_earned_sweet_cash": 500,
            "available_discount_codes": 3
        }
    }
}
```

#### PUT `/api/v1/user/profile`
更新用户个人信息

**请求Header**: `Authorization: Bearer access_token`

**请求参数**:
```json
{
    "username": "新用户名",  // 可选
    "birthday": "1990-01-01"  // 可选
}
```

#### GET `/api/v1/user/referrals`
获取推荐的用户列表

**请求Header**: `Authorization: Bearer access_token`

**查询参数**:
- `page`: 页码 (默认1)
- `per_page`: 每页数量 (默认20，最大100)

**响应**:
```json
{
    "success": true,
    "data": {
        "items": [
            {
                "id": 2,
                "member_code": "1000000002",
                "username": "推荐用户1",
                "member_type": "fan",
                "joined_at": "2024-01-02T00:00:00Z"
            }
        ],
        "pagination": {
            "current_page": 1,
            "per_page": 20,
            "total": 5,
            "total_pages": 1
        }
    }
}
```

### 订单模块 (Order)

#### GET `/api/v1/orders`
获取用户订单列表

**请求Header**: `Authorization: Bearer access_token`

**查询参数**:
- `page`: 页码 (默认1)
- `per_page`: 每页数量 (默认20，最大100)
- `status`: 订单状态筛选 (可选)
- `start_date`: 开始日期 YYYY-MM-DD (可选)
- `end_date`: 结束日期 YYYY-MM-DD (可选)

**响应**:
```json
{
    "success": true,
    "data": {
        "items": [
            {
                "id": 12345,
                "product_name": "冰淇淋",
                "price": 800,  // 美分
                "sweet_cash_earned": 50,
                "order_status": 1,
                "external_created_at": "2024-01-01T10:00:00Z"
            }
        ],
        "pagination": {
            "current_page": 1,
            "per_page": 20,
            "total": 10,
            "total_pages": 1
        }
    }
}
```

### 优惠码模块 (DiscountCode)

#### GET `/api/v1/discount-codes`
获取用户优惠码列表

**请求Header**: `Authorization: Bearer access_token`

**查询参数**:
- `page`: 页码 (默认1)
- `per_page`: 每页数量 (默认20，最大100)
- `status`: 状态筛选 (available/used/expired，可选)
- `type`: 类型筛选 (welcome/referral/purchase_reward/redeemed，可选)

**响应**:
```json
{
    "success": true,
    "data": {
        "items": [
            {
                "id": 1,
                "code": "DISC123456",
                "discount_amount": 800,  // 美分
                "code_type": "welcome",
                "is_used": false,
                "expires_at": "2024-12-31T23:59:59Z",
                "created_at": "2024-01-01T00:00:00Z"
            }
        ],
        "pagination": {
            "current_page": 1,
            "per_page": 20,
            "total": 3,
            "total_pages": 1
        }
    }
}
```

#### POST `/api/v1/discount-codes/redeem`
兑换优惠码

**请求Header**: `Authorization: Bearer access_token`

**请求参数**:
```json
{
    "discount_amount": 500,  // 要兑换的优惠码金额(美分)
    "expire_months": 1       // 有效期(月)，1-3
}
```

**响应**:
```json
{
    "success": true,
    "data": {
        "discount_code": {
            "id": 2,
            "code": "DISC789012",
            "discount_amount": 500,
            "expires_at": "2024-02-01T23:59:59Z"
        },
        "sweet_cash_used": 500,
        "remaining_sweet_cash": 750
    }
}
```

### 充值模块 (Recharge)

#### POST `/api/v1/recharge/create-payment-intent`
创建充值支付意图

**请求Header**: `Authorization: Bearer access_token`

**请求参数**:
```json
{
    "amount": 10000  // 充值金额(美分)，支持: 10000, 20000, 30000, 50000
}
```

**响应**:
```json
{
    "success": true,
    "data": {
        "payment_intent_id": "pi_stripe_payment_intent_id",
        "client_secret": "pi_client_secret",
        "amount": 10000,
        "bonus_amount": 1500,
        "total_amount": 11500
    }
}
```

#### POST `/api/v1/recharge/confirm`
确认充值结果

**请求Header**: `Authorization: Bearer access_token`

**请求参数**:
```json
{
    "payment_intent_id": "pi_stripe_payment_intent_id"
}
```

**响应**:
```json
{
    "success": true,
    "data": {
        "recharge_record": {
            "id": 1,
            "amount": 10000,
            "bonus_amount": 1500,
            "total_amount": 11500,
            "status": "succeeded"
        },
        "new_balance": 12000
    }
}
```

#### GET `/api/v1/recharge/history`
获取充值历史

**请求Header**: `Authorization: Bearer access_token`

**查询参数**:
- `page`: 页码 (默认1)
- `per_page`: 每页数量 (默认20，最大100)

**响应**:
```json
{
    "success": true,
    "data": {
        "items": [
            {
                "id": 1,
                "amount": 10000,
                "bonus_amount": 1500,
                "total_amount": 11500,
                "status": "succeeded",
                "created_at": "2024-01-01T00:00:00Z"
            }
        ],
        "pagination": {
            "current_page": 1,
            "per_page": 20,
            "total": 1,
            "total_pages": 1
        }
    }
}
```