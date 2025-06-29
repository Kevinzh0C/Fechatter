<div align="center">
  <img src="./assets/logo.svg" alt="Fechatter Logo" width="120" height="120">

<h1>Fechatter</h1>

<p>
    <strong>Rust駆動の効率的なエンタープライズ対応リアルタイムチャットプラットフォーム</strong>
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
    <a href="#-はじめに">はじめに</a> •
    <a href="#-機能">機能</a> •
    <a href="#-アーキテクチャ">アーキテクチャ</a> •
    <a href="#-コントリビューション">コントリビューション</a>
  </p>
</div>

---

## ✨ Fechatterとは？

Fechatterは**Rustの効率性**と**エンタープライズグレードの機能**を組み合わせ、優れたメッセージング体験を提供する**モダンで包括的なチャットプラットフォーム**です。チームコラボレーションツールやコミュニティプラットフォームを構築する際に、Fechatterは必要な機能をすべて備えた堅牢な基盤を提供します。

### 🎮 今すぐお試しください

<div align="center">
  <a href="https://fechatter-frontend.vercel.app" target="_blank">
    <img src="https://img.shields.io/badge/ライブデモ-Fechatterを試す-brightgreen?style=for-the-badge&logo=vercel" alt="ライブデモ">
  </a>
</div>

## 🎯 機能

- **リアルタイムメッセージング** - Server-Sent Events（SSE）を使用した即座のメッセージ送受信
- **AIチャットボット** - ChatGPT搭載の統合型会話アシスタント
- **メッセージ検索** - Meilisearch搭載の全文検索機能
- **ワークスペースサポート** - 独立したワークスペースでのチャットとユーザーの整理
- **ファイル共有** - 会話内でのファイルアップロードと共有
- **JWT認証** - セキュアなトークンベース認証システム
- **分析統合** - ClickHouseとApache Supersetによる使用指標の追跡
- **マイクロサービスアーキテクチャ** - 異なる機能に対応する独立したサービスによるモジュラー設計

## 🚀 はじめに

### クイックスタート

2分以内でFechatterを起動：

```bash
# リポジトリをクローン
git clone https://github.com/Kevinzh0C/fechatter.git
cd fechatter

# 環境設定をコピー
cp .env.example .env

# 全サービスを開始
docker-compose up -d

# ブラウザで開く
open http://localhost:5173
```

これで完了です！🎉

### 必要要件

- Docker 20.10以上
- Docker Compose 2.0以上
- 最低4GBのRAM
- ポート5173が利用可能

サポートが必要ですか？[クイックスタートガイド](./docs/QUICK_START.md)をご確認ください。

## 🏗️ アーキテクチャ

Fechatterはスケーラビリティと信頼性を重視した**マイクロサービスアーキテクチャ**を採用しています：

<div align="center">
  <img src="./assets/architecture.svg" alt="Fechatterアーキテクチャ図" width="600">
</div>

### システムアーキテクチャ概要

```
┌─────────────────────────────────────────────────────────────┐
│                   クライアント層                             │
│  ┌─────────────────┐     ┌──────────────────┐              │
│  │ Vue 3 + TypeScript │  │クライアントアプリ │              │
│  │      :3000      │     └──────────────────┘              │
│  └─────────────────┘                                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   ゲートウェイ層                             │
│                ┌─────────────────────┐                      │
│                │Pingoraプロキシ(:8080)│                      │
│                └─────────────────────┘                      │
└─────────────────────────────────────────────────────────────┘
                              │
     ┌────────────────────────┼────────────────────────┐
     │                        │                        │
     ▼                        ▼                        ▼
┌─────────────────────────────────────────────────────────────┐
│                    コアサービス                              │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐        │
│ │Fechatter(:6688)│ │通知(:6687)    │ │ボット(:6686) │       │
│ └──────────────┘ └──────────────┘ └──────────────┘        │
│                  ┌──────────────┐                          │
│                  │分析(:6690)    │                          │
│                  └──────────────┘                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     データ層                                 │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │
│ │PostgreSQL│ │  Redis   │ │Meilisearch│ │ClickHouse│      │
│ └──────────┘ └──────────┘ └──────────┘ └──────────┘      │
│ ┌──────────┐ ┌──────────┐                                 │
│ │   NATS   │ │    S3    │                                 │
│ └──────────┘ └──────────┘                                 │
└─────────────────────────────────────────────────────────────┘
```

### サービス依存関係マトリックス

| サービス             | PostgreSQL | Redis | ClickHouse | NATS | Meilisearch | OpenAI | S3 |
| ------------------- | :--------: | :---: | :--------: | :--: | :---------: | :----: | :-: |
| **チャット**      |     ✓     |  ✓  |     -     |  ✓  |     ✓     |   -   | ✓ |
| **通知**    |     -     |  ✓  |     -     |  ✓  |      -      |   -   | - |
| **ボット**       |     -     |  ✓  |     -     |  -  |      -      |   ✓   | - |
| **分析** |     -     |   -   |     ✓     |  ✓  |      -      |   -   | - |

### 📋 サービス概要

| サービス                    | ポート | 技術  | 目的                       |
| -------------------------- | ---- | ----------- | ----------------------------- |
| **APIゲートウェイ**      | 8080 | Pingora     | ロードバランシング、ルーティング、認証 |
| **Fechatterサーバー** | 6688 | Axum, SQLx  | コアチャット機能       |
| **通知サーバー**    | 6687 | Tokio, SSE  | リアルタイム通知       |
| **ボットサーバー**       | 6686 | OpenAI SDK  | AIチャットアシスタンス            |
| **分析サーバー** | 6690 | ClickHouse  | イベント追跡とメトリクス      |
| **フロントエンド**         | 3000 | Vue 3, Vite | ユーザーインターフェース                |

詳細は[アーキテクチャガイド](./ARCHITECTURE.md)をご覧ください。

## 💻 開発

### ローカル開発

```bash
# 依存関係をインストール
make setup

# 開発環境を開始
make dev

# テストを実行
make test

# プロダクション用にビルド
make build
```

### 技術スタック

- **バックエンド**: Rust, Axum, Tokio, SQLx
- **フロントエンド**: Vue 3, TypeScript, Vite
- **ゲートウェイ**: Pingora（Cloudflareのプロキシフレームワーク）
- **データベース**: PostgreSQL, Redis
- **検索**: Meilisearch
- **メッセージキュー**: NATS JetStream
- **分析**: ClickHouse, Apache Superset
- **デプロイメント**: Docker, Kubernetes

## 📚 ドキュメンテーション

### 入門ガイド

- [クイックスタートガイド](./docs/QUICK_START.md) - 2分で起動
- [インストールガイド](./docs/INSTALLATION.md) - 詳細セットアップ
- [設定](./fechatter_server/docs/CONFIGURATION.md) - 環境設定

### コアドキュメンテーション

- [アーキテクチャ概要](./ARCHITECTURE.md) - システム設計
- [APIリファレンス](./fechatter_server/docs/API_REFERENCE.md) - REST API
- [開発ガイド](./fechatter_server/docs/DEVELOPMENT_GUIDE.md) - 開発セットアップ

### デプロイメントと運用

- [デプロイメントガイド](./fechatter_server/docs/DEPLOYMENT_GUIDE.md) - プロダクションデプロイメント
- [パフォーマンスガイド](./fechatter_server/docs/PERFORMANCE_GUIDE.md) - 最適化のヒント

## 🤝 コントリビューション

ご協力をお待ちしています！Fechatterへのコントリビューションを可能な限り簡単で透明にしたいと考えています。

始める前に[コントリビューションガイド](./CONTRIBUTING.md)をご確認ください。

### 初心者向けのIssue

まずはどこから始めればよいかお探しですか？[good first issues](https://github.com/Kevinzh0C/Fechatter/labels/good%20first%20issue)をご確認ください。

## 📄 ライセンス

FechatterはMITライセンスの下で提供されています。詳細は[LICENSE](./LICENSE)をご覧ください。

---

<div align="center">
  <p>
    <sub>開発者によって開発者のために ❤️ で作られています</sub>
  </p>
  <p>
    <a href="https://github.com/Kevinzh0C/Fechatter">⭐ GitHubでスターをお願いします</a>
  </p>
</div> 