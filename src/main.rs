// supipi
// SUPERのダブルタップを検出してwofiランチャーを起動する
// evdevを使用してキーボードイベントを監視してます
// エラーハンドリングはctrlcをつかう。windows対応は今のところ必要なさそうだし

use evdev::{Device, EventType, KeyCode};
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use ctrlc;
use thiserror::Error;

/// アプリケーション固有のエラー型
#[derive(Error, Debug)]
enum SupipiError {
    /// I/O操作におけるエラー
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// シグナルハンドリングにおけるエラー
    #[error("Signal error: {0}")]
    Signal(#[from] ctrlc::Error),
    /// 対象キーボードが見つからない場合のエラー
    #[error("No suitable KB USB KB keyboard found")]
    NoKeyboard,
}

/// 指定した名前パターンを持つUSBキーボードを検索する関数
/// 
/// /dev/input/event0〜31 のデバイスを検索し、完全一致の "KB USB KB" を持つ、
/// または "KB USB KB" を含む最も若い番号のデバイスを返します（作者環境）
fn find_usb_keyboard() -> Result<String, SupipiError> {
    // 完全一致のデバイスと最若番号の "KB USB KB" を含むデバイスを追跡
    let mut exact_match: Option<(String, String)> = None;
    let mut first_match: Option<(String, String)> = None;
    
    // 最大32個のイベントデバイスをチェック
    for i in 0..32 {
        let path = format!("/dev/input/event{}", i);
        if Path::new(&path).exists() {
            if let Ok(device) = Device::open(&path) {
                if let Some(name) = device.name() {
                    // 完全一致するデバイスを検出
                    if name == "KB USB KB" {
                        println!("Found exact match keyboard: {} at {}", name, path);
                        exact_match = Some((name.to_string(), path.clone()));
                        break; // 完全一致が見つかった場合はループを終了
                    }
                    // 部分一致するデバイスを検出（最初に見つかったもののみ保存）
                    else if name.contains("KB USB KB") && first_match.is_none() {
                        first_match = Some((name.to_string(), path.clone()));
                    }
                }
            }
        }
    }
    
    // 完全一致 → 部分一致の順で優先
    if let Some((name, path)) = exact_match {
        println!("Using exact match keyboard: {} at {}", name, path);
        return Ok(path);
    } else if let Some((name, path)) = first_match {
        println!("Using first found keyboard: {} at {}", name, path);
        return Ok(path);
    }
    
    // デバイスが見つからない場合はエラーを返す
    Err(SupipiError::NoKeyboard)
}

fn main() -> Result<(), SupipiError> {
    // 終了シグナル処理用のフラグ
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // SIGINT(Ctrl+C)とSIGTERMをハンドリング
    ctrlc::set_handler(move || {
        println!("Received SIGINT or SIGTERM");
        r.store(false, Ordering::SeqCst);
    })?;

    // 対象キーボードを検出
    let device_path = find_usb_keyboard()?;
    let mut device = Device::open(&device_path)?;
    println!("Using device: {:?}", device.name());

    // ダブルタップ検出用の設定
    let timeout = Duration::from_millis(240); // ダブルタップの判定時間閾値
    let mut last_press = Instant::now();      // 最後にキーが押された時間
    let mut tap_count = 0;                    // タップ回数カウンター

    println!("Supipi started, listening on {}...", device_path);

    // メインループ
    while running.load(Ordering::SeqCst) {
        match device.fetch_events() {
            Ok(events) => {
                for ev in events {
                    // 左SUPERキーの押下イベントを検出
                    if ev.event_type() == EventType::KEY && ev.code() == KeyCode::KEY_LEFTMETA.0 {
                        if ev.value() == 1 { // キー押下時の値
                            let now = Instant::now();
                            // タイムアウト時間内に連続で押されたかチェック
                            if now.duration_since(last_press) < timeout {
                                tap_count += 1;
                                // ダブルタップを検出した場合
                                if tap_count == 2 {
                                    // wofiランチャーを起動
                                    match Command::new("wofi").args(["--show", "drun"]).spawn() {
                                        Ok(_) => println!("Launched wofi"),
                                        Err(e) => eprintln!("Failed to launch wofi: {}", e),
                                    }
                                    tap_count = 0; // カウンターをリセット
                                }
                            } else {
                                // タイムアウト後の押下は新しいシーケンスの開始
                                tap_count = 1;
                            }
                            last_press = now; // 最終押下時間を更新
                        }
                    }
                }
            }
            Err(e) => {
                // イベント取得失敗時は少し待ってリトライ
                eprintln!("Event fetch error: {}", e);
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    println!("Closing device...");
    Ok(())
}