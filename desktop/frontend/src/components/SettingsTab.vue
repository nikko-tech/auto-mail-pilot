<script setup>
import { ref, onMounted } from 'vue'
import { GetConfig, SaveConfigWithAuth, TestConnection, GetVersion } from '../../wailsjs/go/main/App'

const gasUrl = ref('')
const signature = ref('')
const basicAuthId = ref('')
const basicAuthPw = ref('')
const showPassword = ref(false)
const isLoading = ref(false)
const statusMessage = ref('')
const buildInfo = ref({})

// 初期化
onMounted(async () => {
  try {
    const config = await GetConfig()
    gasUrl.value = config.gas_url || ''
    signature.value = config.signature || ''
    basicAuthId.value = config.basic_auth_id || ''
    basicAuthPw.value = config.basic_auth_pw || ''

    const info = await GetVersion()
    buildInfo.value = info
  } catch (e) {
    statusMessage.value = '設定読み込みエラー: ' + e
  }
})

// 接続テスト
async function testConnection() {
  if (!gasUrl.value) {
    statusMessage.value = 'GAS URLを入力してください'
    return
  }

  isLoading.value = true
  statusMessage.value = '接続テスト中...'

  try {
    // まず保存してからテスト
    await SaveConfigWithAuth(gasUrl.value, signature.value, basicAuthId.value, basicAuthPw.value)
    const result = await TestConnection()
    if (result.success) {
      statusMessage.value = '✅ 接続成功!'
    } else {
      statusMessage.value = '❌ 接続失敗: ' + (result.error || '不明なエラー')
    }
  } catch (e) {
    statusMessage.value = '❌ エラー: ' + e
  } finally {
    isLoading.value = false
  }
}

// 設定保存
async function saveSettings() {
  isLoading.value = true
  statusMessage.value = '保存中...'

  try {
    await SaveConfigWithAuth(gasUrl.value, signature.value, basicAuthId.value, basicAuthPw.value)
    statusMessage.value = '✅ 設定を保存しました'
  } catch (e) {
    statusMessage.value = '❌ 保存エラー: ' + e
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="settings-tab">
    <h2>設定</h2>

    <!-- ステータス -->
    <div v-if="statusMessage" class="status-message">
      {{ statusMessage }}
    </div>

    <!-- GAS設定 -->
    <div class="settings-section">
      <h3>GAS連携設定</h3>
      <div class="form-group">
        <label>GAS WebアプリURL:</label>
        <input
          v-model="gasUrl"
          type="url"
          placeholder="https://script.google.com/macros/s/..."
          class="url-input"
        />
        <button
          class="test-btn"
          :disabled="isLoading"
          @click="testConnection"
        >
          接続テスト
        </button>
      </div>
      <p class="hint">
        Google Apps ScriptのWebアプリとしてデプロイしたURLを入力してください。
      </p>
    </div>

    <!-- Basic認証設定 -->
    <div class="settings-section">
      <h3>Basic認証設定</h3>
      <div class="form-group">
        <label>認証ID:</label>
        <input
          v-model="basicAuthId"
          type="text"
          placeholder="認証ID"
          class="auth-input"
        />
      </div>
      <div class="form-group">
        <label>認証パスワード:</label>
        <div class="password-row">
          <input
            v-model="basicAuthPw"
            :type="showPassword ? 'text' : 'password'"
            placeholder="認証パスワード"
            class="auth-input"
          />
          <button
            type="button"
            class="toggle-pw-btn"
            @click="showPassword = !showPassword"
          >
            {{ showPassword ? '隠す' : '表示' }}
          </button>
        </div>
      </div>
      <p class="hint">
        Basic認証が必要な場合のみ入力してください。認証情報はconfig.jsonに保存されます。
      </p>
    </div>

    <!-- 署名設定 -->
    <div class="settings-section">
      <h3>署名設定</h3>
      <div class="form-group">
        <label>デフォルト署名:</label>
        <textarea
          v-model="signature"
          placeholder="メール末尾に自動挿入される署名を入力"
          class="signature-input"
        ></textarea>
      </div>
    </div>

    <!-- 保存ボタン -->
    <div class="actions">
      <button
        class="save-btn"
        :disabled="isLoading"
        @click="saveSettings"
      >
        {{ isLoading ? '保存中...' : '設定を保存' }}
      </button>
    </div>

    <!-- ビルド情報 -->
    <div class="settings-section build-info">
      <h3>アプリ情報</h3>
      <div class="info-grid">
        <div class="info-row">
          <span class="info-label">バージョン:</span>
          <span class="info-value">{{ buildInfo.version }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">ビルド日時:</span>
          <span class="info-value">{{ buildInfo.buildTime }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">OS / ARCH:</span>
          <span class="info-value">{{ buildInfo.os }} / {{ buildInfo.arch }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-tab {
  max-width: 800px;
}

.settings-tab h2 {
  font-size: 1.5rem;
  color: #333;
  margin-bottom: 24px;
}

.status-message {
  padding: 12px 16px;
  background: #e3f2fd;
  border-radius: 8px;
  margin-bottom: 16px;
  color: #1565c0;
}

.settings-section {
  background: white;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 16px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.settings-section h3 {
  font-size: 1.1rem;
  color: #333;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #eee;
}

.form-group {
  margin-bottom: 12px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: #333;
}

.url-input,
.auth-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.95rem;
  margin-bottom: 8px;
}

.password-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.password-row .auth-input {
  flex: 1;
  margin-bottom: 0;
}

.toggle-pw-btn {
  padding: 10px 16px;
  background: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
}

.toggle-pw-btn:hover {
  background: #eee;
}

.test-btn {
  padding: 8px 16px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.test-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.hint {
  font-size: 0.85rem;
  color: #666;
  margin-top: 8px;
}

.signature-input {
  width: 100%;
  min-height: 150px;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-family: inherit;
  resize: vertical;
}

.actions {
  margin-top: 24px;
}

.save-btn {
  padding: 12px 32px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.build-info {
  background: #f9f9f9;
}

.info-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-row {
  display: flex;
  gap: 16px;
}

.info-label {
  width: 120px;
  color: #666;
}

.info-value {
  color: #333;
}
</style>
