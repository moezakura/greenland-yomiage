//! 環境変数からのアプリ設定ロード。

use crate::error::ConfigError;

/// 環境変数から読み込むアプリ設定。
#[derive(Debug, Clone)]
pub struct Config {
    /// Discord Bot トークン。
    pub discord_token: String,
    /// スラッシュコマンドを登録する対象ギルド ID。
    pub guild_id: u64,
    /// 読み上げ対象テキストチャンネルの初期値（`/join` 実行時に上書きされる）。
    pub yomiage_channel_id: u64,
    /// VOICEVOX Engine のベース URL。
    pub voicevox_base_url: String,
    /// AIVoice2 Engine のベース URL。
    pub aivoice_base_url: String,
    /// ユーザー音声設定 JSON ファイルのパス。
    pub voice_settings_path: String,
    /// 「空気読み」系の挙動設定。
    pub behavior: BehaviorConfig,
}

/// 「空気読み」系機能のオン/オフと調整値。
#[derive(Debug, Clone)]
pub struct BehaviorConfig {
    /// F1: VC で誰かが喋っている間は読み上げを保留する。
    pub wait_while_speaking: bool,
    /// F2: 同一ユーザーの連投を 1 発話へ結合する。
    pub merge_bursts: bool,
    /// F3: Bot と同じ VC にいないユーザーのメッセージを読み上げない。
    pub skip_non_vc: bool,
    /// F1: 最後の発話からこの時間が経過したら「沈黙」とみなす。
    pub quiet_threshold: std::time::Duration,
}

impl BehaviorConfig {
    /// 環境変数から挙動設定を読み込む。未設定時はすべて有効・閾値 500ms。
    fn from_env() -> Self {
        Self {
            wait_while_speaking: optional_bool("YOMIAGE_WAIT_WHILE_SPEAKING", true),
            merge_bursts: optional_bool("YOMIAGE_MERGE_BURSTS", true),
            skip_non_vc: optional_bool("YOMIAGE_SKIP_NON_VC", true),
            quiet_threshold: std::time::Duration::from_millis(optional_parse(
                "YOMIAGE_QUIET_THRESHOLD_MS",
                500,
            )),
        }
    }
}

impl Config {
    /// 環境変数からアプリ設定を読み込む。
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            discord_token: required("DISCORD_TOKEN")?,
            guild_id: required_parse("DISCORD_GUILD_ID")?,
            yomiage_channel_id: required_parse("DISCORD_YOMIAGE_CH_ID")?,
            voicevox_base_url: trim_url(optional(
                "VOICEVOX_BASE_URL",
                "http://localhost:50021",
            )),
            aivoice_base_url: trim_url(optional(
                "AIVOICE2_ENGINE_BASE_URL",
                "http://localhost:8000",
            )),
            voice_settings_path: optional("VOICE_SETTINGS_PATH", "data/voice_settings.json"),
            behavior: BehaviorConfig::from_env(),
        })
    }
}

/// 必須の環境変数を取得する。
fn required(name: &'static str) -> Result<String, ConfigError> {
    match std::env::var(name) {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => Err(ConfigError::Missing(name)),
    }
}

/// 任意の環境変数を取得する。未設定なら既定値を返す。
fn optional(name: &str, default: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_owned())
}

/// 必須の環境変数を取得し、目的の型へパースする。
fn required_parse<T: std::str::FromStr>(name: &'static str) -> Result<T, ConfigError> {
    let value = required(name)?;
    value.parse().map_err(|_| ConfigError::Invalid { name, value })
}

/// ベース URL の末尾スラッシュを取り除く。
fn trim_url(url: String) -> String {
    url.trim_end_matches('/').to_owned()
}

/// 任意の真偽値環境変数を取得する。`true`/`false`/`1`/`0`（大文字小文字無視）を受理。
/// 未設定・解釈不能なら既定値を返す。
fn optional_bool(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

/// 任意の数値環境変数を取得する。未設定・解釈不能なら既定値を返す。
fn optional_parse<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}
