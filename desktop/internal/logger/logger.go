// Package logger はログ出力を管理（標準出力+ファイル両方に出力）
package logger

import (
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"runtime"
	"time"

	"desktop/internal/config"
)

var (
	// Version はビルド時に埋め込まれるバージョン
	Version = "v0.3"
	// BuildTime はビルド時に埋め込まれるビルド日時
	BuildTime = "unknown"
	// CommitHash はビルド時に埋め込まれるコミットハッシュ
	CommitHash = "unknown"
)

// Logger はアプリケーションのロガー
type Logger struct {
	infoLog  *log.Logger
	errorLog *log.Logger
	logFile  *os.File
}

var globalLogger *Logger

// Init はロガーを初期化
func Init() error {
	exeDir, err := config.GetExeDir()
	if err != nil {
		return fmt.Errorf("実行ディレクトリ取得に失敗: %w", err)
	}

	// logsディレクトリを作成
	logsDir := filepath.Join(exeDir, "logs")
	if err := os.MkdirAll(logsDir, 0755); err != nil {
		return fmt.Errorf("logsディレクトリ作成に失敗: %w", err)
	}

	// ログファイル名を生成（YYYYMMDD_HHMMSS.log）
	now := time.Now()
	logFileName := fmt.Sprintf("%s.log", now.Format("20060102_150405"))
	logFilePath := filepath.Join(logsDir, logFileName)

	// ログファイルを作成
	logFile, err := os.OpenFile(logFilePath, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
	if err != nil {
		return fmt.Errorf("ログファイル作成に失敗: %w", err)
	}

	// 標準出力とファイル両方に出力
	multiWriter := io.MultiWriter(os.Stdout, logFile)

	globalLogger = &Logger{
		infoLog:  log.New(multiWriter, "[INFO] ", log.Ldate|log.Ltime),
		errorLog: log.New(multiWriter, "[ERROR] ", log.Ldate|log.Ltime|log.Lshortfile),
		logFile:  logFile,
	}

	// 起動ログを出力
	logStartupInfo()

	return nil
}

// logStartupInfo は起動時の情報をログに出力
func logStartupInfo() {
	Info("========================================")
	Info("アプリケーション起動")
	Info("バージョン: %s", Version)
	Info("ビルド日時: %s", BuildTime)
	Info("コミットハッシュ: %s", CommitHash)
	Info("OS/ARCH: %s/%s", runtime.GOOS, runtime.GOARCH)
	Info("起動時刻: %s", time.Now().Format("2006-01-02 15:04:05"))
	Info("実行引数: %v", os.Args)

	// config.jsonの読み込み元をログ
	configPath, err := config.GetConfigPath()
	if err == nil {
		if _, statErr := os.Stat(configPath); statErr == nil {
			Info("設定ファイル: %s", configPath)
		} else {
			Info("設定ファイル: なし（デフォルト設定を使用）")
		}
	}
	Info("========================================")
}

// Close はロガーを終了
func Close() {
	if globalLogger != nil && globalLogger.logFile != nil {
		globalLogger.logFile.Close()
	}
}

// Info は情報ログを出力
func Info(format string, v ...interface{}) {
	if globalLogger != nil {
		globalLogger.infoLog.Printf(format, v...)
	} else {
		log.Printf("[INFO] "+format, v...)
	}
}

// Error はエラーログを出力
func Error(format string, v ...interface{}) {
	if globalLogger != nil {
		globalLogger.errorLog.Printf(format, v...)
	} else {
		log.Printf("[ERROR] "+format, v...)
	}
}

// GetVersion はバージョン情報を返す
func GetVersion() string {
	return Version
}

// GetBuildInfo はビルド情報を返す
func GetBuildInfo() map[string]string {
	return map[string]string{
		"version":    Version,
		"buildTime":  BuildTime,
		"commitHash": CommitHash,
		"os":         runtime.GOOS,
		"arch":       runtime.GOARCH,
	}
}
