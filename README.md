# KKSS Backend

基于Rust actix-web框架的冰淇凌推广网站后端系统，主要为消费者提供会员管理、优惠码兑换、充值等功能的API服务。

## 技术栈

- **Web框架**: actix-web 4.x
- **数据库**: PostgreSQL
- **ORM**: sqlx
- **认证**: JWT
- **外部服务**: Twilio (短信), Stripe (支付), 七云API

## 功能特性

- 🔐 用户认证 (注册/登录/JWT)
- 📱 手机验证码 (Twilio)
- 👥 会员体系 (粉丝/甜品股东/超级股东)
- 🎫 优惠码管理
- 💰 充值系统 (Stripe)
- 🍦 甜品现金奖励
- 📊 订单同步 (七云API)
- 🔄 数据同步任务

## 快速开始

### 1. 环境准备

确保已安装 Rust 1.70+:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. 项目设置

```bash
# 克隆项目
git clone <repository-url>
cd kkss-backend

# 复制配置文件
cp config.toml.example config.toml

# 编辑配置文件，填入实际的API密钥
vim config.toml
```

### 3. 数据库初始化

```bash
# 安装 sqlx-cli
cargo install sqlx-cli

# 创建数据库
sqlx database create

# 运行迁移
sqlx migrate run
```

### 4. 运行项目

```bash
# 开发模式
cargo run

# 或者使用 cargo watch 自动重启
cargo install cargo-watch
cargo watch -x run
```

服务器将在 `http://localhost:8080` 启动。

## API 文档

### 认证模块

#### POST `/api/v1/auth/send-code`
发送手机验证码

```json
{
  "phone": "+1234567890"
}
```

#### POST `/api/v1/auth/register`
用户注册

```json
{
  "phone": "+1234567890",
  "verification_code": "123456",
  "username": "用户名",
  "password": "Password123",
  "birthday": "1990-01-01",
  "referrer_code": "1000000001"
}
```

#### POST `/api/v1/auth/login`
用户登录

```json
{
  "phone": "+1234567890",
  "password": "Password123"
}
```

### 用户模块

#### GET `/api/v1/user/profile`
获取用户信息 (需要认证)

#### PUT `/api/v1/user/profile`
更新用户信息 (需要认证)

#### GET `/api/v1/user/referrals`
获取推荐用户列表 (需要认证)

### 订单模块

#### GET `/api/v1/orders`
获取用户订单列表 (需要认证)

### 优惠码模块

#### GET `/api/v1/discount-codes`
获取用户优惠码列表 (需要认证)

#### POST `/api/v1/discount-codes/redeem`
兑换优惠码 (需要认证)

### 充值模块

#### POST `/api/v1/recharge/create-payment-intent`
创建支付意图 (需要认证)

#### POST `/api/v1/recharge/confirm`
确认充值 (需要认证)

#### GET `/api/v1/recharge/history`
获取充值历史 (需要认证)

## 配置说明

配置文件使用TOML格式，支持环境变量覆盖：

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://postgres:postgres@localhost:5432/kkss"
max_connections = 10

[jwt]
secret = "your-jwt-secret"
access_token_expires_in = 7200
refresh_token_expires_in = 2592000

[twilio]
account_sid = "your-twilio-account-sid"
auth_token = "your-twilio-auth-token"
from_phone = "+1234567890"

[stripe]
secret_key = "sk_test_your-stripe-secret-key"
webhook_secret = "whsec_your-webhook-secret"

[sevencloud]
username = "your-sevencloud-username"
password = "your-sevencloud-password"
base_url = "https://sz.sunzee.com.cn"
```

### 环境变量

可以通过环境变量覆盖配置：

- `DATABASE_URL` - 数据库连接字符串
- `JWT_SECRET` - JWT密钥
- `TWILIO_ACCOUNT_SID` - Twilio账户SID
- `TWILIO_AUTH_TOKEN` - Twilio认证令牌
- `STRIPE_SECRET_KEY` - Stripe密钥
- `STRIPE_WEBHOOK_SECRET` - Stripe Webhook密钥
- `SEVENCLOUD_USERNAME` - 七云用户名
- `SEVENCLOUD_PASSWORD` - 七云密码

## 数据库设计

### 主要表结构

- `users` - 用户表
- `orders` - 订单表
- `discount_codes` - 优惠码表
- `recharge_records` - 充值记录表
- `sweet_cash_transactions` - 甜品现金交易记录表

## 使用 Podman 启动 PostgreSQL

```bash
podman run -d \
  --name kkss-postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=kkss \
  -p 5432:5432 \
  docker.io/library/postgres:16

export DATABASE_URL="postgres://postgres:postgres@localhost:5432/kkss"

# 初始化数据库并运行迁移
scripts/init_db.sh

# 运行服务
cargo run
```

停止与移除容器：

```bash
podman stop kkss-postgres && podman rm kkss-postgres
```

详细的数据库结构请参考 `migrations/` 目录下的迁移文件。

## 部署

### Docker 部署

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/kkss-backend .
COPY config.toml .

EXPOSE 8080
CMD ["./kkss-backend"]
```

### 生产环境

1. 使用PostgreSQL替代SQLite
2. 设置合适的环境变量
3. 配置反向代理 (nginx)
4. 设置SSL证书
5. 配置日志轮转
6. 设置监控和报警

## 开发

### 项目结构

```
src/
├── main.rs              # 主程序入口
├── lib.rs               # 库入口
├── config.rs            # 配置管理
├── error.rs             # 错误处理
├── database/            # 数据库连接
├── models/              # 数据模型
├── services/            # 业务逻辑
├── handlers/            # HTTP处理器
├── middlewares/         # 中间件
├── utils/               # 工具函数
└── external/            # 外部API集成
```

### 添加新功能

1. 在 `models/` 中定义数据模型
2. 在 `services/` 中实现业务逻辑
3. 在 `handlers/` 中添加HTTP处理器
4. 在 `main.rs` 中注册路由

### 测试

```bash
# 运行测试
cargo test

# 运行特定测试
cargo test test_name

# 生成测试覆盖率报告
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## 许可证

[MIT License](LICENSE)
