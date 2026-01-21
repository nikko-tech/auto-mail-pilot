# Auto Mail Pilot

Rust (egui) と Google Apps Script (GAS) を連携させたメール自動送信アプリケーションです。

## 🚀 クイックスタート

### 1. 前提条件の確認
以下のツールがインストールされている必要があります。
- **Rust**: `cargo --version`
- **Node.js**: `node -v` (GASのデプロイツール `clasp` に必要です)

### 2. GAS (バックエンド) のセットアップ (clasp使用)

自動化ツール `clasp` を使ってデプロイします。

#### 手順
1. **clasp のインストール**:
   ```bash
   npm install -g @google/clasp
   ```

2. **Googleログイン**:
   ブラウザが開き、認証を求められます。
   ```bash
   clasp login
   ```

3. **プロジェクトの作成**:
   `gas` ディレクトリに移動し、新規プロジェクトを作成します（スプレッドシート付き）。
   ```bash
   cd gas
   clasp create --type sheets --title "Auto Mail Pilot DB"
   ```
   ※ ここで「既存のファイル上書き」警告が出るかもしれませんが、リポジトリ内の `Code.gs` を使うので問題ありません。

4. **コードのアップロード**:
   ```bash
   clasp push
   ```

5. **ウェブアプリとしてデプロイ**:
   ```bash
   clasp deploy --description "Initial Deploy"
   ```
   **重要**: `clasp deploy` だけでは「ウェブアプリ」としての公開設定（全員アクセス可能など）が完了しない場合があります。
   出力されたURLにアクセスして動作しない場合は、以下の手順でWebブラウザから設定を確認してください：
   1. `clasp open` でブラウザを開く
   2. [デプロイ] > [デプロイを管理] (または新規デプロイ)
   3. **アクセスできるユーザー** を **「全員 (Anyone)」** に設定

### 3. アプリの起動

ルートディレクトリ（`Cargo.toml` がある場所）に戻り、アプリを起動します。

```bash
cd ..
cargo run
```

### 4. アプリ設定
1. アプリの **Settings** タブを開きます。
2. デプロイしたGASの **ウェブアプリURL** を入力します。
   （`clasp open --webapp` などで確認、あるいは手動デプロイ時のURL）
3. **Test Connection** を押して接続確認します。

## 🐞 トラブルシューティング

- **日本語フォントが豆腐になる**: Windowsのデフォルトフォントを使用していますが、表示されない場合はシステムフォントの設定を確認してください。
- **GASエラー**: `clasp push` 時に `.claspignore` がないと余計なファイルがアップロードされることがあります。
- **CORSエラー**: Webアプリのアクセス権限が「自分のみ」になっていると外部から叩けません。「全員」に設定してください。
