import sys
import os
import struct
import json
import traceback

# ==========================================
# 🛡️ 工业级防线：OS 级别文件描述符劫持 (FD Hijacking)
# ==========================================
def hijack_stdout():
    try:
        # 1. 备份真实的 stdout (FD 1) 到一个新的文件描述符
        real_stdout_fd = os.dup(1)
        # 2. 将 stdout (FD 1) 强行重定向到 stderr (FD 2)
        os.dup2(2, 1)
        # 3. 用备份的真实 stdout 创建一个纯净的二进制写入流
        ipc_out = os.fdopen(real_stdout_fd, 'wb')
        return ipc_out
    except Exception as e:
        print(f"FD 劫持失败: {e}", file=sys.stderr)
        sys.exit(1)

# 初始化纯净通信通道
IPC_OUT = hijack_stdout()
IPC_IN = sys.stdin.buffer

def send_message(method: str, params: dict):
    """发送 4 字节大端序长度 + JSON 载荷"""
    msg = {"method": method, "params": params}
    data = json.dumps(msg, ensure_ascii=False).encode('utf-8')
    # 写入 4 字节大端序长度头 (>I 表示大端序无符号整数)
    IPC_OUT.write(struct.pack('>I', len(data)))
    # 写入 JSON 载荷
    IPC_OUT.write(data)
    IPC_OUT.flush()

def recv_message():
    """读取 4 字节大端序长度，再读取 JSON 载荷"""
    header = IPC_IN.read(4)
    if not header or len(header) < 4:
        return None  # 管道已断开 (Rust 宿主退出)
    
    msg_len = struct.unpack('>I', header)[0]
    data = IPC_IN.read(msg_len)
    
    if len(data) < msg_len:
        return None
        
    return json.loads(data.decode('utf-8'))

def main():
    send_message("system.ready", {"status": "Worker 已点火，FD 劫持成功"})
    
    while True:
        try:
            msg = recv_message()
            if msg is None:
                print("Rust 宿主已断开，Worker 优雅退出...", file=sys.stderr)
                break
                
            method = msg.get("method")
            params = msg.get("params", {})
            
            if method == "ping":
                send_message("pong", {"echo": params})
            elif method == "exit":
                break
            else:
                send_message("error", {"message": f"未知指令: {method}"})
                
        except Exception as e:
            err_msg = f"Worker 异常: {str(e)}\n{traceback.format_exc()}"
            print(err_msg, file=sys.stderr)
            send_message("error", {"message": str(e)})

if __name__ == "__main__":
    main()
