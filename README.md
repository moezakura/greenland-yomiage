# greenland-yomiage

> **Note**: GitHub上のリポジトリはミラーです。開発のメインリポジトリは http://10.77.0.20/greenland/yomiage.git にあります。

Discord用テキスト読み上げBot。テキストチャンネルに投稿されたメッセージを音声合成し、ボイスチャンネルで読み上げます。

## 主な機能

- 📢 テキストメッセージの自動音声読み上げ（投稿順を保証）
- 🔗 URL・コードブロック・絵文字・メンションの前処理（読み上げスキップ／置換）
- 📚 辞書機能による単語登録
- 🔄 複数の TTS エンジンに対応（VOICEVOX / AIVoice2）
- 👤 ユーザーごとの音声設定（JSON ファイルに永続化）
- ⚡ スラッシュコマンド
  - `/join` — ボイスチャンネルに参加
  - `/leave` — ボイスチャンネルから退出
  - `/set-voice` — 使用する音声を選択（引数なしで選択 UI、`voicevox:8` 等で直接指定）
  - `/add-word` — 辞書に単語を追加
  - `/cancel` — 読み上げをキャンセル

## 技術スタック

- **言語**: Rust（edition 2024）
- **Discord API**: [serenity](https://github.com/serenity-rs/serenity)
- **音声**: [songbird](https://github.com/serenity-rs/songbird) v0.6（DAVE プロトコル対応・Opus エンコード・再生キュー）
- **非同期ランタイム**: [tokio](https://tokio.rs/)
- **構造化ログ**: [tracing](https://github.com/tokio-rs/tracing)
- **音声合成**: VOICEVOX Engine / AIVoice2 Engine（切り替え可能）

## アーキテクチャ

依存性逆転と interface 分離を重視したレイヤ構成（依存方向は一方向）:

```
src/
├── domain/         trait と値オブジェクト（外部ライブラリ非依存）
│   ├── tts.rs          TtsEngine / SpeakerDirectory / DictionaryWriter
│   ├── text_rule.rs    TextRule（読み上げスキップ／置換ルールの抽象）
│   └── voice_store.rs  VoiceSettingsStore
├── application/    ユースケース（domain にのみ依存）
│   ├── engine_registry.rs  複数 TTS エンジンの登録・選択
│   ├── rule_pipeline.rs    前処理ルールの連結適用
│   └── rules.rs            標準ルール群
├── infrastructure/ 具象実装（Discord / TTS エンジン / 永続化）
└── bootstrap.rs    Composition Root（依存の組み立て）
```

音声合成エンジンと読み上げルールはそれぞれ trait で抽象化されており、新しい実装を
1 ファイル追加して `bootstrap.rs` に登録するだけで増やせる。依存方向は各レイヤの
モジュール冒頭に記した DEPENDENCY RULE コメントで明示している。

## セットアップ

### 環境変数

`.env.example` をコピーして `.env` を作成し、値を設定する。

```bash
cp .env.example .env
```

| 環境変数 | 必須 | 説明 |
|---|---|---|
| `DISCORD_TOKEN` | ✓ | Discord Bot トークン |
| `DISCORD_GUILD_ID` | ✓ | 対象ギルド ID |
| `DISCORD_YOMIAGE_CH_ID` | ✓ | 読み上げ対象チャンネルの初期値 |
| `VOICEVOX_BASE_URL` | | VOICEVOX Engine の URL（既定: `http://localhost:50021`） |
| `AIVOICE2_ENGINE_BASE_URL` | | AIVoice2 Engine の URL（既定: `http://localhost:8000`） |
| `VOICE_SETTINGS_PATH` | | 音声設定 JSON のパス（既定: `data/voice_settings.json`） |
| `RUST_LOG` | | ログレベル（既定: `info`） |
| `LOG_FORMAT` | | `json` で構造化 JSON 出力 |

### Docker Compose で起動

```bash
docker compose up -d
```

### ローカル開発

```bash
# VOICEVOX Engine を起動
docker run -d -p 50021:50021 voicevox/voicevox_engine:cpu-ubuntu20.04-latest

# ビルド・テスト・実行
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```

## 謝辞

- [VOICEVOX](https://voicevox.hiroshiba.jp/) — 無料で使える中品質なテキスト読み上げソフトウェア
- [serenity](https://github.com/serenity-rs/serenity) / [songbird](https://github.com/serenity-rs/songbird)
