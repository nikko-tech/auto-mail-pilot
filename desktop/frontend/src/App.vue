<script setup>
import { ref, onMounted } from 'vue'
import MailTab from './components/MailTab.vue'
import HistoryTab from './components/HistoryTab.vue'
import SettingsTab from './components/SettingsTab.vue'
import { GetVersion, TestConnection } from '../wailsjs/go/main/App'

const activeTab = ref('mail')
const version = ref('')
const connectionStatus = ref('')

const tabs = [
  { id: 'mail', label: 'メール作成', icon: '✉' },
  { id: 'history', label: '送信履歴', icon: '📜' },
  { id: 'settings', label: '設定', icon: '⚙' }
]

onMounted(async () => {
  try {
    const info = await GetVersion()
    version.value = info.version

    // 起動時に接続テスト
    const result = await TestConnection()
    if (result.success) {
      connectionStatus.value = '接続成功'
    } else {
      connectionStatus.value = result.error || '接続失敗'
    }
  } catch (e) {
    console.error('初期化エラー:', e)
    connectionStatus.value = 'エラー'
  }
})
</script>

<template>
  <div class="app-container">
    <!-- ヘッダー -->
    <header class="app-header">
      <h1 class="app-title">Auto Mail Pilot</h1>
      <div class="header-info">
        <span class="version">{{ version }}</span>
        <span :class="['connection-status', connectionStatus === '接続成功' ? 'success' : 'error']">
          {{ connectionStatus }}
        </span>
      </div>
    </header>

    <!-- タブナビゲーション -->
    <nav class="tab-nav">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['tab-button', { active: activeTab === tab.id }]"
        @click="activeTab = tab.id"
      >
        <span class="tab-icon">{{ tab.icon }}</span>
        <span class="tab-label">{{ tab.label }}</span>
      </button>
    </nav>

    <!-- タブコンテンツ -->
    <main class="tab-content">
      <MailTab v-if="activeTab === 'mail'" />
      <HistoryTab v-else-if="activeTab === 'history'" />
      <SettingsTab v-else-if="activeTab === 'settings'" />
    </main>
  </div>
</template>

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: 'Segoe UI', 'Yu Gothic UI', 'Meiryo', sans-serif;
  background: #f5f5f5;
}

.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.app-title {
  font-size: 1.5rem;
  font-weight: 600;
}

.header-info {
  display: flex;
  gap: 16px;
  align-items: center;
}

.version {
  font-size: 0.85rem;
  opacity: 0.8;
}

.connection-status {
  font-size: 0.85rem;
  padding: 4px 12px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.2);
}

.connection-status.success {
  background: rgba(76, 175, 80, 0.8);
}

.connection-status.error {
  background: rgba(244, 67, 54, 0.8);
}

.tab-nav {
  display: flex;
  background: white;
  border-bottom: 1px solid #e0e0e0;
  padding: 0 16px;
}

.tab-button {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 24px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 1rem;
  color: #666;
  transition: all 0.2s;
  border-bottom: 3px solid transparent;
}

.tab-button:hover {
  background: #f5f5f5;
  color: #333;
}

.tab-button.active {
  color: #667eea;
  border-bottom-color: #667eea;
  font-weight: 600;
}

.tab-icon {
  font-size: 1.1rem;
}

.tab-content {
  flex: 1;
  overflow: auto;
  padding: 20px;
}
</style>
