use crate::models::ServiceStatus;
use crate::utils::shell;
use log::{debug, info};
use std::process::Command;
use tauri::command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows CREATE_NO_WINDOW 标志，用于隐藏控制台窗口
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const SERVICE_PORT: u16 = 18789;

/// 检测端口是否有服务在监听，返回 PID
/// 简单直接：端口被占用 = 服务运行中
fn check_port_listening(port: u16) -> Option<u32> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output()
            .ok()?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .and_then(|line| line.trim().parse::<u32>().ok())
        } else {
            None
        }
    }

    #[cfg(windows)]
    {
        let mut cmd = Command::new("netstat");
        cmd.args(["-ano"]);
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output().ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                    if let Some(pid_str) = line.split_whitespace().last() {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            return Some(pid);
                        }
                    }
                }
            }
        }
        None
    }
}

/// 获取服务状态（简单版：直接检查端口占用）
#[command]
pub async fn get_service_status() -> Result<ServiceStatus, String> {
    // 简单直接：检查端口是否被占用
    let pid = check_port_listening(SERVICE_PORT);
    let running = pid.is_some();

    Ok(ServiceStatus {
        running,
        pid,
        port: SERVICE_PORT,
        uptime_seconds: None,
        memory_mb: None,
        cpu_percent: None,
    })
}

/// 启动服务
#[command]
pub async fn start_service() -> Result<String, String> {
    info!("[服务] 启动服务...");

    // 检查是否已经运行
    let status = get_service_status().await?;
    if status.running {
        info!("[服务] 服务已在运行中");
        return Err("服务已在运行中".to_string());
    }

    // 检查 openclaw 命令是否存在
    let openclaw_path = shell::get_openclaw_path();
    if openclaw_path.is_none() {
        info!("[服务] 找不到 openclaw 命令");
        return Err(
            "找不到 openclaw 命令，请先通过 npm install -g @jerryan999/openclaw-zh 安装"
                .to_string(),
        );
    }
    info!("[服务] openclaw 路径: {:?}", openclaw_path);

    // 直接后台启动 gateway（不等待 doctor，避免阻塞）
    info!("[服务] 后台启动 gateway...");
    shell::spawn_openclaw_gateway().map_err(|e| format!("启动服务失败: {}", e))?;

    // 轮询等待端口开始监听（最多 15 秒）
    info!("[服务] 等待端口 {} 开始监听...", SERVICE_PORT);
    for i in 1..=15 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Some(pid) = check_port_listening(SERVICE_PORT) {
            info!("[服务] ✓ 启动成功 ({}秒), PID: {}", i, pid);
            return Ok(format!("服务已启动，PID: {}", pid));
        }
        if i % 3 == 0 {
            debug!("[服务] 等待中... ({}秒)", i);
        }
    }

    info!("[服务] 等待超时，端口仍未监听");
    Err("服务启动超时（15秒），请检查 openclaw 日志".to_string())
}

/// 停止服务
#[command]
pub async fn stop_service() -> Result<String, String> {
    info!("[服务] 停止服务...");

    let _ = shell::run_openclaw(&["gateway", "stop"]);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let status = get_service_status().await?;
    if !status.running {
        info!("[服务] ✓ 已停止");
        return Ok("服务已停止".to_string());
    }

    // 尝试强制停止
    let _ = shell::run_openclaw(&["gateway", "stop", "--force"]);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let status = get_service_status().await?;
    if status.running {
        Err(format!("无法停止服务，PID: {:?}", status.pid))
    } else {
        info!("[服务] ✓ 已停止");
        Ok("服务已停止".to_string())
    }
}

/// 重启服务
#[command]
pub async fn restart_service() -> Result<String, String> {
    info!("[服务] 重启服务...");

    let _ = shell::run_openclaw(&["gateway", "restart"]);
    std::thread::sleep(std::time::Duration::from_secs(2));

    let status = get_service_status().await?;
    if status.running {
        info!("[服务] ✓ 重启成功, PID: {:?}", status.pid);
        Ok(format!("服务已重启，PID: {:?}", status.pid))
    } else {
        // 手动停止再启动
        let _ = stop_service().await;
        std::thread::sleep(std::time::Duration::from_secs(1));
        start_service().await
    }
}

/// 获取日志
#[command]
pub async fn get_logs(lines: Option<u32>) -> Result<Vec<String>, String> {
    let n = lines.unwrap_or(100);

    match shell::run_openclaw(&["logs", "--lines", &n.to_string()]) {
        Ok(output) => Ok(output.lines().map(|s| s.to_string()).collect()),
        Err(e) => Err(format!("读取日志失败: {}", e)),
    }
}
