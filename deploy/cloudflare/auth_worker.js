// 🛡️ 紫电 AI - Cloudflare Worker Serverless 商业鉴权、隐私遥测与明细账单中台 (v3.1.0)

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, X-Timestamp, X-Signature",
  "Content-Type": "application/json"
};

const STAGE = "beta";
const DAILY_FREE_POINTS = STAGE === "beta" ? 600 : 300;

export default {
  async fetch(request, env) {
    if (request.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: CORS_HEADERS });
    }

    const url = new URL(request.url);

    if (url.pathname === "/health") {
      return new Response(JSON.stringify({ status: "ok", stage: STAGE, service: "zidian-auth-worker" }), {
        status: 200,
        headers: CORS_HEADERS
      });
    }

    // 2. 查询设备额度、状态与最近 5 笔消费明细
    if (url.pathname === "/api/v1/quota/status" && request.method === "POST") {
      return await handleQueryStatus(request, env);
    }

    if (url.pathname === "/api/v1/quota/deduct" && request.method === "POST") {
      return await handleDeductQuota(request, env);
    }

    if (url.pathname === "/api/v1/telemetry/report" && request.method === "POST") {
      return await handleTelemetryReport(request, env);
    }

    return new Response(JSON.stringify({ error: "Not Found" }), { status: 404, headers: CORS_HEADERS });
  }
};

async function handleQueryStatus(request, env) {
  try {
    const payload = await request.json();
    const { device_fingerprint } = payload;
    if (!device_fingerprint) {
      return new Response(JSON.stringify({ success: false, error: "Missing device_fingerprint" }), { status: 400, headers: CORS_HEADERS });
    }

    const todayUtc = new Date().toISOString().split("T")[0];
    let device = await env.DB.prepare("SELECT * FROM devices WHERE device_fingerprint = ?").bind(device_fingerprint).first();

    if (!device) {
      await env.DB.prepare(
        "INSERT INTO devices (device_fingerprint, daily_limit, used_points_today, last_reset_utc_date) VALUES (?, ?, 0, ?)"
      ).bind(device_fingerprint, DAILY_FREE_POINTS, todayUtc).run();

      device = {
        device_fingerprint,
        daily_limit: DAILY_FREE_POINTS,
        used_points_today: 0,
        bonus_points: 0,
        last_reset_utc_date: todayUtc,
        status: "active"
      };
    }

    let currentUsed = device.used_points_today;
    if (device.last_reset_utc_date !== todayUtc) {
      currentUsed = 0;
      await env.DB.prepare(
        "UPDATE devices SET used_points_today = 0, last_reset_utc_date = ?, updated_at = CURRENT_TIMESTAMP WHERE device_fingerprint = ?"
      ).bind(todayUtc, device_fingerprint).run();
    }

    const remaining = Math.max(0, (device.daily_limit - currentUsed) + (device.bonus_points || 0));

    // 🌟 查询该设备最近 5 笔扣费记录
    const recentLogsQuery = await env.DB.prepare(
      "SELECT id, tool_name, points_deducted, created_at FROM quota_logs WHERE device_fingerprint = ? ORDER BY id DESC LIMIT 5"
    ).bind(device_fingerprint).all();

    return new Response(JSON.stringify({
      success: true,
      device_fingerprint,
      stage: STAGE,
      daily_limit: device.daily_limit,
      used_today: currentUsed,
      bonus_points: device.bonus_points || 0,
      remaining_points: remaining,
      status: device.status,
      recent_logs: recentLogsQuery.results || []
    }), { status: 200, headers: CORS_HEADERS });

  } catch (err) {
    return new Response(JSON.stringify({ success: false, error: err.message }), { status: 500, headers: CORS_HEADERS });
  }
}

async function handleDeductQuota(request, env) {
  try {
    const bodyBytes = new Uint8Array(await request.arrayBuffer());
    const bodyText = new TextDecoder().decode(bodyBytes);
    let payload = {};
    try {
      payload = JSON.parse(bodyText);
    } catch {
      return new Response(JSON.stringify({ success: false, error: "Invalid JSON" }), { status: 400, headers: CORS_HEADERS });
    }

    const { device_fingerprint, points_needed = 1, tool_name = "unknown" } = payload;
    if (!device_fingerprint) {
      return new Response(JSON.stringify({ success: false, error: "Missing device_fingerprint" }), { status: 400, headers: CORS_HEADERS });
    }

    const timestampStr = request.headers.get("X-Timestamp");
    const signatureHex = request.headers.get("X-Signature");
    const secret = env.RELAY_SECRET || "ai-forge-commercial-secret-2026";

    if (!timestampStr || !signatureHex) {
      return new Response(JSON.stringify({ success: false, error: "Missing HMAC headers" }), { status: 401, headers: CORS_HEADERS });
    }

    const nowSec = Math.floor(Date.now() / 1000);
    const reqSec = parseInt(timestampStr, 10);
    if (isNaN(reqSec) || Math.abs(nowSec - reqSec) > 300) {
      return new Response(JSON.stringify({ success: false, error: "Timestamp expired" }), { status: 403, headers: CORS_HEADERS });
    }

    const isValid = await verifyHmac(secret, timestampStr, bodyBytes, signatureHex);
    if (!isValid) {
      return new Response(JSON.stringify({ success: false, error: "Invalid signature" }), { status: 401, headers: CORS_HEADERS });
    }

    const todayUtc = new Date().toISOString().split("T")[0];
    let device = await env.DB.prepare("SELECT * FROM devices WHERE device_fingerprint = ?").bind(device_fingerprint).first();

    if (!device) {
      await env.DB.prepare(
        "INSERT INTO devices (device_fingerprint, daily_limit, used_points_today, last_reset_utc_date) VALUES (?, ?, 0, ?)"
      ).bind(device_fingerprint, DAILY_FREE_POINTS, todayUtc).run();

      device = {
        device_fingerprint,
        daily_limit: DAILY_FREE_POINTS,
        used_points_today: 0,
        bonus_points: 0,
        last_reset_utc_date: todayUtc,
        status: "active"
      };
    }

    if (device.status === "banned") {
      return new Response(JSON.stringify({ success: false, error: "Device is banned" }), { status: 403, headers: CORS_HEADERS });
    }

    let currentUsed = device.used_points_today;
    if (device.last_reset_utc_date !== todayUtc) {
      currentUsed = 0;
      await env.DB.prepare(
        "UPDATE devices SET used_points_today = 0, last_reset_utc_date = ?, updated_at = CURRENT_TIMESTAMP WHERE device_fingerprint = ?"
      ).bind(todayUtc, device_fingerprint).run();
    }

    const totalAvailable = (device.daily_limit - currentUsed) + (device.bonus_points || 0);
    if (totalAvailable < points_needed) {
      return new Response(JSON.stringify({
        success: false,
        error: "今日免费算力额度已用完，明日 00:00 自动重置",
        remaining_points: Math.max(0, totalAvailable)
      }), { status: 402, headers: CORS_HEADERS });
    }

    const newUsed = currentUsed + points_needed;
    await env.DB.prepare(
      "UPDATE devices SET used_points_today = ?, updated_at = CURRENT_TIMESTAMP WHERE device_fingerprint = ?"
    ).bind(newUsed, device_fingerprint).run();

    const clientIp = request.headers.get("CF-Connecting-IP") || "unknown";
    await env.DB.prepare(
      "INSERT INTO quota_logs (device_fingerprint, tool_name, points_deducted, client_ip) VALUES (?, ?, ?, ?)"
    ).bind(device_fingerprint, tool_name, points_needed, clientIp).run();

    return new Response(JSON.stringify({
      success: true,
      device_fingerprint,
      points_deducted: points_needed,
      remaining_points: totalAvailable - points_needed
    }), { status: 200, headers: CORS_HEADERS });

  } catch (err) {
    return new Response(JSON.stringify({ success: false, error: err.message }), { status: 500, headers: CORS_HEADERS });
  }
}

async function handleTelemetryReport(request, env) {
  try {
    const payload = await request.json();
    const { device_fingerprint, tool_name, elapsed_ms = 0, success = true, error_message = null, client_version = "1.0.0" } = payload;

    const clientIp = request.headers.get("CF-Connecting-IP") || "unknown";
    await env.DB.prepare(
      "INSERT INTO telemetry_events (device_fingerprint, tool_name, elapsed_ms, success, error_message, client_version, client_ip) VALUES (?, ?, ?, ?, ?, ?, ?)"
    ).bind(device_fingerprint || "anonymous", tool_name || "unknown", elapsed_ms, success ? 1 : 0, error_message, client_version, clientIp).run();

    return new Response(JSON.stringify({ success: true }), { status: 200, headers: CORS_HEADERS });
  } catch (e) {
    return new Response(JSON.stringify({ success: false, error: e.message }), { status: 200, headers: CORS_HEADERS });
  }
}

async function verifyHmac(secret, timestampStr, bodyBytes, expectedHex) {
  try {
    const encoder = new TextEncoder();
    const secretBytes = encoder.encode(secret);

    const timestampNum = BigInt(timestampStr);
    const timestampBytes = new Uint8Array(8);
    let tempVal = timestampNum;
    for (let i = 7; i >= 0; i--) {
      timestampBytes[i] = Number(tempVal & 0xffn);
      tempVal >>= 8n;
    }

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
  } catch {
    return false;
  }
}
