<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  GetTemplates,
  GetRecipients,
  GetSignatures,
  GetHtmlSignature,
  SendMail,
  OpenFileDialog,
  ApplyTemplateVariables,
  MatchAllRecipientsByFileName,
  MatchTemplateByFileName,
  ValidateSendSafety,
  ReadFilesAsAttachments
} from '../../wailsjs/go/main/App'
import { EventsOn } from '../../wailsjs/runtime/runtime'

// EventsOnの解除関数を保持
let offFileDrop = null
let offFileDropGo = null

// データ
const templates = ref([])
const recipients = ref([])
const signature = ref('')
const htmlSignature = ref('')
const hasHtmlSignature = computed(() => htmlSignature.value !== '')

// 検索フィルター
const recipientSearch = ref('')
const templateSearch = ref('')

// 選択状態
const selectedRecipientIndex = ref(null)
const selectedTemplateIndex = ref(null)

// 宛先ロック状態（手動ロックのみ。ロック中はファイル追加時の自動切替を防ぐ）
const recipientLocked = ref(false)

// ファイル名マッチの候補ID一覧（ハイライト用）
const matchedRecipientIds = ref([])

// メール編集用（最大3宛先）
const activeRecipientTab = ref(0)
const mailData = ref([
  { email: '', subject: '', body: '' },
  { email: '', subject: '', body: '' },
  { email: '', subject: '', body: '' }
])

// 添付ファイル
const attachments = ref([])

// 状態
const isLoading = ref(false)
const statusMessage = ref('')
const showSignature = ref(true)

// 送信確認ダイアログ
const showConfirmDialog = ref(false)
const confirmationChecked = ref(false)
const validationErrors = ref([])
const showValidationErrors = ref(false)

// ドラッグ＆ドロップ
const isDragging = ref(false)
let isProcessingDrop = false // 二重処理防止フラグ

// フィルター済みリスト
const filteredRecipients = computed(() => {
  const search = recipientSearch.value.toLowerCase()
  if (!search) return recipients.value
  return recipients.value.filter(r =>
    r.name?.toLowerCase().includes(search) ||
    r.company?.toLowerCase().includes(search) ||
    r.email?.toLowerCase().includes(search)
  )
})

const filteredTemplates = computed(() => {
  const search = templateSearch.value.toLowerCase()
  if (!search) return templates.value
  return templates.value.filter(t =>
    t.name?.toLowerCase().includes(search) ||
    t.subject?.toLowerCase().includes(search)
  )
})

// 選択中の宛先
const selectedRecipient = computed(() => {
  if (selectedRecipientIndex.value === null) return null
  return recipients.value[selectedRecipientIndex.value]
})

// 選択中のテンプレート
const selectedTemplate = computed(() => {
  if (selectedTemplateIndex.value === null) return null
  return templates.value[selectedTemplateIndex.value]
})

// 初期化
onMounted(async () => {
  await loadData()

  // 1) Wails標準イベント（これが生きてれば最短）
  offFileDrop = EventsOn("wails:file-drop", (...args) => {
    console.log("【wails:file-drop】args=", args)
    // 仕様上は (x, y, paths) だが、念のため崩れにも耐える
    const paths = Array.isArray(args[2]) ? args[2] : (Array.isArray(args[0]) ? args[0] : null)
    if (Array.isArray(paths)) void handleFileDrop(paths)
  })
  console.log("EventsOn(wails:file-drop) 登録完了")

  // 2) Go側 OnFileDrop → EventsEmit のフォールバック
  offFileDropGo = EventsOn("onFileDrop", (paths) => {
    console.log("【onFileDrop from Go】paths=", paths)
    if (Array.isArray(paths)) void handleFileDrop(paths)
  })
  console.log("EventsOn(onFileDrop) 登録完了")
})

// クリーンアップ
onUnmounted(() => {
  if (offFileDrop) {
    offFileDrop()
    offFileDrop = null
  }
  if (offFileDropGo) {
    offFileDropGo()
    offFileDropGo = null
  }
})

// ファイルドロップ処理（Go側からのイベント経由）
async function handleFileDrop(paths) {
  console.log('onFileDropイベント受信:', paths)

  // 二重処理防止
  if (isProcessingDrop) {
    console.log('処理中のためスキップ')
    return
  }

  if (!paths || paths.length === 0) {
    console.log('パスが空です')
    return
  }

  isProcessingDrop = true
  isDragging.value = false
  statusMessage.value = 'ファイル読み込み中...'

  try {
    const files = await ReadFilesAsAttachments(paths)
    console.log('読み込まれたファイル:', files)
    if (files && files.length > 0) {
      attachments.value = [...attachments.value, ...files]
      statusMessage.value = `${files.length}件のファイルをドロップしました`

      // ファイル名から宛先・テンプレートを自動選択
      await autoSelectFromFileName(files[0].fileName)
    }
  } catch (e) {
    console.error('ファイル読み込みエラー:', e)
    statusMessage.value = 'ファイル読み込みエラー: ' + e
  } finally {
    isProcessingDrop = false
  }
}

// ファイル名から宛先・テンプレートを自動選択
async function autoSelectFromFileName(fileName) {
  // 宛先マッチング（ロック中はスキップ）
  if (!recipientLocked.value) {
    const matches = await MatchAllRecipientsByFileName(fileName, recipients.value)
    if (matches && matches.length > 0) {
      // マッチした全候補のIDを保存（ハイライト用）
      matchedRecipientIds.value = matches.map(r => r.id)

      // 1件なら自動選択、複数なら最初の1件を選択しつつユーザーに選ばせる
      const first = matches[0]
      const index = recipients.value.findIndex(r => r.id === first.id)
      if (index >= 0) {
        selectedRecipientIndex.value = index
        mailData.value[activeRecipientTab.value].email = first.email
      }

      if (matches.length === 1) {
        statusMessage.value = `宛先を自動選択: ${first.company} ${first.name}`
      } else {
        statusMessage.value = `${matches.length}件の候補が見つかりました（黄色ハイライト）。クリックで変更できます`
      }
    } else {
      matchedRecipientIds.value = []
    }
  }

  // テンプレートマッチング
  const matchedTemplate = await MatchTemplateByFileName(fileName, templates.value)
  if (matchedTemplate) {
    const index = templates.value.findIndex(t => t.id === matchedTemplate.id)
    if (index >= 0) {
      selectedTemplateIndex.value = index
      applyTemplate()
    }
  }
}

// データ読み込み
async function loadData() {
  isLoading.value = true
  statusMessage.value = 'データ読み込み中...'
  try {
    // テンプレート、宛先、署名、HTML署名を並列取得
    const [tplResult, rcpResult, sigResult, htmlSigResult] = await Promise.all([
      GetTemplates(),
      GetRecipients(),
      GetSignatures(),
      GetHtmlSignature()
    ])

    console.log('GetSignatures result:', sigResult)

    templates.value = tplResult || []
    recipients.value = rcpResult || []
    htmlSignature.value = htmlSigResult || ''

    // sigResult が配列でも { signatures: [...] } でも両対応
    const sigs = Array.isArray(sigResult)
      ? sigResult
      : (sigResult && Array.isArray(sigResult.signatures) ? sigResult.signatures : [])

    if (sigs.length > 0) {
      signature.value = sigs[0].content || ''
      console.log('署名を設定しました:', sigs[0].name)
    } else {
      console.warn('署名が0件（sigResult形式 or GAS側データを確認）')
    }

    // 最初のテンプレートを選択して適用
    if (templates.value.length > 0) {
      selectedTemplateIndex.value = 0
      await applyTemplate()
    }

    statusMessage.value = `${templates.value.length}件のテンプレート、${recipients.value.length}件の宛先、署名: ${sigs.length}件`
  } catch (e) {
    console.error('データ読み込みエラー:', e)
    statusMessage.value = 'データ読み込みエラー: ' + e
  } finally {
    isLoading.value = false
  }
}

// 宛先選択（常にクリック可能）
function selectRecipient(index) {
  const realIndex = recipients.value.findIndex(r => r.id === filteredRecipients.value[index].id)
  selectedRecipientIndex.value = realIndex
  const recipient = recipients.value[realIndex]

  // メールアドレスを設定
  mailData.value[activeRecipientTab.value].email = recipient.email

  // テンプレートが選択されている場合は変数置換を適用
  if (selectedTemplate.value) {
    applyTemplate()
  }

  statusMessage.value = `宛先を選択: ${recipient.company} ${recipient.name}`
}

// 宛先ロック（ファイル追加時の自動切替を防止。手動クリックは常に可能）
function lockRecipient() {
  if (!selectedRecipient.value) {
    statusMessage.value = 'ロックする宛先を選択してください'
    return
  }
  recipientLocked.value = true
  statusMessage.value = `宛先をロック: ${selectedRecipient.value.company} ${selectedRecipient.value.name}（ファイル追加時の自動切替を防止）`
}

// 宛先ロック解除
function unlockRecipient() {
  recipientLocked.value = false
  statusMessage.value = '宛先ロックを解除しました'
}

// テンプレート選択
function selectTemplate(index) {
  const realIndex = templates.value.findIndex(t => t.id === filteredTemplates.value[index].id)
  selectedTemplateIndex.value = realIndex
  applyTemplate()
}

// テンプレート適用
async function applyTemplate() {
  const template = selectedTemplate.value
  if (!template) return

  let subject = template.subject
  let body = template.body

  // 宛先が選択されている場合は変数置換
  if (selectedRecipient.value) {
    try {
      subject = await ApplyTemplateVariables(subject, selectedRecipient.value)
      body = await ApplyTemplateVariables(body, selectedRecipient.value)
    } catch (e) {
      console.error('変数置換エラー:', e)
    }
  }

  mailData.value[activeRecipientTab.value].subject = subject
  mailData.value[activeRecipientTab.value].body = body

  // テンプレートに署名があれば設定
  if (template.signature) {
    signature.value = template.signature
  }

  statusMessage.value = `テンプレート「${template.name}」を適用しました`
}

// ファイル選択
async function selectFiles() {
  try {
    const files = await OpenFileDialog()
    if (files && files.length > 0) {
      attachments.value = [...attachments.value, ...files]
      statusMessage.value = `${files.length}件のファイルを追加しました`

      // ファイル名から宛先・テンプレートを自動選択
      await autoSelectFromFileName(files[0].fileName)
    }
  } catch (e) {
    statusMessage.value = 'ファイル選択エラー: ' + e
  }
}

// 添付ファイル削除
function removeAttachment(index) {
  attachments.value.splice(index, 1)
}

// 添付ファイルの有効/無効切り替え
function toggleAttachment(index) {
  attachments.value[index].enabled = !attachments.value[index].enabled
}

// 送信前確認を開く
async function openSendConfirmation() {
  const mail = mailData.value[activeRecipientTab.value]

  if (!mail.email) {
    statusMessage.value = 'エラー: 宛先を入力してください'
    return
  }
  if (!mail.subject) {
    statusMessage.value = 'エラー: 件名を入力してください'
    return
  }

  // バリデーション実行
  const enabledAttachments = attachments.value.filter(a => a.enabled)
  try {
    const errors = await ValidateSendSafety(selectedRecipient.value, enabledAttachments, mail.body)
    validationErrors.value = errors || []
  } catch (e) {
    console.error('バリデーションエラー:', e)
    validationErrors.value = []
  }

  // エラーがある場合は警告を表示
  if (validationErrors.value.length > 0) {
    showValidationErrors.value = true
    return
  }

  // 確認ダイアログを開く
  confirmationChecked.value = false
  showConfirmDialog.value = true
}

// バリデーションエラーを確認済みにして送信確認へ
function acknowledgeErrors() {
  showValidationErrors.value = false
  confirmationChecked.value = false
  showConfirmDialog.value = true
}

// メール送信
async function sendMail() {
  const mail = mailData.value[activeRecipientTab.value]

  // 本文に署名を追加（HTML署名がある場合はGo側で処理するのでスキップ）
  let finalBody = mail.body
  if (!hasHtmlSignature.value && signature.value) {
    finalBody = mail.body + '\n\n' + signature.value
  }

  // 有効な添付ファイルのみ送信
  const enabledAttachments = attachments.value.filter(a => a.enabled)

  isLoading.value = true
  statusMessage.value = '送信中...'
  showConfirmDialog.value = false

  try {
    const result = await SendMail(mail.email, mail.subject, finalBody, enabledAttachments)
    if (result.success) {
      statusMessage.value = '✅ メール送信成功! 次のファイルを追加してください'
      // フォームを完全クリア
      mailData.value[activeRecipientTab.value] = { email: '', subject: '', body: '' }
      attachments.value = []
      selectedRecipientIndex.value = null
      selectedTemplateIndex.value = null
      matchedRecipientIds.value = []
      recipientLocked.value = false
      recipientSearch.value = ''
    } else {
      statusMessage.value = '❌ 送信失敗: ' + (result.error || '不明なエラー')
    }
  } catch (e) {
    statusMessage.value = '❌ 送信エラー: ' + e
  } finally {
    isLoading.value = false
  }
}

// フォームリセット
function resetForm() {
  mailData.value[activeRecipientTab.value] = { email: '', subject: '', body: '' }
  attachments.value = []
  selectedRecipientIndex.value = null
  selectedTemplateIndex.value = null
  matchedRecipientIds.value = []
  recipientLocked.value = false
  recipientSearch.value = ''
  statusMessage.value = 'フォームをリセットしました'
}
</script>

<template>
  <div
    class="mail-tab"
    :class="{ 'drag-over': isDragging }"
    @dragenter.prevent="isDragging = true"
    @dragover.prevent="isDragging = true"
    @dragleave.prevent="isDragging = false"
    @drop.prevent="isDragging = false"
  >
    <!-- ステータスバー -->
    <div class="status-bar" :class="{ loading: isLoading }">
      {{ statusMessage }}
    </div>

    <!-- 3列レイアウト -->
    <div class="three-column-layout">
      <!-- 左列: 宛先 + 添付ファイル -->
      <div class="left-column">
        <!-- 宛先選択 -->
        <div class="panel recipient-panel">
          <div class="panel-header">
            <h3>宛先</h3>
            <div class="lock-controls">
              <button
                v-if="!recipientLocked"
                class="lock-btn"
                :disabled="!selectedRecipient"
                @click="lockRecipient"
              >
                🔒 ロック
              </button>
              <button
                v-else
                class="unlock-btn"
                @click="unlockRecipient"
              >
                🔓 解除
              </button>
            </div>
          </div>
          <div v-if="recipientLocked && selectedRecipient" class="lock-indicator">
            🔒 {{ selectedRecipient.company }} {{ selectedRecipient.name }}（自動切替OFF）
          </div>
          <div v-if="matchedRecipientIds.length > 1" class="match-indicator">
            📋 {{ matchedRecipientIds.length }}件の候補 — クリックで選択してください
          </div>
          <input
            v-model="recipientSearch"
            type="text"
            placeholder="名前・会社名・メールで検索..."
            class="search-input"
          />
          <div class="selection-list">
            <div
              v-for="(recipient, index) in filteredRecipients"
              :key="recipient.id"
              :class="[
                'selection-item',
                { selected: recipients[selectedRecipientIndex]?.id === recipient.id },
                { matched: matchedRecipientIds.includes(recipient.id) }
              ]"
              @click="selectRecipient(index)"
            >
              <div class="item-main">{{ recipient.company }}</div>
              <div class="item-sub">{{ recipient.name }} &lt;{{ recipient.email }}&gt;</div>
            </div>
            <div v-if="filteredRecipients.length === 0" class="no-items">
              該当する宛先がありません
            </div>
          </div>
        </div>

        <!-- 添付ファイル -->
        <div
          class="panel attachments-panel"
          :class="{ 'drag-over': isDragging }"
        >
          <h3>添付ファイル</h3>
          <div class="drop-zone-hint" v-if="attachments.length === 0">
            ここにファイルをドラッグ＆ドロップ
          </div>
          <button class="add-file-btn" @click="selectFiles">+ ファイルを追加</button>
          <div class="attachments-list">
            <div
              v-for="(file, index) in attachments"
              :key="index"
              :class="['attachment-item', { disabled: !file.enabled }]"
            >
              <input
                type="checkbox"
                :checked="file.enabled"
                @change="toggleAttachment(index)"
              />
              <span class="file-name">{{ file.fileName }}</span>
              <button class="remove-btn" @click="removeAttachment(index)">×</button>
            </div>
          </div>
        </div>
      </div>

      <!-- 中央列: テンプレート + 署名 -->
      <div class="middle-column">
        <!-- テンプレート選択 -->
        <div class="panel template-panel">
          <h3>テンプレート</h3>
          <input
            v-model="templateSearch"
            type="text"
            placeholder="テンプレート名で検索..."
            class="search-input"
          />
          <div class="selection-list">
            <div
              v-for="(template, index) in filteredTemplates"
              :key="template.id"
              :class="['selection-item', { selected: templates[selectedTemplateIndex]?.id === template.id }]"
              @click="selectTemplate(index)"
            >
              <div class="item-main">{{ template.name }}</div>
              <div class="item-sub">{{ template.subject }}</div>
            </div>
            <div v-if="filteredTemplates.length === 0" class="no-items">
              該当するテンプレートがありません
            </div>
          </div>
        </div>

        <!-- 署名プレビュー -->
        <div class="panel signature-panel">
          <div class="panel-header">
            <h3>署名</h3>
            <button class="signature-toggle" @click="showSignature = !showSignature">
              {{ showSignature ? '▼' : '▶' }}
            </button>
          </div>
          <div v-if="showSignature && hasHtmlSignature" class="signature-preview signature-html" v-html="htmlSignature"></div>
          <pre v-else-if="showSignature" class="signature-preview">{{ signature || '（署名なし）' }}</pre>
        </div>
      </div>

      <!-- 右列: メール作成 -->
      <div class="right-column">
        <div class="panel mail-panel">
          <!-- 宛先タブ -->
          <div class="recipient-tabs">
            <button
              v-for="(_, index) in mailData"
              :key="index"
              :class="['recipient-tab', { active: activeRecipientTab === index }]"
              @click="activeRecipientTab = index"
            >
              宛先{{ index + 1 }}
              <span v-if="mailData[index].email" class="has-data">●</span>
            </button>
            <button class="reset-btn" @click="resetForm">リセット</button>
          </div>

          <!-- メール編集フォーム -->
          <div class="mail-form">
            <div class="form-row">
              <label>宛先:</label>
              <div class="email-row">
                <input
                  v-model="mailData[activeRecipientTab].email"
                  type="email"
                  placeholder="メールアドレス"
                  class="email-input"
                />
              </div>
            </div>
            <div v-if="selectedRecipient" class="recipient-info-row">
              <span v-if="recipientLocked">🔒 </span>
              {{ selectedRecipient.company }} / {{ selectedRecipient.name }}
            </div>

            <div class="form-row">
              <label>件名:</label>
              <input
                v-model="mailData[activeRecipientTab].subject"
                type="text"
                placeholder="件名を入力"
                class="subject-input"
              />
            </div>

            <div class="form-row body-row">
              <label>本文:</label>
              <textarea
                v-model="mailData[activeRecipientTab].body"
                placeholder="本文を入力"
                class="body-input"
              ></textarea>
            </div>
          </div>

          <!-- 送信ボタン -->
          <button
            class="send-btn"
            :disabled="isLoading || !mailData[activeRecipientTab].email"
            @click="openSendConfirmation"
          >
            {{ isLoading ? '送信中...' : '✉ 送信確認' }}
          </button>
        </div>
      </div>
    </div>

    <!-- バリデーションエラーダイアログ -->
    <div v-if="showValidationErrors" class="modal-overlay">
      <div class="modal validation-modal">
        <h3>⚠️ 送信前の確認</h3>
        <div class="validation-errors">
          <div v-for="(error, index) in validationErrors" :key="index" class="validation-error">
            {{ error }}
          </div>
        </div>
        <div class="modal-actions">
          <button class="cancel-btn" @click="showValidationErrors = false">キャンセル</button>
          <button class="warning-btn" @click="acknowledgeErrors">確認しました（送信へ進む）</button>
        </div>
      </div>
    </div>

    <!-- 送信確認ダイアログ -->
    <div v-if="showConfirmDialog" class="modal-overlay">
      <div class="modal confirm-modal">
        <h3>📧 送信内容の確認</h3>
        <div class="confirm-details">
          <div class="confirm-row">
            <span class="confirm-label">宛先:</span>
            <span class="confirm-value">{{ mailData[activeRecipientTab].email }}</span>
          </div>
          <div v-if="selectedRecipient" class="confirm-row">
            <span class="confirm-label">会社名:</span>
            <span class="confirm-value">{{ selectedRecipient.company }}</span>
          </div>
          <div v-if="selectedRecipient" class="confirm-row">
            <span class="confirm-label">氏名:</span>
            <span class="confirm-value">{{ selectedRecipient.name }}</span>
          </div>
          <div class="confirm-row">
            <span class="confirm-label">件名:</span>
            <span class="confirm-value">{{ mailData[activeRecipientTab].subject }}</span>
          </div>
          <div class="confirm-row">
            <span class="confirm-label">添付:</span>
            <span class="confirm-value">
              {{ attachments.filter(a => a.enabled).length }}件
              <span v-if="attachments.filter(a => a.enabled).length > 0">
                ({{ attachments.filter(a => a.enabled).map(a => a.fileName).join(', ') }})
              </span>
            </span>
          </div>
        </div>

        <div class="confirm-checkbox">
          <label>
            <input type="checkbox" v-model="confirmationChecked" />
            宛先・添付ファイルが正しいことを確認しました
          </label>
        </div>

        <div class="modal-actions">
          <button class="cancel-btn" @click="showConfirmDialog = false">キャンセル</button>
          <button
            class="send-confirm-btn"
            :disabled="!confirmationChecked"
            @click="sendMail"
          >
            送信する
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mail-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 12px;
  transition: background 0.2s ease;
  --wails-drop-target: drop;
}

.mail-tab.drag-over {
  background: rgba(25, 118, 210, 0.05);
}

.status-bar {
  padding: 8px 16px;
  background: #e3f2fd;
  border-radius: 8px;
  font-size: 0.9rem;
  color: #1565c0;
  flex-shrink: 0;
}

.status-bar.loading {
  background: #fff3e0;
  color: #ef6c00;
}

/* 3列レイアウト */
.three-column-layout {
  display: flex;
  gap: 12px;
  flex: 1;
  min-height: 0;
}

.left-column {
  width: 280px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.middle-column {
  width: 280px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.right-column {
  flex: 1;
  min-width: 400px;
  display: flex;
  flex-direction: column;
}

/* パネル共通 */
.panel {
  background: white;
  border-radius: 8px;
  padding: 12px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.panel h3 {
  margin: 0 0 8px 0;
  font-size: 0.95rem;
  color: #333;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.panel-header h3 {
  margin: 0;
}

/* 宛先パネル */
.recipient-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 200px;
}

.lock-controls {
  display: flex;
  gap: 4px;
}

.lock-btn, .unlock-btn {
  padding: 4px 8px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
}

.lock-btn {
  background: #667eea;
  color: white;
}

.lock-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.unlock-btn {
  background: #ff9800;
  color: white;
}

.lock-indicator {
  padding: 4px 8px;
  background: #fff3e0;
  border: 1px solid #ff9800;
  border-radius: 4px;
  color: #e65100;
  font-size: 0.8rem;
  margin-bottom: 8px;
}

.search-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  margin-bottom: 8px;
  font-size: 0.85rem;
  box-sizing: border-box;
}

.search-input:disabled {
  background: #f5f5f5;
}

.selection-list {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #eee;
  border-radius: 4px;
  min-height: 100px;
}

.selection-item {
  padding: 8px 10px;
  cursor: pointer;
  border-bottom: 1px solid #f5f5f5;
}

.selection-item:hover {
  background: #f5f5f5;
}

.selection-item.selected {
  background: #e3f2fd;
  border-left: 3px solid #1976d2;
}

.selection-item.matched {
  background: #fff8e1;
  border-left: 3px solid #ffa000;
}

.selection-item.matched.selected {
  background: #e3f2fd;
  border-left: 3px solid #1976d2;
}

.match-indicator {
  padding: 4px 8px;
  background: #fff8e1;
  border: 1px solid #ffa000;
  border-radius: 4px;
  color: #e65100;
  font-size: 0.8rem;
  margin-bottom: 8px;
}

.item-main {
  font-weight: 500;
  color: #333;
  font-size: 0.9rem;
}

.item-sub {
  font-size: 0.8rem;
  color: #666;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.no-items {
  padding: 16px;
  text-align: center;
  color: #999;
  font-size: 0.85rem;
}

/* 添付ファイルパネル */
.attachments-panel {
  flex-shrink: 0;
  transition: all 0.2s ease;
  border: 2px dashed transparent;
  --wails-drop-target: drop;
}

.attachments-panel.drag-over {
  background: #e3f2fd;
  border-color: #1976d2;
}

.drop-zone-hint {
  padding: 16px;
  text-align: center;
  color: #999;
  font-size: 0.8rem;
  border: 2px dashed #ddd;
  border-radius: 8px;
  margin-bottom: 8px;
  background: #fafafa;
  --wails-drop-target: drop;
}

.add-file-btn {
  width: 100%;
  padding: 6px 12px;
  background: #f5f5f5;
  border: 1px dashed #ccc;
  border-radius: 4px;
  cursor: pointer;
  margin-bottom: 8px;
  font-size: 0.85rem;
}

.add-file-btn:hover {
  background: #eee;
}

.attachments-list {
  max-height: 120px;
  overflow-y: auto;
}

.attachment-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  background: #f5f5f5;
  border-radius: 4px;
  margin-bottom: 4px;
  font-size: 0.85rem;
}

.attachment-item.disabled {
  opacity: 0.5;
}

.file-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.remove-btn {
  background: #ff5252;
  color: white;
  border: none;
  border-radius: 50%;
  width: 18px;
  height: 18px;
  cursor: pointer;
  font-size: 0.75rem;
  line-height: 1;
}

/* テンプレートパネル */
.template-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 200px;
}

/* 署名パネル */
.signature-panel {
  flex-shrink: 0;
}

.signature-toggle {
  background: none;
  border: none;
  color: #666;
  cursor: pointer;
  font-size: 0.85rem;
  padding: 0;
}

.signature-preview {
  margin: 8px 0 0 0;
  padding: 8px;
  background: #f9f9f9;
  border-radius: 4px;
  font-size: 0.8rem;
  white-space: pre-wrap;
  color: #666;
  max-height: 150px;
  overflow-y: auto;
}

.signature-html {
  white-space: normal;
  color: inherit;
  border: 1px solid #e0e0e0;
}

/* メールパネル */
.mail-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.recipient-tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.recipient-tab {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: #f5f5f5;
  cursor: pointer;
  font-size: 0.85rem;
  position: relative;
}

.recipient-tab.active {
  background: #667eea;
  color: white;
  border-color: #667eea;
}

.recipient-tab .has-data {
  color: #4caf50;
  font-size: 0.6rem;
  margin-left: 4px;
}

.reset-btn {
  margin-left: auto;
  padding: 6px 12px;
  background: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 4px;
  cursor: pointer;
  color: #666;
  font-size: 0.85rem;
}

.reset-btn:hover {
  background: #eee;
}

.mail-form {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.form-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 8px;
  flex-shrink: 0;
}

.form-row label {
  width: 50px;
  padding-top: 6px;
  font-weight: 500;
  color: #333;
  font-size: 0.85rem;
  flex-shrink: 0;
}

.email-row {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
}

.email-input,
.subject-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.9rem;
}

.email-input.locked {
  background: #fff3e0;
  border-color: #ff9800;
}

.recipient-info-row {
  font-size: 0.8rem;
  color: #666;
  padding: 4px 0 8px 58px;
  flex-shrink: 0;
}

.recipient-info-row.locked {
  color: #e65100;
  font-weight: 500;
}

.body-row {
  flex: 1;
  min-height: 0;
}

.body-input {
  flex: 1;
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  resize: none;
  font-family: inherit;
  font-size: 0.9rem;
  box-sizing: border-box;
}

.send-btn {
  margin-top: 12px;
  padding: 12px 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
  flex-shrink: 0;
}

.send-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* モーダル */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal {
  background: white;
  border-radius: 12px;
  padding: 24px;
  max-width: 500px;
  width: 90%;
  max-height: 80vh;
  overflow-y: auto;
}

.modal h3 {
  margin-bottom: 16px;
  color: #333;
}

.validation-modal {
  border-top: 4px solid #ff9800;
}

.validation-errors {
  background: #fff3e0;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.validation-error {
  padding: 8px 0;
  color: #e65100;
  border-bottom: 1px solid #ffe0b2;
}

.validation-error:last-child {
  border-bottom: none;
}

.confirm-modal {
  border-top: 4px solid #667eea;
}

.confirm-details {
  background: #f5f5f5;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.confirm-row {
  display: flex;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid #eee;
}

.confirm-row:last-child {
  border-bottom: none;
}

.confirm-label {
  width: 80px;
  color: #666;
  font-weight: 500;
}

.confirm-value {
  flex: 1;
  color: #333;
}

.confirm-checkbox {
  margin-bottom: 16px;
  padding: 12px;
  background: #e3f2fd;
  border-radius: 8px;
}

.confirm-checkbox label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  color: #1565c0;
  font-weight: 500;
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.cancel-btn {
  padding: 10px 24px;
  background: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 8px;
  cursor: pointer;
  color: #666;
}

.cancel-btn:hover {
  background: #eee;
}

.warning-btn {
  padding: 10px 24px;
  background: #ff9800;
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
}

.warning-btn:hover {
  background: #f57c00;
}

.send-confirm-btn {
  padding: 10px 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
}

.send-confirm-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.send-confirm-btn:hover:not(:disabled) {
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.4);
}
</style>
