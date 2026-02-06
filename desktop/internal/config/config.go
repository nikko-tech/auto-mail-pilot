// Package config は設定ファイルの読み書きを管理
package config

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

const (
	appName    = "auto-mail-pilot"
	configFile = "auto-mail-pilot-config.json"
)

// Config はアプリケーション設定
type Config struct {
	GASURL       string `json:"gas_url"`       // GAS WebアプリURL
	Signature    string `json:"signature"`     // デフォルト署名
	BasicAuthID  string `json:"basic_auth_id"` // Basic認証ID
	BasicAuthPW  string `json:"basic_auth_pw"` // Basic認証パスワード
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
		if _, err := os.Stat(exeConfigPath); err == nil {
			data, err := os.ReadFile(exeConfigPath)
			if err == nil {
				json.Unmarshal(data, config)
			}
		}
	}

	// 2. ユーザー設定ディレクトリの設定を読み込み（上書き）
	userConfigDir, err := GetUserConfigDir()
	if err == nil {
		userConfigPath := filepath.Join(userConfigDir, configFile)
		if _, err := os.Stat(userConfigPath); err == nil {
			data, err := os.ReadFile(userConfigPath)
			if err == nil {
				var userConfig Config
				if json.Unmarshal(data, &userConfig) == nil {
					// ユーザー設定で上書き
					if userConfig.GASURL != "" {
						config.GASURL = userConfig.GASURL
					}
					if userConfig.Signature != "" {
						config.Signature = userConfig.Signature
					}
					if userConfig.BasicAuthID != "" {
						config.BasicAuthID = userConfig.BasicAuthID
					}
					if userConfig.BasicAuthPW != "" {
						config.BasicAuthPW = userConfig.BasicAuthPW
					}
				}
			}
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
