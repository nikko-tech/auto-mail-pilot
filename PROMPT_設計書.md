# メール自動送信アプリ 設計プロンプト

## 概要
Rust + egui で構築するデスクトップアプリケーション。
Google Apps Script (GAS) と連携し、Googleスプレッドシートをデータストアとして使用する。

---

## アーキテクチャ

```
┌─────────────────────────────────────────────────────────────┐
│                    メール自動送信アプリ (Rust/egui)           │
├──────────────────┬──────────────────────────────────────────┤
│  [タブ切替]       │   [選択中タブの内容]                      │
│  ・メイン(送信)    │                                          │
│  ・管理/設定       │                                          │
└──────────────────┴──────────────────────────────────────────┘

### 1. メインタブ（送信画面）
- **左カラム**: 宛先選択（マスタ連携）・OCRボタン
- **中央カラム**: 自動反映された件名・本文の最終調整
- **右カラム**: 添付ファイル管理 & 送信ボタン

### 2. 管理/設定タブ
- **GAS連携管理**: エンドポイントURL、接続テスト
- **テンプレート編集**: スプレッドシートを介さずアプリ内から直接プレビュー編集
- **署名設定**: 共通署名の登録・編集
- **マスタデータ管理**: 宛先リストの簡易閲覧・SSへのクイックアクセス
                              │
                              ▼ HTTP (doGet/doPost)
┌─────────────────────────────────────────────────────────────┐
│                Google Apps Script (GAS)                     │
│  ・doGet: テンプレート一覧取得                               │
│  ・doPost: メール送信 / テンプレート保存                     │
│  ・OCR処理: PDFからメールアドレス/名前抽出 (Cloud Vision)     │
│  ※送信元はGASを承認したGmailアカウントになります。           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Googleスプレッドシート                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ テンプレートシート │  │ 宛先リストシート │  │ 設定シート       │  │
│  │ A: テンプレート名  │  │ A: 会社/氏名      │  │ A: 署名文        │  │
│  │ B: 件名           │  │ B: メールアドレス  │  │                  │  │
│  │ C: 本文(改行可)    │  │ C: 紐付けテンプレートID │                │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 1. Rust 構造体定義

### Template 構造体
```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Template {
    pub id: String,           // 一意識別子 (行番号など)
    pub name: String,         // テンプレート名
    pub subject: String,      // 件名
    pub body: String,         // 本文 ({{name}}, {{company}} 等の変数対応)
}
```

### MailDraft 構造体
```rust
#[derive(Clone, Debug, Default)]
pub struct MailDraft {
    pub recipients: Vec<RecipientInfo>, // 最大3件の宛先情報
    pub subject: String,               // 件名 (全宛先共通)
    pub attachments: Vec<Attachment>,  // 添付ファイル一覧
    pub signature: String,             // 署名 (設定から取得)
}

#[derive(Clone, Debug, Default)]
pub struct RecipientInfo {
    pub email: String,                 // 宛先メールアドレス
    pub body: String,                  // この宛先専用の本文
}

#[derive(Clone, Debug)]
pub struct Attachment {
    pub file_path: String,   // ファイルパス
    pub file_name: String,   // 表示用ファイル名
    pub enabled: bool,       // 送信対象に含めるか
}
```

### AppState 構造体
```rust
pub struct AppState {
    // テンプレート関連
    pub templates: Vec<Template>,
    pub selected_template_index: Option<usize>,
    pub editing_template: Option<Template>,

    // メール下書き
    pub mail_draft: MailDraft,

    // UI状態
    pub status_message: String,
    pub is_loading: bool,
}
```

---

## 2. egui UI構成 サンプルコード

```rust
        // サイドバー: タブ切り替え
        egui::SidePanel::left("nav_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.selectable_value(&mut self.state.tab, Tab::Main, "送信");
                ui.selectable_value(&mut self.state.tab, Tab::Settings, "設定・管理");
            });
        });

        match self.state.tab {
            Tab::Main => self.show_main_tab(ui),
            Tab::Settings => self.show_settings_tab(ui),
        }
    }
}

impl MailApp {
    fn show_main_tab(&mut self, ui: &mut egui::Ui) {
        // 送信画面レイアウト (3カラム構成)
        egui::SidePanel::left("recipient_list").show_inside(ui, |ui| {
             ui.heading("宛先マスタから選択");
             // 宛先リスト表示・OCRボタン
        });
        
        egui::CentralPanel::default().show_inside(ui, |ui| {
             ui.heading("メール内容の確認");
             // 件名・本文・署名の編集エリア
        });

        egui::SidePanel::right("submit_panel").show_inside(ui, |ui| {
             ui.heading("添付 & 送信");
             // ファイルリスト、送信ボタン
        });
    }

    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("システム設定");
        ui.group(|ui| {
            ui.label("GAS WebアプリURL:");
            ui.text_edit_singleline(&mut self.state.config.gas_url);
            if ui.button("接続テスト").clicked() { /* ... */ }
        });

        ui.separator();
        ui.heading("テンプレート・署名管理");
        // ... 他の設定項目
    }
}
    }
}
```

---

## 3. GAS側コード

### doGet - テンプレート一覧取得
```javascript
function doGet(e) {
  const action = e.parameter.action || 'getTemplates';

  if (action === 'getTemplates') {
    return getTemplates();
  }

  return ContentService.createTextOutput(JSON.stringify({ error: 'Unknown action' }))
    .setMimeType(ContentService.MimeType.JSON);
}

function getTemplates() {
  const ss = SpreadsheetApp.getActiveSpreadsheet();
  const sheet = ss.getSheetByName('テンプレート');
  const data = sheet.getDataRange().getValues();

  const templates = [];
  for (let i = 1; i < data.length; i++) {  // ヘッダー行をスキップ
    templates.push({
      id: String(i + 1),  // 行番号をIDとして使用
      name: data[i][0],
      subject: data[i][1],
      body: data[i][2]
    });
  }

  return ContentService.createTextOutput(JSON.stringify({ templates: templates }))
    .setMimeType(ContentService.MimeType.JSON);
}
```

### doPost - メール送信 / テンプレート保存
```javascript
function doPost(e) {
  const payload = JSON.parse(e.postData.contents);
  const action = payload.action;

  switch (action) {
    case 'sendMail':
      return sendMail(payload);
    case 'saveTemplate':
      return saveTemplate(payload);
    default:
      return ContentService.createTextOutput(JSON.stringify({ error: 'Unknown action' }))
        .setMimeType(ContentService.MimeType.JSON);
  }
}

function sendMail(payload) {
  try {
    const options = {
      to: payload.to,
      subject: payload.subject,
      body: payload.body,
    };

    // 添付ファイルがある場合
    if (payload.attachments && payload.attachments.length > 0) {
      options.attachments = payload.attachments.map(att => {
        // Base64デコードしてBlobを作成
        return Utilities.newBlob(
          Utilities.base64Decode(att.data),
          att.mimeType,
          att.fileName
        );
      });
    }

    GmailApp.sendEmail(options.to, options.subject, options.body, {
      attachments: options.attachments
    });

    // 送信ログを記録 (オプション)
    logSentMail(payload);

    return ContentService.createTextOutput(JSON.stringify({ success: true }))
      .setMimeType(ContentService.MimeType.JSON);
  } catch (error) {
    return ContentService.createTextOutput(JSON.stringify({
      success: false,
      error: error.toString()
    }))
      .setMimeType(ContentService.MimeType.JSON);
  }
}

function saveTemplate(payload) {
  try {
    const ss = SpreadsheetApp.getActiveSpreadsheet();
    const sheet = ss.getSheetByName('テンプレート');

    const rowIndex = parseInt(payload.id);

    if (rowIndex > 1) {
      // 既存テンプレートの更新
      sheet.getRange(rowIndex, 1, 1, 3).setValues([
        [payload.name, payload.subject, payload.body]
      ]);
    } else {
      // 新規テンプレートの追加
      sheet.appendRow([payload.name, payload.subject, payload.body]);
    }

    return ContentService.createTextOutput(JSON.stringify({ success: true }))
      .setMimeType(ContentService.MimeType.JSON);
  } catch (error) {
    return ContentService.createTextOutput(JSON.stringify({
      success: false,
      error: error.toString()
    }))
      .setMimeType(ContentService.MimeType.JSON);
  }
}

function logSentMail(payload) {
  const ss = SpreadsheetApp.getActiveSpreadsheet();
  let logSheet = ss.getSheetByName('送信ログ');

  if (!logSheet) {
    logSheet = ss.insertSheet('送信ログ');
    logSheet.appendRow(['送信日時', '宛先', '件名', '本文']);
  }

  logSheet.appendRow([
    new Date(),
    payload.to,
    payload.subject,
    payload.body.substring(0, 100) + '...'  // 本文は先頭100文字のみ
  ]);
}
```

---

## 4. 変数置換ロジック (Rust)

```rust
impl MailApp {
    /// テンプレートの変数を実際の値に置換
    fn apply_template_variables(&self, template: &Template, variables: &HashMap<String, String>) -> MailDraft {
        let mut subject = template.subject.clone();
        let mut body = template.body.clone();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);  // {{key}} 形式
            subject = subject.replace(&placeholder, value);
            body = body.replace(&placeholder, value);
        }

        MailDraft {
            to: variables.get("email").cloned().unwrap_or_default(),
            subject,
            body,
            attachments: Vec::new(),
        }
    }
}
```

---

## 5. 依存クレート (Cargo.toml)

```toml
[package]
name = "mail-sender"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.29"
egui = "0.29"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json", "blocking"] }
tokio = { version = "1", features = ["full"] }
base64 = "0.22"
```

---

## 6. 今後の拡張候補

1. **宛先リストのCSV/Excel読み込み**: 一括送信機能
2. **送信履歴の表示**: アプリ内で過去の送信履歴を確認
3. **スケジュール送信**: 指定日時に自動送信
4. **エラーハンドリング強化**: 送信失敗時のリトライ機能
5. **OAuth認証**: より安全なGoogleAPI連携

---

## ファイル構成 (予定)

```
メール送信/
├── PROMPT_設計書.md      # この設計書
├── src/
│   ├── main.rs           # エントリーポイント
│   ├── app.rs            # AppState, MailApp 実装
│   ├── models.rs         # Template, MailDraft, Attachment
│   ├── api.rs            # GAS通信ロジック
│   └── ui/
│       ├── mod.rs
│       ├── template_panel.rs
│       └── mail_panel.rs
├── Cargo.toml
└── gas/
    └── Code.gs           # GASスクリプト
```
