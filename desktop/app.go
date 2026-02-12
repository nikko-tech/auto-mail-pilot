package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"html"
	"mime"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"desktop/internal/api"
	"desktop/internal/config"
	"desktop/internal/logger"
	"desktop/internal/models"

	"github.com/wailsapp/wails/v2/pkg/runtime"
)

// App はアプリケーションの状態を管理
type App struct {
	ctx       context.Context
	config    *config.Config
	apiClient *api.Client
}

// NewApp は新しいAppインスタンスを作成
func NewApp() *App {
	return &App{}
}

// startup はアプリ起動時に呼ばれる
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx

	// ロガー初期化
	if err := logger.Init(); err != nil {
		fmt.Printf("ロガー初期化失敗: %v\n", err)
	}

	// 設定読み込み
	cfg, err := config.Load()
	if err != nil {
		logger.Error("設定読み込み失敗: %v", err)
		cfg = config.DefaultConfig()
	}
	a.config = cfg

	// ファイルベースHTML署名を読み込み
	a.loadSignatureFromFile()

	// APIクライアント初期化
	a.apiClient = api.NewClient(cfg.GASURL)
	a.apiClient.SetBasicAuth(cfg.BasicAuthID, cfg.BasicAuthPW)

	logger.Info("アプリケーション準備完了")
}

// loadSignatureFromFile はexeディレクトリのsignature.htmlを読み込み、
// 画像参照をBase64データURIに変換してメモリに保持する
func (a *App) loadSignatureFromFile() {
	defer func() {
		if r := recover(); r != nil {
			logger.Error("署名読み込みでパニック発生: %v", r)
		}
	}()

	exeDir, err := config.GetExeDir()
	if err != nil {
		logger.Error("exeディレクトリ取得失敗: %v", err)
		return
	}
	logger.Info("署名検索ディレクトリ: %s", exeDir)

	sigPath := filepath.Join(exeDir, "signature.html")
	data, err := os.ReadFile(sigPath)
	if err != nil {
		logger.Info("signature.html未検出（スキップ）: %v", err)
		return
	}
	logger.Info("signature.html読み込み成功 (%d bytes)", len(data))

	htmlContent := string(data)

	// <img src="ファイル名"> を検索してBase64データURIに置換
	imgRe := regexp.MustCompile(`<img\s[^>]*src="([^"]+)"`)
	matches := imgRe.FindAllStringSubmatchIndex(htmlContent, -1)
	logger.Info("imgタグ検出: %d件", len(matches))

	// 後ろから置換（インデックスずれ防止）
	for i := len(matches) - 1; i >= 0; i-- {
		m := matches[i]
		// m[2]:m[3] がキャプチャグループ1（src属性値）
		src := htmlContent[m[2]:m[3]]
		logger.Info("img src発見: %s", src)

		if strings.HasPrefix(src, "http://") || strings.HasPrefix(src, "https://") || strings.HasPrefix(src, "data:") {
			continue
		}

		imgPath := filepath.Join(exeDir, src)
		imgData, readErr := os.ReadFile(imgPath)
		if readErr != nil {
			logger.Error("署名画像読み込み失敗: %s - %v", imgPath, readErr)
			continue
		}

		ext := strings.ToLower(filepath.Ext(src))
		mimeType := "image/png"
		switch ext {
		case ".jpg", ".jpeg":
			mimeType = "image/jpeg"
		case ".gif":
			mimeType = "image/gif"
		case ".svg":
			mimeType = "image/svg+xml"
		case ".webp":
			mimeType = "image/webp"
		}

		b64 := base64.StdEncoding.EncodeToString(imgData)
		dataURI := fmt.Sprintf("data:%s;base64,%s", mimeType, b64)
		logger.Info("署名画像Base64化: %s (%d bytes)", src, len(imgData))

		htmlContent = htmlContent[:m[2]] + dataURI + htmlContent[m[3]:]
	}

	a.config.HtmlSignature = htmlContent
	logger.Info("ファイルベースHTML署名読み込み完了 (%d bytes)", len(htmlContent))
}

// domReady はDOM準備完了時に呼ばれる
func (a *App) domReady(ctx context.Context) {
	logger.Info("domReady")
}

// shutdown はアプリ終了時に呼ばれる
func (a *App) shutdown(ctx context.Context) {
	logger.Info("アプリケーション終了")
	logger.Close()
}

// GetVersion はバージョン情報を返す
func (a *App) GetVersion() map[string]string {
	return logger.GetBuildInfo()
}

// GetConfig は現在の設定を返す
func (a *App) GetConfig() *config.Config {
	return a.config
}

// SaveConfig は設定を保存
func (a *App) SaveConfig(gasURL string, signature string) error {
	a.config.GASURL = gasURL
	a.config.Signature = signature

	if err := config.Save(a.config); err != nil {
		logger.Error("設定保存失敗: %v", err)
		return err
	}

	// APIクライアントのURLを更新
	a.apiClient.SetBaseURL(gasURL)

	logger.Info("設定保存成功")
	return nil
}

// SaveConfigWithAuth は設定を保存（Basic認証含む）
func (a *App) SaveConfigWithAuth(gasURL, signature, authID, authPW string) error {
	a.config.GASURL = gasURL
	a.config.Signature = signature
	a.config.BasicAuthID = authID
	a.config.BasicAuthPW = authPW

	if err := config.Save(a.config); err != nil {
		logger.Error("設定保存失敗: %v", err)
		return err
	}

	// APIクライアントのURLと認証情報を更新
	a.apiClient.SetBaseURL(gasURL)
	a.apiClient.SetBasicAuth(authID, authPW)

	logger.Info("設定保存成功（認証情報含む）")
	return nil
}

// ValidateSendSafety は送信前の安全性チェックを実行
func (a *App) ValidateSendSafety(recipient *models.Recipient, attachments []models.Attachment, body string) []string {
	var errors []string

	if recipient == nil {
		errors = append(errors, "宛先が選択されていません")
		return errors
	}

	// 1. 添付ファイルと宛先の整合性チェック
	for _, att := range attachments {
		if !att.Enabled {
			continue
		}
		// ファイル名から会社名を抽出してチェック
		fileName := strings.ToLower(att.FileName)
		recipientCompany := strings.ToLower(strings.ReplaceAll(recipient.Company, " ", ""))
		recipientName := strings.ToLower(strings.ReplaceAll(recipient.Name, " ", ""))

		// 法人格を除去
		corporateTypes := []string{"株式会社", "有限会社", "合同会社", "合資会社", "合名会社", "(株)", "（株）", "㈱"}
		for _, ct := range corporateTypes {
			recipientCompany = strings.ReplaceAll(recipientCompany, strings.ToLower(ct), "")
			fileName = strings.ReplaceAll(fileName, strings.ToLower(ct), "")
		}

		// ファイル名に会社名または氏名の一部が含まれているかチェック
		if recipientCompany != "" && !strings.Contains(fileName, recipientCompany) {
			// 会社名の一部（3文字以上）でもマッチすればOK
			matched := false
			if len(recipientCompany) >= 3 {
				for i := 0; i <= len(recipientCompany)-3; i++ {
					part := recipientCompany[i : i+3]
					if strings.Contains(fileName, part) {
						matched = true
						break
					}
				}
			}
			if !matched && recipientName != "" {
				// 氏名でもチェック
				nameParts := strings.Fields(recipientName)
				for _, part := range nameParts {
					if len(part) >= 2 && strings.Contains(fileName, part) {
						matched = true
						break
					}
				}
			}
			if !matched {
				errors = append(errors, fmt.Sprintf("添付ファイル「%s」が宛先「%s」と一致しない可能性があります", att.FileName, recipient.Company))
			}
		}
	}

	// 2. 本文に宛先の情報が含まれているかチェック
	bodyLower := strings.ToLower(body)
	companyLower := strings.ToLower(recipient.Company)
	nameLower := strings.ToLower(recipient.Name)

	if companyLower != "" && !strings.Contains(bodyLower, companyLower) {
		// 会社名の一部でも含まれていればOK
		found := false
		parts := strings.Fields(companyLower)
		for _, part := range parts {
			if len(part) >= 2 && strings.Contains(bodyLower, part) {
				found = true
				break
			}
		}
		if !found && nameLower != "" {
			// 氏名でもチェック
			nameParts := strings.Fields(nameLower)
			for _, part := range nameParts {
				if len(part) >= 2 && strings.Contains(bodyLower, part) {
					found = true
					break
				}
			}
		}
		if !found {
			errors = append(errors, "本文に宛先の会社名または氏名が見つかりません")
		}
	}

	return errors
}

// TestConnection は接続テストを実行
func (a *App) TestConnection() *models.ConnectionTestResponse {
	if a.config.GASURL == "" {
		return &models.ConnectionTestResponse{
			Success: false,
			Error:   "GAS URLが設定されていません",
		}
	}

	resp, err := a.apiClient.TestConnection()
	if err != nil {
		return &models.ConnectionTestResponse{
			Success: false,
			Error:   err.Error(),
		}
	}
	return resp
}

// GetTemplates はテンプレート一覧を取得
func (a *App) GetTemplates() ([]models.Template, error) {
	if a.config.GASURL == "" {
		return nil, fmt.Errorf("GAS URLが設定されていません")
	}
	return a.apiClient.GetTemplates()
}

// GetRecipients は宛先一覧を取得
func (a *App) GetRecipients() ([]models.Recipient, error) {
	if a.config.GASURL == "" {
		return nil, fmt.Errorf("GAS URLが設定されていません")
	}
	return a.apiClient.GetRecipients()
}

// GetSettings は設定（署名など）を取得
func (a *App) GetSettings() (*models.SettingsResponse, error) {
	// ローカル設定の署名を優先
	localSignature := a.config.Signature

	if a.config.GASURL == "" {
		return &models.SettingsResponse{
			Signature: localSignature,
		}, nil
	}

	// GASから設定を取得
	resp, err := a.apiClient.GetSettings()
	if err != nil {
		// GASエラー時はローカル設定を返す
		logger.Error("GAS設定取得失敗、ローカル設定を使用: %v", err)
		return &models.SettingsResponse{
			Signature: localSignature,
		}, nil
	}

	// GASから署名が取得できなければローカル設定を使用
	if resp.Signature == "" && localSignature != "" {
		resp.Signature = localSignature
	}

	return resp, nil
}

// GetSignatures は署名一覧を取得
func (a *App) GetSignatures() ([]models.Signature, error) {
	if a.config.GASURL == "" {
		return nil, fmt.Errorf("GAS URLが設定されていません")
	}
	return a.apiClient.GetSignatures()
}

// GetHtmlSignature はHTML署名を返す
func (a *App) GetHtmlSignature() string {
	return a.config.HtmlSignature
}

// SaveHtmlSignature はHTML署名を保存
func (a *App) SaveHtmlSignature(htmlSignature string) error {
	a.config.HtmlSignature = htmlSignature
	if err := config.Save(a.config); err != nil {
		logger.Error("HTML署名保存失敗: %v", err)
		return err
	}
	logger.Info("HTML署名保存成功")
	return nil
}

// SendMail はメールを送信
func (a *App) SendMail(to string, subject string, body string, attachments []models.Attachment) (*models.SendMailResponse, error) {
	if a.config.GASURL == "" {
		return nil, fmt.Errorf("GAS URLが設定されていません")
	}

	req := &models.SendMailRequest{
		To:          to,
		Subject:     subject,
		Body:        body,
		Attachments: attachments,
	}

	// HTML署名が設定されている場合、HTML本文を構築
	if a.config.HtmlSignature != "" {
		htmlBody := buildHtmlBody(body, a.config.HtmlSignature)
		// データURIをCID参照に変換（MIMEエンコーディング問題回避）
		htmlBody, inlineImages := extractInlineImages(htmlBody)
		req.HtmlBody = htmlBody
		req.InlineImages = inlineImages
		// プレーンテキストフォールバック用にタグ除去版署名を追加
		plainSig := stripHtmlTags(a.config.HtmlSignature)
		req.Body = body + "\n\n" + plainSig
		logger.Info("HTML送信: htmlBody=%d bytes, inlineImages=%d件", len(htmlBody), len(inlineImages))
	}

	return a.apiClient.SendMail(req)
}

// buildHtmlBody はプレーンテキスト本文とHTML署名を結合してHTML本文を生成
func buildHtmlBody(plainTextBody string, htmlSignature string) string {
	escaped := html.EscapeString(plainTextBody)
	htmlBodyText := strings.ReplaceAll(escaped, "\n", "<br>\n")

	return fmt.Sprintf(`<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body>
<div style="font-family: sans-serif; font-size: 14px;">
<div>%s</div>
<br>
%s
</div>
</body>
</html>`, htmlBodyText, htmlSignature)
}

// extractInlineImages はHTML内のdata:URIをCID参照に変換し、インライン画像を抽出する
// これによりhtmlBodyサイズが大幅に縮小され、MIMEエンコーディングの問題を回避する
func extractInlineImages(htmlContent string) (string, []models.InlineImage) {
	re := regexp.MustCompile(`src="data:([^;]+);base64,([^"]+)"`)
	matches := re.FindAllStringSubmatchIndex(htmlContent, -1)

	if len(matches) == 0 {
		return htmlContent, nil
	}

	var images []models.InlineImage
	result := htmlContent

	// 後ろから置換（インデックスずれ防止）
	for i := len(matches) - 1; i >= 0; i-- {
		m := matches[i]
		// m[2]:m[3] = キャプチャグループ1（MIMEタイプ）
		// m[4]:m[5] = キャプチャグループ2（Base64データ）
		mimeType := result[m[2]:m[3]]
		b64Data := result[m[4]:m[5]]
		key := fmt.Sprintf("sig_img_%d", i)

		images = append(images, models.InlineImage{
			Key:      key,
			Data:     b64Data,
			MimeType: mimeType,
		})

		// src="data:...;base64,..." → src="cid:sig_img_X"
		result = result[:m[0]] + `src="cid:` + key + `"` + result[m[1]:]
	}

	logger.Info("CID変換: %d画像をインライン画像に分離", len(images))
	return result, images
}

// stripHtmlTags はHTMLタグを除去してプレーンテキストを返す
func stripHtmlTags(s string) string {
	re := regexp.MustCompile(`<br\s*/?>`)
	result := re.ReplaceAllString(s, "\n")
	re = regexp.MustCompile(`<[^>]*>`)
	result = re.ReplaceAllString(result, "")
	result = strings.ReplaceAll(result, "&amp;", "&")
	result = strings.ReplaceAll(result, "&lt;", "<")
	result = strings.ReplaceAll(result, "&gt;", ">")
	result = strings.ReplaceAll(result, "&quot;", "\"")
	result = strings.ReplaceAll(result, "&#39;", "'")
	result = strings.ReplaceAll(result, "&nbsp;", " ")
	return strings.TrimSpace(result)
}

// SaveTemplate はテンプレートを保存
func (a *App) SaveTemplate(id, name, subject, body string) error {
	if a.config.GASURL == "" {
		return fmt.Errorf("GAS URLが設定されていません")
	}

	req := &models.SaveTemplateRequest{
		ID:      id,
		Name:    name,
		Subject: subject,
		Body:    body,
	}

	return a.apiClient.SaveTemplate(req)
}

// OpenFileDialog はファイル選択ダイアログを開く
func (a *App) OpenFileDialog() ([]models.Attachment, error) {
	files, err := runtime.OpenMultipleFilesDialog(a.ctx, runtime.OpenDialogOptions{
		Title: "添付ファイルを選択",
	})
	if err != nil {
		logger.Error("ファイル選択失敗: %v", err)
		return nil, err
	}

	var attachments []models.Attachment
	for _, filePath := range files {
		att, err := a.readFileAsAttachment(filePath)
		if err != nil {
			logger.Error("ファイル読み込み失敗: %s - %v", filePath, err)
			continue
		}
		attachments = append(attachments, *att)
	}

	logger.Info("ファイル選択: %d件", len(attachments))
	return attachments, nil
}

// ReadFileAsAttachment はファイルを添付ファイル形式で読み込む
func (a *App) ReadFileAsAttachment(filePath string) (*models.Attachment, error) {
	return a.readFileAsAttachment(filePath)
}

// ReadFilesAsAttachments は複数のファイルパスから添付ファイルを読み込む（ドラッグ＆ドロップ用）
func (a *App) ReadFilesAsAttachments(filePaths []string) ([]models.Attachment, error) {
	var attachments []models.Attachment
	for _, filePath := range filePaths {
		att, err := a.readFileAsAttachment(filePath)
		if err != nil {
			logger.Error("ファイル読み込み失敗: %s - %v", filePath, err)
			continue
		}
		attachments = append(attachments, *att)
	}
	logger.Info("ドロップファイル読み込み: %d件", len(attachments))
	return attachments, nil
}

func (a *App) readFileAsAttachment(filePath string) (*models.Attachment, error) {
	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("ファイル読み込み失敗: %w", err)
	}

	// MIMEタイプを推定
	ext := filepath.Ext(filePath)
	mimeType := mime.TypeByExtension(ext)
	if mimeType == "" {
		mimeType = "application/octet-stream"
	}

	return &models.Attachment{
		FilePath: filePath,
		FileName: filepath.Base(filePath),
		Enabled:  true,
		Data:     base64.StdEncoding.EncodeToString(data),
		MimeType: mimeType,
	}, nil
}

// ApplyTemplateVariables はテンプレート変数を置換
func (a *App) ApplyTemplateVariables(text string, recipient *models.Recipient) string {
	if recipient == nil {
		return text
	}

	replacements := map[string]string{
		"{{name}}":    recipient.Name,
		"{{company}}": recipient.Company,
		"{{email}}":   recipient.Email,
		"{{id}}":      recipient.ID,
	}

	result := text
	for placeholder, value := range replacements {
		result = strings.ReplaceAll(result, placeholder, value)
	}

	return result
}

// MatchAllRecipientsByFileName はファイル名からマッチする宛先を全件返す
func (a *App) MatchAllRecipientsByFileName(fileName string, recipients []models.Recipient) []models.Recipient {
	baseName := strings.TrimSuffix(fileName, filepath.Ext(fileName))
	parts := strings.FieldsFunc(baseName, func(r rune) bool {
		return r == '_' || r == ' ' || r == '(' || r == ')' || r == '-'
	})

	var matched []models.Recipient
	seen := map[string]bool{}

	for _, part := range parts {
		normalizedPart := normalizeString(part)
		if normalizedPart == "" {
			continue
		}

		for i, recipient := range recipients {
			if seen[recipient.ID] {
				continue
			}
			normalizedName := normalizeString(recipient.Name)
			normalizedCompany := normalizeString(recipient.Company)
			combined := normalizedName + normalizedCompany

			if strings.Contains(normalizedName, normalizedPart) ||
				strings.Contains(normalizedCompany, normalizedPart) ||
				strings.Contains(combined, normalizedPart) {
				logger.Info("宛先マッチ: %s -> %s (%s)", part, recipient.Name, recipient.Company)
				matched = append(matched, recipients[i])
				seen[recipient.ID] = true
			}
		}
	}

	return matched
}

// MatchRecipientByFileName はファイル名から宛先をマッチング
func (a *App) MatchRecipientByFileName(fileName string, recipients []models.Recipient) *models.Recipient {
	// ファイル名を正規化（拡張子除去、区切り文字で分割）
	baseName := strings.TrimSuffix(fileName, filepath.Ext(fileName))
	parts := strings.FieldsFunc(baseName, func(r rune) bool {
		return r == '_' || r == ' ' || r == '(' || r == ')' || r == '-'
	})

	// 各パーツで宛先をマッチング
	for _, part := range parts {
		normalizedPart := normalizeString(part)
		if normalizedPart == "" {
			continue
		}

		for i, recipient := range recipients {
			normalizedName := normalizeString(recipient.Name)
			normalizedCompany := normalizeString(recipient.Company)
			combined := normalizedName + normalizedCompany

			if strings.Contains(normalizedName, normalizedPart) ||
				strings.Contains(normalizedCompany, normalizedPart) ||
				strings.Contains(combined, normalizedPart) {
				logger.Info("宛先マッチ: %s -> %s (%s)", part, recipient.Name, recipient.Company)
				return &recipients[i]
			}
		}
	}

	return nil
}

// MatchTemplateByFileName はファイル名からテンプレートをマッチング
func (a *App) MatchTemplateByFileName(fileName string, templates []models.Template) *models.Template {
	baseName := strings.TrimSuffix(fileName, filepath.Ext(fileName))
	parts := strings.FieldsFunc(baseName, func(r rune) bool {
		return r == '_' || r == ' ' || r == '(' || r == ')' || r == '-'
	})

	for _, part := range parts {
		normalizedPart := normalizeString(part)
		if normalizedPart == "" {
			continue
		}

		for i, template := range templates {
			normalizedName := normalizeString(template.Name)
			if strings.Contains(normalizedName, normalizedPart) ||
				strings.Contains(normalizedPart, normalizedName) {
				logger.Info("テンプレートマッチ: %s -> %s", part, template.Name)
				return &templates[i]
			}
		}
	}

	return nil
}

// normalizeString は文字列を正規化（スペース除去、小文字化）
func normalizeString(s string) string {
	return strings.ToLower(strings.ReplaceAll(s, " ", ""))
}
