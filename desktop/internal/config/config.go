// Package config は設定ファイルの読み書きを管理
package config

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path/filepath"
)

const (
	appName    = "auto-mail-pilot"
	configFile = "auto-mail-pilot-config.json"
)

// Config はアプリケーション設定
type Config struct {
	GASURL        string `json:"gas_url"`        // GAS WebアプリURL
	Signature     string `json:"signature"`      // デフォルト署名（プレーンテキスト）
	HtmlSignature string `json:"html_signature"` // HTML署名
	BasicAuthID   string `json:"basic_auth_id"`  // Basic認証ID
	BasicAuthPW   string `json:"basic_auth_pw"`  // Basic認証パスワード
}

// DefaultConfig はデフォルト設定を返す
func DefaultConfig() *Config {
	return &Config{
		GASURL:      "",
		Signature:   "",
		BasicAuthID: "",
		BasicAuthPW: "",
	}
}

// GetExeDir は実行ファイルのディレクトリを取得
func GetExeDir() (string, error) {
	exe, err := os.Executable()
	if err != nil {
		return "", fmt.Errorf("実行ファイルのパス取得に失敗: %w", err)
	}
	return filepath.Dir(exe), nil
}

// GetUserConfigDir はユーザー設定ディレクトリを取得（AppData/auto-mail-pilot）
func GetUserConfigDir() (string, error) {
	configDir, err := os.UserConfigDir()
	if err != nil {
		return "", fmt.Errorf("ユーザー設定ディレクトリ取得に失敗: %w", err)
	}
	return filepath.Join(configDir, appName), nil
}

// ensureUserConfigDir はユーザー設定ディレクトリを作成
func ensureUserConfigDir() (string, error) {
	dir, err := GetUserConfigDir()
	if err != nil {
		return "", err
	}
	if err := os.MkdirAll(dir, 0755); err != nil {
		return "", fmt.Errorf("設定ディレクトリ作成に失敗: %w", err)
	}
	return dir, nil
}

// Load は設定ファイルを読み込む
// 優先順位: 1.環境変数 2.ユーザー設定(AppData) 3.実行ディレクトリ(配布用)
func Load() (*Config, error) {
	config := DefaultConfig()

	// 1. 実行ディレクトリの設定を読み込み（配布用の初期設定）
	exeDir, err := GetExeDir()
	if err == nil {
		exeConfigPath := filepath.Join(exeDir, configFile)
		log.Printf("[CONFIG] exeディレクトリ設定パス: %s", exeConfigPath)
		if _, err := os.Stat(exeConfigPath); err == nil {
			data, err := os.ReadFile(exeConfigPath)
			if err != nil {
				log.Printf("[CONFIG] exeディレクトリ設定読み込み失敗: %v", err)
			} else {
				data = stripBOM(data)
				if jsonErr := json.Unmarshal(data, config); jsonErr != nil {
					log.Printf("[CONFIG] exeディレクトリ設定JSONパース失敗: %v (先頭: %q)", jsonErr, truncate(data, 100))
				} else {
					log.Printf("[CONFIG] exeディレクトリ設定読み込み成功: gas_url=%s", truncateStr(config.GASURL, 60))
				}
			}
		} else {
			log.Printf("[CONFIG] exeディレクトリ設定未検出: %s", exeConfigPath)
		}
	} else {
		log.Printf("[CONFIG] exeディレクトリ取得失敗: %v", err)
	}

	// 2. ユーザー設定ディレクトリの設定を読み込み（上書き）
	userConfigDir, err := GetUserConfigDir()
	if err == nil {
		userConfigPath := filepath.Join(userConfigDir, configFile)
		log.Printf("[CONFIG] ユーザー設定パス: %s", userConfigPath)
		if _, err := os.Stat(userConfigPath); err == nil {
			data, err := os.ReadFile(userConfigPath)
			if err != nil {
				log.Printf("[CONFIG] ユーザー設定読み込み失敗: %v", err)
			} else {
				data = stripBOM(data)
				var userConfig Config
				if jsonErr := json.Unmarshal(data, &userConfig); jsonErr != nil {
					log.Printf("[CONFIG] ユーザー設定JSONパース失敗: %v (先頭: %q)", jsonErr, truncate(data, 100))
				} else {
					log.Printf("[CONFIG] ユーザー設定読み込み成功")
					// ユーザー設定で上書き
					if userConfig.GASURL != "" {
						config.GASURL = userConfig.GASURL
					}
					if userConfig.Signature != "" {
						config.Signature = userConfig.Signature
					}
					if userConfig.HtmlSignature != "" {
						config.HtmlSignature = userConfig.HtmlSignature
					}
					if userConfig.BasicAuthID != "" {
						config.BasicAuthID = userConfig.BasicAuthID
					}
					if userConfig.BasicAuthPW != "" {
						config.BasicAuthPW = userConfig.BasicAuthPW
					}
				}
			}
		} else {
			log.Printf("[CONFIG] ユーザー設定未検出: %s", userConfigPath)
		}
	}

	// 3. 環境変数が設定されていれば上書き（最優先）
	if gasURL := os.Getenv("AUTO_MAIL_PILOT_GAS_URL"); gasURL != "" {
		config.GASURL = gasURL
	}

	return config, nil
}

// Save は設定ファイルをユーザー設定ディレクトリに保存
func Save(config *Config) error {
	userConfigDir, err := ensureUserConfigDir()
	if err != nil {
		return err
	}

	configPath := filepath.Join(userConfigDir, configFile)
	data, err := json.MarshalIndent(config, "", "  ")
	if err != nil {
		return fmt.Errorf("設定シリアライズに失敗: %w", err)
	}

	if err := os.WriteFile(configPath, data, 0644); err != nil {
		return fmt.Errorf("設定書き込みに失敗: %w", err)
	}

	return nil
}

// GetConfigPath はユーザー設定ファイルのパスを返す
func GetConfigPath() (string, error) {
	userConfigDir, err := GetUserConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(userConfigDir, configFile), nil
}

// stripBOM はUTF-8 BOM（0xEF 0xBB 0xBF）を除去する
func stripBOM(data []byte) []byte {
	return bytes.TrimPrefix(data, []byte{0xEF, 0xBB, 0xBF})
}

// truncate はバイト列を指定長で切り詰めて返す（ログ用）
func truncate(data []byte, maxLen int) []byte {
	if len(data) <= maxLen {
		return data
	}
	return data[:maxLen]
}

// truncateStr は文字列を指定長で切り詰めて返す（ログ用）
func truncateStr(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}
