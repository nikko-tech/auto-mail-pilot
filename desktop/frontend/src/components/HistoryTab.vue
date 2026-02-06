<script setup>
import { ref } from 'vue'

// 送信履歴（将来的にはGASから取得）
const history = ref([])
const isLoading = ref(false)

// 現状は履歴機能未実装のためプレースホルダー表示
</script>

<template>
  <div class="history-tab">
    <div class="history-header">
      <h2>送信履歴</h2>
      <button class="refresh-btn" :disabled="isLoading">
        {{ isLoading ? '読み込み中...' : '更新' }}
      </button>
    </div>

    <div class="history-content">
      <div v-if="history.length === 0" class="empty-state">
        <div class="empty-icon">📭</div>
        <p>送信履歴はありません</p>
        <p class="empty-hint">メールを送信すると、ここに履歴が表示されます</p>
      </div>

      <div v-else class="history-list">
        <div v-for="item in history" :key="item.id" class="history-item">
          <div class="history-date">{{ item.date }}</div>
          <div class="history-to">{{ item.to }}</div>
          <div class="history-subject">{{ item.subject }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-tab {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.history-header h2 {
  font-size: 1.25rem;
  color: #333;
}

.refresh-btn {
  padding: 8px 16px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.history-content {
  flex: 1;
  background: white;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  overflow: auto;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #666;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 16px;
}

.empty-hint {
  font-size: 0.9rem;
  color: #999;
  margin-top: 8px;
}

.history-list {
  padding: 16px;
}

.history-item {
  display: flex;
  gap: 16px;
  padding: 12px;
  border-bottom: 1px solid #eee;
}

.history-item:last-child {
  border-bottom: none;
}

.history-date {
  width: 120px;
  color: #666;
  font-size: 0.9rem;
}

.history-to {
  width: 200px;
  font-weight: 500;
}

.history-subject {
  flex: 1;
  color: #333;
}
</style>
