// Package api はGASとのHTTP通信を管理
package api

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"desktop/internal/logger"
	"desktop/internal/models"
)

// Client はGAS APIクライアント
type Client struct {
	baseURL     string
	httpClient  *http.Client
	maxRetries  int
	basicAuthID string
	basicAuthPW string
}

// NewClient は新しいAPIクライアントを作成
func NewClient(baseURL string) *Client {
	return &Client{
		baseURL: baseURL,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
			// GASはリダイレクトするので、自動追跡を許可
			CheckRedirect: func(req *http.Request, via []*http.Request) error {
				// リダイレクト回数制限
				if len(via) >= 10 {
					return fmt.Errorf("リダイレクト回数が上限を超えました")
				}
				return nil
			},
		},
		maxRetries: 3,
	}
}

// SetBaseURL はベースURLを設定
func (c *Client) SetBaseURL(url string) {
	c.baseURL = url
}

// SetBasicAuth はBasic認証情報を設定
func (c *Client) SetBasicAuth(id, pw string) {
	c.basicAuthID = id
	c.basicAuthPW = pw
}

// getAuthHeader はBasic認証ヘッダー値を返す
func (c *Client) getAuthHeader() string {
	if c.basicAuthID == "" && c.basicAuthPW == "" {
		return ""
	}
	auth := c.basicAuthID + ":" + c.basicAuthPW
	return "Basic " + base64.StdEncoding.EncodeToString([]byte(auth))
}

// doGet はGETリクエストを実行（リトライ付き）
func (c *Client) doGet(params map[string]string) ([]byte, error) {
	url := c.baseURL
	if len(params) > 0 {
		url += "?"
		first := true
		for k, v := range params {
			if !first {
				url += "&"
			}
			url += k + "=" + v
			first = false
		}
	}

	var lastErr error
	for i := 0; i < c.maxRetries; i++ {
		if i > 0 {
			// 指数バックオフ
			time.Sleep(time.Duration(1<<uint(i)) * time.Second)
			logger.Info("リトライ %d/%d: %s", i+1, c.maxRetries, url)
		}

		req, err := http.NewRequest("GET", url, nil)
		if err != nil {
			lastErr = err
			logger.Error("リクエスト作成失敗: %v", err)
			continue
		}

		// Basic認証ヘッダーを追加
		if authHeader := c.getAuthHeader(); authHeader != "" {
			req.Header.Set("Authorization", authHeader)
		}

		resp, err := c.httpClient.Do(req)
		if err != nil {
			lastErr = err
			logger.Error("GET失敗: %v", err)
			continue
		}
		defer resp.Body.Close()

		body, err := io.ReadAll(resp.Body)
		if err != nil {
			lastErr = err
			logger.Error("レスポンス読み込み失敗: %v", err)
			continue
		}

		if resp.StatusCode != http.StatusOK {
			lastErr = fmt.Errorf("HTTPステータス: %d, Body: %s", resp.StatusCode, string(body))
			logger.Error("HTTPエラー: %v", lastErr)
			continue
		}

		return body, nil
	}

	return nil, fmt.Errorf("リトライ上限超過: %w", lastErr)
}

// doPost はPOSTリクエストを実行（リダイレクト対応・リトライ付き）
func (c *Client) doPost(payload interface{}) ([]byte, error) {
	jsonData, err := json.Marshal(payload)
	if err != nil {
		return nil, fmt.Errorf("JSONエンコード失敗: %w", err)
	}

	var lastErr error
	for i := 0; i < c.maxRetries; i++ {
		if i > 0 {
			time.Sleep(time.Duration(1<<uint(i)) * time.Second)
			logger.Info("リトライ %d/%d", i+1, c.maxRetries)
		}

		// リダイレクト時にPOSTボディが失われる問題に対応
		// 手動でリダイレクトを追跡
		client := &http.Client{
			Timeout: 30 * time.Second,
			CheckRedirect: func(req *http.Request, via []*http.Request) error {
				// リダイレクトを手動で処理するため停止
				return http.ErrUseLastResponse
			},
		}

		req, err := http.NewRequest("POST", c.baseURL, bytes.NewBuffer(jsonData))
		if err != nil {
			lastErr = err
			logger.Error("リクエスト作成失敗: %v", err)
			continue
		}
		req.Header.Set("Content-Type", "application/json")

		// Basic認証ヘッダーを追加
		if authHeader := c.getAuthHeader(); authHeader != "" {
			req.Header.Set("Authorization", authHeader)
		}

		resp, err := client.Do(req)
		if err != nil {
			lastErr = err
			logger.Error("POST失敗: %v", err)
			continue
		}

		// リダイレクトの場合、リダイレクト先にGETでアクセス
		if resp.StatusCode == http.StatusFound || resp.StatusCode == http.StatusMovedPermanently {
			location := resp.Header.Get("Location")
			resp.Body.Close()

			if location == "" {
				lastErr = fmt.Errorf("リダイレクト先が不明")
				continue
			}

			logger.Info("リダイレクト: %s", location)

			// リダイレクト先へのGETリクエストにもBasic認証を追加
			redirectReq, err := http.NewRequest("GET", location, nil)
			if err != nil {
				lastErr = err
				logger.Error("リダイレクトリクエスト作成失敗: %v", err)
				continue
			}
			if authHeader := c.getAuthHeader(); authHeader != "" {
				redirectReq.Header.Set("Authorization", authHeader)
			}

			resp, err = c.httpClient.Do(redirectReq)
			if err != nil {
				lastErr = err
				logger.Error("リダイレクト先へのアクセス失敗: %v", err)
				continue
			}
		}

		defer resp.Body.Close()

		body, err := io.ReadAll(resp.Body)
		if err != nil {
			lastErr = err
			logger.Error("レスポンス読み込み失敗: %v", err)
			continue
		}

		if resp.StatusCode != http.StatusOK {
			lastErr = fmt.Errorf("HTTPステータス: %d, Body: %s", resp.StatusCode, string(body))
			logger.Error("HTTPエラー: %v", lastErr)
			continue
		}

		return body, nil
	}

	return nil, fmt.Errorf("リトライ上限超過: %w", lastErr)
}

// TestConnection は接続テストを実行
func (c *Client) TestConnection() (*models.ConnectionTestResponse, error) {
	logger.Info("接続テスト開始: %s", c.baseURL)

	body, err := c.doGet(map[string]string{"action": "test"})
	if err != nil {
		return &models.ConnectionTestResponse{
			Success: false,
			Error:   err.Error(),
		}, err
	}

	var resp models.ConnectionTestResponse
	if err := json.Unmarshal(body, &resp); err != nil {
		// GASがJSON以外を返す場合も成功とみなす
		logger.Info("接続テスト成功（非JSON応答）")
		return &models.ConnectionTestResponse{
			Success: true,
			Message: "接続成功",
		}, nil
	}

	logger.Info("接続テスト結果: success=%v", resp.Success)
	return &resp, nil
}

// GetTemplates はテンプレート一覧を取得
func (c *Client) GetTemplates() ([]models.Template, error) {
	logger.Info("テンプレート一覧取得")

	body, err := c.doGet(map[string]string{"action": "getTemplates"})
	if err != nil {
		return nil, err
	}

	var resp models.TemplatesResponse
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("JSONデコード失敗: %w", err)
	}

	if resp.Error != "" {
		return nil, fmt.Errorf("GASエラー: %s", resp.Error)
	}

	logger.Info("テンプレート取得成功: %d件", len(resp.Templates))
	return resp.Templates, nil
}

// GetRecipients は宛先一覧を取得
func (c *Client) GetRecipients() ([]models.Recipient, error) {
	logger.Info("宛先一覧取得")

	body, err := c.doGet(map[string]string{"action": "getRecipients"})
	if err != nil {
		return nil, err
	}

	var resp models.RecipientsResponse
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("JSONデコード失敗: %w", err)
	}

	if resp.Error != "" {
		return nil, fmt.Errorf("GASエラー: %s", resp.Error)
	}

	logger.Info("宛先取得成功: %d件", len(resp.Recipients))
	return resp.Recipients, nil
}

// GetSignatures は署名一覧を取得
func (c *Client) GetSignatures() ([]models.Signature, error) {
	logger.Info("署名一覧取得")

	body, err := c.doGet(map[string]string{"action": "getSignatures"})
	if err != nil {
		return nil, err
	}

	var resp models.SignaturesResponse
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("JSONデコード失敗: %w", err)
	}

	if resp.Error != "" {
		return nil, fmt.Errorf("GASエラー: %s", resp.Error)
	}

	logger.Info("署名取得成功: %d件", len(resp.Signatures))
	return resp.Signatures, nil
}

// GetSettings は設定を取得
func (c *Client) GetSettings() (*models.SettingsResponse, error) {
	logger.Info("設定取得")

	body, err := c.doGet(map[string]string{"action": "getSettings"})
	if err != nil {
		return nil, err
	}

	var resp models.SettingsResponse
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("JSONデコード失敗: %w", err)
	}

	logger.Info("設定取得成功")
	return &resp, nil
}

// SendMail はメールを送信
func (c *Client) SendMail(req *models.SendMailRequest) (*models.SendMailResponse, error) {
	logger.Info("メール送信: to=%s, subject=%s", req.To, req.Subject)

	req.Action = "sendMail"
	body, err := c.doPost(req)
	if err != nil {
		return nil, err
	}

	var resp models.SendMailResponse
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("JSONデコード失敗: %w", err)
	}

	if resp.Error != "" {
		logger.Error("メール送信失敗: %s", resp.Error)
		return &resp, fmt.Errorf("送信エラー: %s", resp.Error)
	}

	logger.Info("メール送信成功")
	return &resp, nil
}

// SaveTemplate はテンプレートを保存
func (c *Client) SaveTemplate(req *models.SaveTemplateRequest) error {
	logger.Info("テンプレート保存: id=%s, name=%s", req.ID, req.Name)

	req.Action = "saveTemplate"
	body, err := c.doPost(req)
	if err != nil {
		return err
	}

	var resp models.SendMailResponse
	if err := json.Unmarshal(body, &resp); err != nil {
		return fmt.Errorf("JSONデコード失敗: %w", err)
	}

	if resp.Error != "" {
		logger.Error("テンプレート保存失敗: %s", resp.Error)
		return fmt.Errorf("保存エラー: %s", resp.Error)
	}

	logger.Info("テンプレート保存成功")
	return nil
}
