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
- ⚡ スラッシュコマンド対応
  - `/join` - ボイスチャンネルに参加
  - `/leave` - ボイスチャンネルから退出
  - `/add_word` - 辞書に単語を追加
  - `/cancel` - 現在の読み上げをキャンセル

## 技術スタック

- **言語**: Go 1.20
- **Discord API**: [discordgo](https://github.com/bwmarrin/discordgo) v0.27.1
- **音声合成**: VOICEVOX Engine
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
```

- `DISCORD_TOKEN`: Discord Bot のトークン
- `DISCORD_GUILD_ID`: 対象のDiscordサーバー（ギルド）のID
- `DISCORD_YOMIAGE_CH_ID`: 読み上げ対象のテキストチャンネルのID

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
│       └── wavgenerator/        # WAV生成
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

4. VOICEVOX Engineを起動
```bash
docker run -d -p 50021:50021 voicevox/voicevox_engine:cpu-ubuntu20.04-latest
```

5. 環境変数を設定して実行
```bash
export DISCORD_TOKEN=your_token
export DISCORD_GUILD_ID=your_guild_id
export DISCORD_YOMIAGE_CH_ID=your_channel_id
export VOICEVOX_BASE_URL=http://localhost:50021
./bot
```

## トラブルシューティング

### Botがボイスチャンネルに参加できない
- Botに適切な権限（Voice Connect, Voice Speak）が付与されているか確認してください
- ギルドIDとチャンネルIDが正しく設定されているか確認してください

### 読み上げが動作しない
- VOICEVOX Engineが正常に起動しているか確認してください
- `docker-compose logs voicevox` でログを確認してください
- 環境変数 `DISCORD_YOMIAGE_CH_ID` で指定したチャンネルにメッセージを投稿しているか確認してください

### メモリ不足エラー
- VOICEVOX Engineのスレッド数を調整してください（docker-compose.yml内の `VOICEVOX_CPU_NUM_THREADS`）

## ライセンス

このプロジェクトのライセンスについては、リポジトリのLICENSEファイルを参照してください。

## 謝辞

- [VOICEVOX](https://voicevox.hiroshiba.jp/) - 無料で使える中品質なテキスト読み上げソフトウェア
- [discordgo](https://github.com/bwmarrin/discordgo) - Discord API のGo実装