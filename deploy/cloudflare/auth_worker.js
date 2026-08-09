// 🛡️ AI-Forge Cloudflare Worker Serverless 鉴权中台 (v1.0.0)

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, X-Timestamp, X-Signature",
  "Content-Type": "application/json"
};

export default {
  async fetch(request, env) {
    // 1. 处理 OPTIONS 跨域预检请求
    if (request.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: CORS_HEADERS });
    }

    const url = new URL(request.url);

    // 2. 健康检查接口
    if (url.pathname === "/health") {
      return new Response(JSON.stringify({ status: "ok", service: "ai-forge-auth-worker" }), {
        status: 200,
        headers: CORS_HEADERS
      });
    }

    // 3. 额度校验与扣减接口
    if (url.pathname === "/api/v1/quota/deduct" && request.method === "POST") {
      return await handleDeductQuota(request, env);
    }

    return new Response(JSON.stringify({ error: "Not Found" }), { status: 404, headers: CORS_HEADERS });
  }
};

async function handleDeductQuota(request, env) {
  try {
    const bodyBytes = new Uint8Array(await request.arrayBuffer());
    const bodyText = new TextDecoder().decode(bodyBytes);
    let payload = {};
    try {
      payload = JSON.parse(bodyText);
    } catch (e) {
      return new Response(JSON.stringify({ success: false, error: "Invalid JSON payload" }), { status: 400, headers: CORS_HEADERS });
    }

    const { device_fingerprint, points_needed = 1, tool_name = "unknown" } = payload;
    if (!device_fingerprint) {
      return new Response(JSON.stringify({ success: false, error: "Missing device_fingerprint" }), { status: 400, headers: CORS_HEADERS });
    }

    // 1. HMAC 签名与时间戳提取
    const timestampStr = request.headers.get("X-Timestamp");
    const signatureHex = request.headers.get("X-Signature");
    const secret = env.RELAY_SECRET || "ai-forge-commercial-secret-2026";

    if (!timestampStr || !signatureHex) {
      return new Response(JSON.stringify({ success: false, error: "Missing X-Timestamp or X-Signature header" }), { status: 401, headers: CORS_HEADERS });
    }

    // 2. 防重放攻击校验 (时间漂移不能超过 300 秒)
    const nowSec = Math.floor(Date.now() / 1000);
    const reqSec = parseInt(timestampStr, 10);
    if (isNaN(reqSec) || Math.abs(nowSec - reqSec) > 300) {
      return new Response(JSON.stringify({ success: false, error: "Request timestamp drifted too far" }), { status: 403, headers: CORS_HEADERS });
    }

    // 3. 校验 HMAC 签名
    const isValid = await verifyHmac(secret, timestampStr, bodyBytes, signatureHex);
    if (!isValid) {
      return new Response(JSON.stringify({ success: false, error: "Invalid HMAC signature" }), { status: 401, headers: CORS_HEADERS });
    }

    const todayUtc = new Date().toISOString().split("T")[0]; // "YYYY-MM-DD"

    // 4. 查询 D1 数据库设备记录
    let device = await env.DB.prepare("SELECT * FROM devices WHERE device_fingerprint = ?").bind(device_fingerprint).first();

    if (!device) {
      // 首次点火新设备：静默注册并授予 100 点每日免费额度
      await env.DB.prepare(
        "INSERT INTO devices (device_fingerprint, daily_limit, used_points_today, last_reset_utc_date) VALUES (?, 100, 0, ?)"
      ).bind(device_fingerprint, todayUtc).run();

      device = {
        device_fingerprint,
        daily_limit: 100,
        used_points_today: 0,
        bonus_points: 0,
        last_reset_utc_date: todayUtc,
        status: "active"
      };
    }

    if (device.status === "banned") {
      return new Response(JSON.stringify({ success: false, error: "Device has been banned" }), { status: 403, headers: CORS_HEADERS });
    }

    // 5. 惰性重置逻辑 (Lazy Reset)
    let currentUsed = device.used_points_today;
    if (device.last_reset_utc_date !== todayUtc) {
      currentUsed = 0;
      await env.DB.prepare(
        "UPDATE devices SET used_points_today = 0, last_reset_utc_date = ?, updated_at = CURRENT_TIMESTAMP WHERE device_fingerprint = ?"
      ).bind(todayUtc, device_fingerprint).run();
    }

    // 6. 校验可用额度
    const totalAvailable = (device.daily_limit - currentUsed) + device.bonus_points;
    if (totalAvailable < points_needed) {
      return new Response(JSON.stringify({
        success: false,
        error: "Insufficient quota. Daily points exhausted.",
        remaining_points: Math.max(0, totalAvailable)
      }), { status: 402, headers: CORS_HEADERS });
    }

    // 7. 原子扣减用量并记录审计日志
    const newUsed = currentUsed + points_needed;
    await env.DB.prepare(
      "UPDATE devices SET used_points_today = ?, updated_at = CURRENT_TIMESTAMP WHERE device_fingerprint = ?"
    ).bind(newUsed, device_fingerprint).run();

    const clientIp = request.headers.get("CF-Connecting-IP") || "unknown";
    await env.DB.prepare(
      "INSERT INTO quota_logs (device_fingerprint, tool_name, points_deducted, client_ip) VALUES (?, ?, ?, ?)"
    ).bind(device_fingerprint, tool_name, points_needed, clientIp).run();

    const remainingAfter = totalAvailable - points_needed;

    return new Response(JSON.stringify({
      success: true,
      device_fingerprint,
      points_deducted: points_needed,
      remaining_points: remainingAfter
    }), { status: 200, headers: CORS_HEADERS });

  } catch (err) {
    return new Response(JSON.stringify({ success: false, error: err.message || "Internal Worker Error" }), { status: 500, headers: CORS_HEADERS });
  }
}

async function verifyHmac(secret, timestampStr, bodyBytes, expectedHex) {
  try {
    const encoder = new TextEncoder();
    const secretBytes = encoder.encode(secret);

    // 将 timestamp 转为 8 字节大端序 (u64be)
    const timestampNum = BigInt(timestampStr);
    const timestampBytes = new Uint8Array(8);
    let tempVal = timestampNum;
    for (let i = 7; i >= 0; i--) {
      timestampBytes[i] = Number(tempVal & 0xffn);
      tempVal >>= 8n;
    }

    // 拼接负载: timestampBytes + bodyBytes
    const payload = new Uint8Array(timestampBytes.length + bodyBytes.length);
    payload.set(timestampBytes, 0);
    payload.set(bodyBytes, timestampBytes.length);

    const key = await crypto.subtle.importKey(
      "raw",
      secretBytes,
      { name: "HMAC", hash: "SHA-256" },
      false,
      ["sign"]
    );

    const signature = new Uint8Array(await crypto.subtle.sign("HMAC", key, payload));
    const computedHex = [...signature].map(b => b.toString(16).padStart(2, "0")).join("");

    return computedHex.toLowerCase() === expectedHex.toLowerCase();
  } catch (e) {
    return false;
  }
}
