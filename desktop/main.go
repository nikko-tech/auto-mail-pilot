package main

import (
	"embed"

	"desktop/internal/logger"

	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"
	"github.com/wailsapp/wails/v2/pkg/options/windows"
)

//go:embed all:frontend/dist
var assets embed.FS

func main() {
	// アプリインスタンス作成
	app := NewApp()

	// アプリケーションタイトル（バージョン付き）
	title := "Auto Mail Pilot " + logger.Version

	// Wailsアプリケーション起動
	err := wails.Run(&options.App{
		Title:     title,
		Width:     1200,
		Height:    800,
		MinWidth:  800,
		MinHeight: 600,
		AssetServer: &assetserver.Options{
			Assets: assets,
		},
		BackgroundColour: &options.RGBA{R: 255, G: 255, B: 255, A: 1},

		// ライフサイクルフック
		OnStartup:  app.startup,
		OnDomReady: app.domReady,
		OnShutdown: app.shutdown,

		Bind: []interface{}{
			app,
		},

		// ドラッグ＆ドロップは無効（Wailsのバグで動作しないため）
		// ファイル追加は「+ ファイルを追加」ボタンから

		// Windows固有設定
		Windows: &windows.Options{
			WebviewIsTransparent: false,
			WindowIsTranslucent:  false,
			DisableWindowIcon:    false,
		},
	})

	if err != nil {
		println("エラー:", err.Error())
	}
}
