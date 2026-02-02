# Auto Mail Pilot

メール自動送信アプリケーション。Google SheetsとGASを連携し、複数宛先への一括メール送信を自動化。

---

## 起動方法（優先順位）

> **HTML/JavaScript版を最優先で起動すること。** ビルド不要で即座に使える。

| 優先度 | 版 | 起動方法 | 備考 |
|--------|-----|---------|------|
| **1（推奨）** | HTML/JS版 | `web\index.html` をブラウザで開く | ビルド不要・即起動 |
| 2 | Web版 (FastAPI) | `cd auto-mail-pilot-web && python app.py` | Python必要 |
| 3 | Electron版 | `cd auto-mail-pilot-electron && npm run dev` | Node.js必要 |
| 4 | Java版 | `cd auto-mail-pilot-java && mvn javafx:run` | Maven必要 |
| 5 | Rust版 | `cd auto-mail-pilot && cargo run` | 初回ビルド長い |

---

## 実行環境

### HTML/JS版（推奨）
- **形式**: 単一HTMLファイル（ブラウザで実行）
- **依存**: Tailwind CSS (CDN)
- **場所**: `auto-mail-pilot/web/index.html`
- **必要条件**: ブラウザのみ

### Rust版
- **言語**: Rust + egui (GUI)
- **場所**: `auto-mail-pilot/`
- **必要条件**: Rust (cargo)

### バックエンド（共通）
- **バックエンド**: Google Apps Script (GAS)
- **データソース**: Google Sheets

---

## 機能

| 機能 | 説明 |
|------|------|
| 動的宛先取得 | Google Sheets「宛先リスト」から自動同期 |
| 変数置換 | `{{name}}`, `{{company}}`, `{{email}}`, `{{id}}` の自動置換 |
| 複数宛先送信 | 最大3名までタブ形式で個別編集・一括送信 |
| 署名管理 | 「署名」シートから取得し、送信時に自動挿入 |
| テンプレート紐付け | 「紐付けマスター」に基づく自動テンプレート適用 |

---

## 認証情報

- ユーザー名: `nikko`
- パスワード: `nikko`

---

## Google Sheets構成

| シート名 | 内容 |
|---------|------|
| 宛先リスト | name, company, email, id |
| テンプレート | 件名・本文テンプレート |
| 署名 | メール署名 |
| 紐付けマスター | 宛先-テンプレート対応 |

---

## 使用例

### 基本

```
「メールアプリを起動して」
→ web\index.html をブラウザで開く
→ ログイン（nikko/nikko）
→ 宛先・テンプレート選択
→ 送信
```

### 個別編集

```
「田中さんだけ件名変えて」
→ タブで個別に編集
→ 他はそのまま送信
```

---

## 注意事項

- GASのアクセス権限は「全員 (Anyone)」に設定必要
- 大量送信時はGmailの送信制限に注意
- HTML/JS版はlocalStorageにGAS URLを保存する
