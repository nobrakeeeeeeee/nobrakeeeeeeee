# No Brakeeeeeeee

オートレースのデータを収集・蓄積・閲覧するためのデスクトップアプリです。

AUTORACE.JP の内部APIから、出走表・レース結果・選手情報を人間が閲覧する程度の速度で収集し、ローカルのSQLiteに保存します。収集したデータはアプリ上で検索・閲覧できます。

## 主な機能

- **本日の開催** — AUTORACE.JP からライブで開催情報を取得して一覧表示
- **開催・レース閲覧** — 収集済みの出走表・結果・レース情報（レース名、天候など）を場・日付・レース番号で閲覧
- **選手検索** — 選手マスタの同期・名前/カナ/選手コードでの検索、プロフィールと収集済み戦績の表示
- **データ収集**
  - 特定の日付、期間（開始日〜終了日）、場ごとの最近の開催、全場一括に対応
  - 収集は1リクエストあたり約1.2秒の間隔で実行（サーバー負荷への配慮）
  - キャンセル可能、途中再開も可能（再収集は上書き更新）
- **自動アップデート** — GitHub Releases の `latest.json` を参照し、新バージョン検出時にバナーからワンクリック更新（Tauri Updater、署名検証付き）

## 技術スタック

| 層 | 技術 |
|---|---|
| デスクトップフレームワーク | Tauri 2 |
| バックエンド | Rust（APIクライアント / SQLite / Tauriコマンド） |
| フロントエンド | React + TypeScript + Vite |
| データベース | SQLite（rusqlite, WALモード） |

## 必要環境

### 開発

- Rust（stable, rustup経由）
- Node.js 18+ / npm
- Linux での開発には以下のシステムパッケージが必要です（Ubuntu/Debian系）:

  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev
  ```

### 実行

ビルド済みのインストーラー（Windows: `.msi` / `.exe`）を使用してください。

## セットアップ

```bash
npm install
npm run tauri dev
```

初回起動時にアプリデータ用ディレクトリへ `nobrake.db` が作成されます。

## 使い方

1. **選手タブ → 「マスタ同期」** で選手一覧を取得（任意だが推奨）
2. **収集タブ** でデータを収集
   - 開始日〜終了日を指定して「期間を収集」
   - または「最近の開催を一括収集」で各場の直近開催をまとめて取得
3. **開催・レースタブ** で収集データを閲覧
4. **選手タブ** で選手を検索し、プロフィール・戦績を確認

## データについて

### データソース

[AUTORACE.JP](https://autorace.jp/)（オートレース公式サイト）が内部的に利用しているJSON APIを利用しています。エンドポイント例:

- `GET /race_info/XML/Hold/Today` — 本日の開催
- `POST /race_info/Program` — 出走表・選手別成績
- `POST /race_info/RaceResult` — レース結果
- `POST /race_info/OtherRaceInfo` — レース情報（レース名・天候など）
- `GET /race_info/Players` — 選手マスタ
- `POST /race_info/Profile` — 選手プロフィール

POSTリクエストにはセッションCookieとCSRFトークン（`X-XSRF-TOKEN`）が必要で、アプリ側で自動管理・期限切れ時の再取得を行います。

### 収集ポリシー

- リクエスト間隔は最小 **1.2秒** に制限しています
- 大量の過去データを一度に収集する場合は、サーバーへの負荷と利用規約を考慮し、必要な範囲に留めてください
- 収集したデータの二次利用・再配布については、AUTORACE.JP および JKA の利用規約をご確認ください

### データベース構成

| テーブル | 内容 |
|---|---|
| `holds` | 開催日ごとの開催情報（場・期間・グレード・タイトル） |
| `programs` | Program APIの生JSONレスポンス |
| `entries` | 出走表（車番・選手・級別・ハンデ等） |
| `race_results` | レース結果（着順・タイム・ST等） |
| `race_infos` | レース情報（レース名・天候・発走時刻） |
| `players` | 選手マスタ |

## プロジェクト構成

```
├── src/                  # フロントエンド（React + TypeScript）
│   ├── api.ts            # Tauriコマンドのラッパーと型定義
│   └── views/            # 画面コンポーネント
└── src-tauri/
    └── src/
        ├── api.rs        # AUTORACE.JP APIクライアント（セッション/レート制限）
        ├── db.rs         # SQLiteスキーマとクエリ
        ├── commands.rs   # Tauriコマンド（収集・検索・閲覧）
        └── model.rs      # 共有データ型
```

## テスト

```bash
cd src-tauri
cargo test          # DBのユニットテスト
cargo test -- --ignored   # 実APIへのスモークテスト（ネットワーク必須）
```

## ライセンス

[MIT License](LICENSE)
