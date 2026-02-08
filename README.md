# greenland-yomiage

> **Note**: GitHub上のリポジトリはミラーです。開発のメインリポジトリは http://10.77.0.20/greenland/yomiage.git にあります。

Discord用テキスト読み上げBot - VOICEVOXを使用してテキストメッセージを音声に変換し、ボイスチャンネルで読み上げます。

## 概要

greenland-yomiageは、Discordサーバーのテキストチャンネルに投稿されたメッセージを、VOICEVOXエンジンを使用して音声合成し、ボイスチャンネルで読み上げるBotです。

## 主な機能

- 📢 テキストメッセージの自動音声読み上げ
- 🔗 URLの自動省略（「URL省略」と読み上げ）
- 📝 コードブロックの自動省略（「こんなの読めないのだ」と読み上げ）
- 📚 辞書機能による単語登録
- 🔄 複数のTTSエンジンに対応（VOICEVOX / AIVoice2）
- 👤 ユーザーごとの音声設定（エンジンは音声選択時に自動設定）
- ⚡ スラッシュコマンド対応
  - `/join` - ボイスチャンネルに参加
  - `/leave` - ボイスチャンネルから退出
  - `/set-voice` - 使用する音声を選択（VOICEVOX/AIVoice両方から選択可能）
  - `/add-word` - 辞書に単語を追加
  - `/cancel` - 現在の読み上げをキャンセル

## 技術スタック

- **言語**: Go 1.20
- **Discord API**: [discordgo](https://github.com/bwmarrin/discordgo) v0.27.1
- **音声合成**: VOICEVOX Engine / AIVoice2 Engine (切り替え可能)
- **音声エンコード**: Opus (gopus)
- **コンテナ**: Docker / Docker Compose

## セットアップ

### 前提条件

- Docker および Docker Compose がインストールされていること
- Discord Bot トークンを取得済みであること
- Discord サーバーの管理権限を持っていること

### 環境変数

以下の環境変数を`.env`ファイルに設定してください：

```env
DISCORD_TOKEN=your_bot_token_here
DISCORD_GUILD_ID=your_guild_id_here
DISCORD_YOMIAGE_CH_ID=your_text_channel_id_here
TTS_ENGINE=voicevox
VOICEVOX_BASE_URL=http://localhost:50021
AIVOICE2_ENGINE_BASE_URL=http://localhost:8000
```

- `DISCORD_TOKEN`: Discord Bot のトークン
- `DISCORD_GUILD_ID`: 対象のDiscordサーバー（ギルド）のID
- `DISCORD_YOMIAGE_CH_ID`: 読み上げ対象のテキストチャンネルのID
- `TTS_ENGINE`: 使用するTTSエンジン（`voicevox` または `aivoice`、デフォルト: `voicevox`）
- `VOICEVOX_BASE_URL`: VOICEVOX Engine のURL
- `AIVOICE2_ENGINE_BASE_URL`: AIVoice2 Engine のURL

### 起動方法

1. リポジトリをクローン
```bash
git clone http://10.77.0.20/greenland/yomiage.git greenland-yomiage
cd greenland-yomiage
```

2. 環境変数ファイルを作成
```bash
cp .env.example .env
# .envファイルを編集して必要な値を設定
```

3. Docker Composeで起動
```bash
docker-compose up -d
```

## アーキテクチャ

### サービス構成

- **bot**: メインのDiscord Botアプリケーション（Go）
- **voicevox**: VOICEVOX Engine（音声合成エンジン）
- **aivoice** (オプション): AIVoice2 Engine（音声合成エンジン）

### ディレクトリ構造

```
greenland-yomiage/
├── app/
│   ├── general/
│   │   ├── cmd/
│   │   │   └── main.go         # エントリーポイント
│   │   └── internal/
│   │       ├── config/          # 設定管理
│   │       ├── handler/         # Discordイベントハンドラー
│   │       ├── initialize/      # 初期化処理
│   │       ├── listener/        # 音声リスナー
│   │       ├── props/           # プロパティ
│   │       └── speaker/         # 音声スピーカー
│   └── internal/
│       ├── dictionary/          # 辞書機能
│       ├── opus/                # Opusエンコード
│       ├── usecase/             # ユースケース層
│       ├── voicevox/            # VOICEVOX連携
│       ├── aivoice/             # AIVoice2連携
│       └── wavgenerator/        # WAV生成インターフェース
├── docker-compose.yml           # Docker Compose設定
├── Dockerfile                   # Botコンテナ定義
└── compose.yml                  # 簡易版Compose設定
```

## 開発

### ローカル開発環境

1. Go 1.20以上をインストール
2. 依存関係をインストール
```bash
cd app
go mod download
```

3. ビルド
```bash
go build -o bot general/cmd/main.go
```

4. TTSエンジンを起動

**VOICEVOX Engineを使用する場合:**
```bash
docker run -d -p 50021:50021 voicevox/voicevox_engine:cpu-ubuntu20.04-latest
```

**AIVoice2 Engineを使用する場合:**
```bash
# AIVoice2 Engineを起動（別途セットアップが必要）
# ポート8000でAPIサーバーを起動してください
```

5. 環境変数を設定して実行

```bash
export DISCORD_TOKEN=your_token
export DISCORD_GUILD_ID=your_guild_id
export DISCORD_YOMIAGE_CH_ID=your_channel_id
export VOICEVOX_BASE_URL=http://localhost:50021
export AIVOICE2_ENGINE_BASE_URL=http://localhost:8000  # AIVoice使用時のみ
./bot
```

## TTSエンジンの使い分け

### 音声の選択

`/set-voice` コマンドを使用すると、VOICEVOXとAIVoice2の両方のスピーカーが一覧で表示されます。
選択した音声に応じて、使用するエンジンが自動的に設定されます。

- `[VOICEVOX]` プレフィックス付きの音声を選択 → VOICEVOX Engineが使用されます
- `[AIVoice]` プレフィックス付きの音声を選択 → AIVoice2 Engineが使用されます

### AIVoice2 Engineのセットアップ

AIVoice2の音声を使用する場合は、別途AIVoice2 Engine APIサーバーをセットアップする必要があります。

1. AIVoice2 Engine APIサーバーを起動（ポート8000）
2. 環境変数 `AIVOICE2_ENGINE_BASE_URL` を設定
3. `/set-voice` コマンドで `[AIVoice]` プレフィックス付きの音声を選択

**APIエンドポイント:**
```
POST {AIVOICE2_ENGINE_BASE_URL}/synthesize
Content-Type: application/json

{
  "text": "こんにちは",
  "speaker": "akane",
  "style": "平静"
}
```

## トラブルシューティング

### Botがボイスチャンネルに参加できない
- Botに適切な権限（Voice Connect, Voice Speak）が付与されているか確認してください
- ギルドIDとチャンネルIDが正しく設定されているか確認してください

### 読み上げが動作しない
- 使用しているTTSエンジンが正常に起動しているか確認してください
- VOICEVOX使用時: `docker-compose logs voicevox` でログを確認
- AIVoice使用時: AIVoice2 Engine APIサーバーが起動しているか確認
- 環境変数 `DISCORD_YOMIAGE_CH_ID` で指定したチャンネルにメッセージを投稿しているか確認してください

### メモリ不足エラー
- VOICEVOX Engineのスレッド数を調整してください（docker-compose.yml内の `VOICEVOX_CPU_NUM_THREADS`）

### AIVoiceで音声が生成されない
- `AIVOICE2_ENGINE_BASE_URL` が正しく設定されているか確認してください
- AIVoice2 Engine APIサーバーが起動しているか確認してください
- `/set-voice` で `[AIVoice]` プレフィックス付きの音声を選択しているか確認してください

## ライセンス

このプロジェクトのライセンスについては、リポジトリのLICENSEファイルを参照してください。

## 謝辞

- [VOICEVOX](https://voicevox.hiroshiba.jp/) - 無料で使える中品質なテキスト読み上げソフトウェア
- [discordgo](https://github.com/bwmarrin/discordgo) - Discord API のGo実装