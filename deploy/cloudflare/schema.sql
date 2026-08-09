-- 🛡️ AI-Forge Cloudflare D1 边缘数据库 Schema (v1.0.0)

-- 1. 设备指纹与每日免费额度表
CREATE TABLE IF NOT EXISTS devices (
    device_fingerprint TEXT PRIMARY KEY NOT NULL, -- 硬件指纹 HMAC-SHA256 (主键)
    daily_limit INTEGER NOT NULL DEFAULT 100,      -- 每日免费赠送额度 (默认 100 点)
    used_points_today INTEGER NOT NULL DEFAULT 0,  -- 今日已用额度
    bonus_points INTEGER NOT NULL DEFAULT 0,       -- 永久/付费赠送额度
    last_reset_utc_date TEXT NOT NULL,             -- 上次重置的 UTC 日期 (例如 "2026-08-08")
    status TEXT NOT NULL DEFAULT 'active',         -- 设备状态 ('active', 'banned')
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 2. 算力扣减与审计日志表
CREATE TABLE IF NOT EXISTS quota_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_fingerprint TEXT NOT NULL,              -- 外键关联设备指纹
    tool_name TEXT NOT NULL,                       -- 调用的工具名 (如 'tool-pdf-parse')
    points_deducted INTEGER NOT NULL,             -- 扣减的点数 (如 1 点)
    client_ip TEXT,                                -- 客户端 IP (安全审计)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (device_fingerprint) REFERENCES devices(device_fingerprint)
);

-- 3. 性能优化索引
CREATE INDEX IF NOT EXISTS idx_devices_fingerprint ON devices(device_fingerprint);
CREATE INDEX IF NOT EXISTS idx_logs_fingerprint_date ON quota_logs(device_fingerprint, created_at);
