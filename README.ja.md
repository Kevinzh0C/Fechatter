<div align="center">
  <img src="./assets/logo.svg" alt="Fechatter Logo" width="120" height="120">

<h1>Fechatter</h1>

<p>
    <strong>Rust製リアルタイムチャットプラットフォーム</strong>
  </p>

<p>
    <a href="README.md">🇺🇸 English</a> •
    <a href="README.zh-CN.md">🇨🇳 中文</a> •
    <a href="README.ja.md">🇯🇵 日本語</a>
  </p>

<p>
    <a href="https://github.com/Kevinzh0C/Fechatter/blob/master/LICENSE">
      <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License">
    </a>
    <a href="https://www.rust-lang.org/">
      <img src="https://img.shields.io/badge/built%20with-Rust-orange.svg" alt="Built with Rust">
    </a>
    <a href="https://github.com/Kevinzh0C/Fechatter/actions">
      <img src="https://github.com/Kevinzh0C/Fechatter/workflows/build/badge.svg" alt="ビルドステータス">
    </a>
  </p>

  <p>
    <a href="https://fechatter-frontend.vercel.app">🚀 ライブデモ</a> •
    <a href="#-機能">機能</a> •
    <a href="#-アーキテクチャ">アーキテクチャ</a> •
    <a href="./ROADMAP.md">🗺️ ロードマップ</a> •
    <a href="#-開発">開発</a>
  </p>
</div>

---

## ✨ Fechatterとは？

FechatterはRustで構築されたリアルタイムチャットプラットフォームです。高性能メッセージング、AI統合、全文検索機能を提供します。

<div align="center">
  <a href="https://fechatter-frontend.vercel.app" target="_blank">
    <img src="https://img.shields.io/badge/ライブデモ-Fechatterを試す-brightgreen?style=for-the-badge&logo=vercel" alt="ライブデモ">
  </a>
</div>

## 🎯 機能

### コア機能

- **リアルタイムメッセージング** - NATS JetStreamによるメッセージ配信
- **AIチャットボット** - OpenAI API統合
- **全文検索** - Meilisearchによる高速検索
- **ワークスペース** - チャットとユーザーの整理
- **ファイル共有** - ファイルアップロードと共有
- **JWT認証** - セキュアなトークンベース認証

### パフォーマンス最適化

- **Arena分配器** - メモリ分配の最適化
- **オブジェクトプール** - オブジェクト再利用
- **ゼロコピー解析** - 効率的なデータ処理
- **多層キャッシュ** - メモリ + Redis + PostgreSQLキャッシュ

## 🏗️ アーキテクチャ

マイクロサービスアーキテクチャを採用し、各サービスが独立して動作します。

### サービス構成

| サービス                    | PostgreSQL | Redis | ClickHouse | NATS | Meilisearch | OpenAI | S3 |
| --------------------------- | :--------: | :---: | :--------: | :--: | :---------: | :----: | :-: |
| **fechatter_server**  |     ✓     |  ✓  |     -     |  ✓  |     ✓     |   -   | ✓ |
| **notify_server**     |     -     |  ✓  |     -     |  ✓  |      -      |   -   | - |
| **bot_server**        |     -     |  ✓  |     -     |  ✓  |      -      |   ✓   | - |
| **analytics_server**  |     -     |   -   |     ✓     |  ✓  |      -      |   -   | - |
| **fechatter_gateway** |     -     |  ✓  |     -     |  -  |      -      |   -   | - |

詳細なアーキテクチャ図は[ARCHITECTURE.md](./ARCHITECTURE.md)をご覧ください。

## 💻 開発

### クイックスタート

```bash
# リポジトリをクローン
git clone https://github.com/Kevinzh0C/fechatter.git
cd fechatter

# 環境設定
cp .env.example .env

# サービス起動
docker-compose up -d

# テスト実行
cargo test
```

### 技術スタック

#### バックエンド
- **言語**: Rust 1.75+
- **フレームワーク**: Axum, Tokio
- **データベース**: PostgreSQL, Redis, ClickHouse
- **メッセージキュー**: NATS JetStream

#### フロントエンド
- **フレームワーク**: Vue 3, TypeScript
- **ビルドツール**: Vite
- **状態管理**: Pinia

## 📚 ドキュメント

- [クイックスタート](./docs/QUICK_START.md) - 基本的な使い方
- [インストール](./docs/INSTALLATION.md) - 詳細なセットアップ
- [アーキテクチャ](./ARCHITECTURE.md) - システム設計
- [API リファレンス](./fechatter_server/docs/API_REFERENCE.md) - REST API
- [ロードマップ](./ROADMAP.md) - 開発計画

## 🤝 コントリビューション

プロジェクトへの貢献を歓迎します。

### 貢献方法

1. バグ修正
2. テストの追加
3. ドキュメントの改善
4. 新機能の提案

詳細は[コントリビューションガイド](./CONTRIBUTING.md)をご覧ください。

## 📄 ライセンス

このプロジェクトはMITライセンスの下で公開されています。詳細は[LICENSE](./LICENSE)をご覧ください。

---

<div align="center">
  <p>
    <a href="https://github.com/Kevinzh0C/Fechatter">⭐ GitHubでスターをお願いします</a>
  </p>
</div>
