// Package models はアプリケーションで使用するデータ構造を定義
package models

// Template はメールテンプレート
type Template struct {
	ID        string `json:"id"`        // 一意識別子（行番号）
	Name      string `json:"name"`      // テンプレート名
	Subject   string `json:"subject"`   // 件名
	Body      string `json:"body"`      // 本文
	Signature string `json:"signature"` // 署名
}

// Recipient は宛先情報
type Recipient struct {
	ID         string `json:"id"`         // 一意識別子
	Name       string `json:"name"`       // 氏名
	Company    string `json:"company"`    // 会社名
	Email      string `json:"email"`      // メールアドレス
	TemplateID string `json:"templateId"` // 紐付けテンプレートID
}

// RecipientInfo は宛先ごとの本文情報
type RecipientInfo struct {
	Email string `json:"email"` // 宛先メールアドレス
	Body  string `json:"body"`  // この宛先専用の本文
}

// Attachment は添付ファイル情報
type Attachment struct {
	FilePath string `json:"filePath"` // ファイルパス
	FileName string `json:"fileName"` // 表示用ファイル名
	Enabled  bool   `json:"enabled"`  // 送信対象に含めるか
	Data     string `json:"data"`     // Base64エンコードされたデータ
	MimeType string `json:"mimeType"` // MIMEタイプ
}

// MailDraft はメール下書き
type MailDraft struct {
	Recipients  []RecipientInfo `json:"recipients"`  // 宛先情報（最大3件）
	Subject     string          `json:"subject"`     // 件名
	Attachments []Attachment    `json:"attachments"` // 添付ファイル一覧
	Signature   string          `json:"signature"`   // 署名
}

// SendMailRequest はメール送信リクエスト
type SendMailRequest struct {
	Action      string       `json:"action"`      // "sendMail"
	To          string       `json:"to"`          // 宛先
	Subject     string       `json:"subject"`     // 件名
	Body        string       `json:"body"`        // 本文
	Attachments []Attachment `json:"attachments"` // 添付ファイル
}

// SendMailResponse はメール送信レスポンス
type SendMailResponse struct {
	Success bool   `json:"success"`
	Error   string `json:"error,omitempty"`
}

// TemplatesResponse はテンプレート一覧取得レスポンス
type TemplatesResponse struct {
	Templates []Template `json:"templates"`
	Error     string     `json:"error,omitempty"`
}

// RecipientsResponse は宛先一覧取得レスポンス
type RecipientsResponse struct {
	Recipients []Recipient `json:"recipients"`
	Error      string      `json:"error,omitempty"`
}

// Signature は署名情報
type Signature struct {
	Name    string `json:"name"`    // 署名名
	Content string `json:"content"` // 署名内容
}

// SignaturesResponse は署名一覧取得レスポンス
type SignaturesResponse struct {
	Signatures []Signature `json:"signatures"`
	Error      string      `json:"error,omitempty"`
}

// SettingsResponse は設定取得レスポンス（GAS互換）
type SettingsResponse struct {
	Settings  map[string]interface{} `json:"settings"`
	Signature string                 `json:"signature"` // ローカル用
	Error     string                 `json:"error,omitempty"`
}

// SaveTemplateRequest はテンプレート保存リクエスト
type SaveTemplateRequest struct {
	Action  string `json:"action"` // "saveTemplate"
	ID      string `json:"id"`
	Name    string `json:"name"`
	Subject string `json:"subject"`
	Body    string `json:"body"`
}

// ConnectionTestResponse は接続テストレスポンス
type ConnectionTestResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message,omitempty"`
	Error   string `json:"error,omitempty"`
}
