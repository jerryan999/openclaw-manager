use log::{info, warn, error};
use std::path::{Path, PathBuf};
use tauri::Manager;

/// 获取打包资源的路径
/// 
/// 在开发模式下，资源位于 `src-tauri/resources/`
/// 在生产模式下，资源被打包到应用程序包中
pub fn get_resource_path(app_handle: &tauri::AppHandle, resource_name: &str) -> Option<PathBuf> {
    // 尝试从打包资源中获取
    if let Ok(resource_path) = app_handle.path().resource_dir() {
        info!("[资源] 资源根目录: {:?}", resource_path);
        let full_path = resource_path.join(resource_name);
        info!("[资源] 检查路径: {:?}, exists={}", full_path, full_path.exists());
        if full_path.exists() {
            info!("[资源] ✓ 找到打包资源: {:?}", full_path);
            return Some(full_path);
        }
        
        // 列出资源目录的内容，帮助调试
        if let Ok(entries) = std::fs::read_dir(&resource_path) {
            info!("[资源] 资源目录内容:");
            for entry in entries.flatten() {
                info!("[资源]   - {:?}", entry.file_name());
            }
        }
    } else {
        warn!("[资源] 无法获取资源目录");
    }
    
    // 开发模式：从 src-tauri/resources/ 读取
    if cfg!(debug_assertions) {
        let dev_path = PathBuf::from("src-tauri/resources").join(resource_name);
        if dev_path.exists() {
            info!("[资源] 找到开发资源: {:?}", dev_path);
            return Some(dev_path);
        }
    }
    
    warn!("[资源] ✗ 未找到资源: {}", resource_name);
    None
}

/// 检查是否有打包的 Node.js
pub fn has_bundled_nodejs(app_handle: &tauri::AppHandle) -> bool {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    let resource_name = match (os, arch) {
        ("macos", "aarch64") => "nodejs/node-macos-arm64.tar.gz",
        ("macos", "x86_64") => "nodejs/node-macos-x64.tar.gz",
        ("windows", "x86_64") => "nodejs/node-windows-x64.zip",
        ("linux", "x86_64") => "nodejs/node-linux-x64.tar.gz",
        _ => {
            warn!("[资源] 不支持的平台: {}-{}", os, arch);
            return false;
        }
    };
    
    get_resource_path(app_handle, resource_name).is_some()
}

/// 检查是否有打包的 OpenClaw
pub fn has_bundled_openclaw(app_handle: &tauri::AppHandle) -> bool {
    // 检查是否有打包的 npm 包
    if let Some(path) = get_resource_path(app_handle, "openclaw") {
        // 检查目录中是否有 .tgz 文件
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_str = file_name.to_string_lossy();
                if file_str.ends_with(".tgz") || file_str.ends_with(".tar.gz") {
                    info!("[资源] 找到打包的 OpenClaw: {:?}", entry.path());
                    return true;
                }
            }
        }
    }
    false
}

/// 提取打包的 Node.js 到系统目录
pub async fn extract_bundled_nodejs(
    app_handle: &tauri::AppHandle,
    target_dir: &Path,
) -> Result<PathBuf, String> {
    info!("[提取] 开始提取 Node.js 到: {:?}", target_dir);
    
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    let resource_name = match (os, arch) {
        ("macos", "aarch64") => "nodejs/node-macos-arm64.tar.gz",
        ("macos", "x86_64") => "nodejs/node-macos-x64.tar.gz",
        ("windows", "x86_64") => "nodejs/node-windows-x64.zip",
        ("linux", "x86_64") => "nodejs/node-linux-x64.tar.gz",
        _ => return Err(format!("不支持的平台: {}-{}", os, arch)),
    };
    
    let resource_path = get_resource_path(app_handle, resource_name)
        .ok_or_else(|| "未找到打包的 Node.js".to_string())?;
    
    // 创建目标目录
    std::fs::create_dir_all(target_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;
    
    // 根据平台解压
    if os == "windows" {
        extract_zip(&resource_path, target_dir)?;
    } else {
        extract_tar_gz(&resource_path, target_dir)?;
    }
    
    // 返回 node 二进制文件的路径
    let node_bin = if os == "windows" {
        target_dir.join("node.exe")
    } else {
        target_dir.join("bin").join("node")
    };
    
    if !node_bin.exists() {
        return Err(format!("提取后未找到 node 二进制: {:?}", node_bin));
    }
    
    info!("[提取] Node.js 提取成功: {:?}", node_bin);
    Ok(node_bin)
}

/// 获取打包的 OpenClaw npm 包路径
pub fn get_bundled_openclaw_package(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
    let openclaw_dir = get_resource_path(app_handle, "openclaw")?;
    
    // 查找 .tgz 文件
    if let Ok(entries) = std::fs::read_dir(&openclaw_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name()?.to_string_lossy();
            if file_name.ends_with(".tgz") || file_name.ends_with(".tar.gz") {
                info!("[资源] 找到 OpenClaw 包: {:?}", path);
                return Some(path);
            }
        }
    }
    
    None
}

/// 解压 tar.gz 文件
fn extract_tar_gz(archive_path: &Path, target_dir: &Path) -> Result<(), String> {
    info!("[解压] 解压 tar.gz: {:?} -> {:?}", archive_path, target_dir);
    
    use std::process::Command;
    
    let output = Command::new("tar")
        .args([
            "-xzf",
            &archive_path.to_string_lossy(),
            "-C",
            &target_dir.to_string_lossy(),
            "--strip-components=1",
        ])
        .output()
        .map_err(|e| format!("执行 tar 命令失败: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("[解压] tar 命令失败: {}", stderr);
        return Err(format!("解压失败: {}", stderr));
    }
    
    info!("[解压] tar.gz 解压成功");
    Ok(())
}

/// 解压 zip 文件
fn extract_zip(archive_path: &Path, target_dir: &Path) -> Result<(), String> {
    info!("[解压] 解压 zip: {:?} -> {:?}", archive_path, target_dir);
    
    use std::process::Command;
    
    // Windows: 使用 PowerShell 的 Expand-Archive
    let script = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        archive_path.display(),
        target_dir.display()
    );
    
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| format!("执行 PowerShell 命令失败: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("[解压] PowerShell 命令失败: {}", stderr);
        return Err(format!("解压失败: {}", stderr));
    }
    
    // 移动文件（Expand-Archive 会创建子目录）
    let extracted_dir = target_dir.join(
        archive_path
            .file_stem()
            .ok_or("无效的文件名")?
    );
    
    if extracted_dir.exists() {
        // 将内容移动到 target_dir
        if let Ok(entries) = std::fs::read_dir(&extracted_dir) {
            for entry in entries.flatten() {
                let src = entry.path();
                let file_name = entry.file_name();
                let dst = target_dir.join(&file_name);
                std::fs::rename(&src, &dst)
                    .map_err(|e| format!("移动文件失败: {} -> {}: {}", src.display(), dst.display(), e))?;
            }
        }
        // 删除临时目录
        let _ = std::fs::remove_dir_all(&extracted_dir);
    }
    
    info!("[解压] zip 解压成功");
    Ok(())
}
