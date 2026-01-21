function doGet(e) {
  const action = e.parameter.action || 'getTemplates';

  if (action === 'getTemplates') {
    return getTemplates();
  } else if (action === 'getRecipients') {
    return getRecipients();
  } else if (action === 'getSignatures') {
    return getSignatures();
  } else if (action === 'getLinkings') {
    return getLinkings();
  }

  return ContentService.createTextOutput(JSON.stringify({ error: 'Unknown action' }))
    .setMimeType(ContentService.MimeType.JSON);
}

function getTemplates() {
  const ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName('テンプレート');
  
  if (!sheet) {
    sheet = ss.insertSheet('テンプレート');
    sheet.appendRow(['Name', 'Subject', 'Body']);
    sheet.appendRow(['Greeting', 'Hello', 'Hi {{name}},\n\nHow are you?']);
  }

  const data = sheet.getDataRange().getValues();
  const templates = [];
  
  for (let i = 1; i < data.length; i++) {
    templates.push({
      id: String(i + 1),
      name: data[i][0],
      subject: data[i][1],
      body: data[i][2]
    });
  }

  return ContentService.createTextOutput(JSON.stringify({ templates: templates }))
    .setMimeType(ContentService.MimeType.JSON);
}

function getRecipients() {
  const ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName('宛先リスト');
  
  if (!sheet) {
    // 宛先リストがない場合は新規作成（デモデータ付き）
    sheet = ss.insertSheet('宛先リスト');
    sheet.appendRow(['ID', '会社名', '氏名', 'メールアドレス']);
    sheet.appendRow(['1', 'Sample Corp', '田中 太郎 様', 'info@example.com']);
  }

  const data = sheet.getDataRange().getValues();
  const recipients = [];
  
  // ヘッダー行をスキップ
  for (let i = 1; i < data.length; i++) {
    recipients.push({
      id: String(data[i][0] || i + 1), // ID列が空なら行番号
      company: data[i][1] || "",
      name: data[i][2] || "",
      email: data[i][3] || ""
    });
  }

  return ContentService.createTextOutput(JSON.stringify({ recipients: recipients }))
    .setMimeType(ContentService.MimeType.JSON);
}

function getSignatures() {
  const ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName('署名');
  
  if (!sheet) {
    sheet = ss.insertSheet('署名');
    sheet.appendRow(['名前', '署名内容']);
    sheet.appendRow(['デフォルト', '--\n株式会社サンプル\n田中 太郎\ninfo@example.com']);
  }

  const data = sheet.getDataRange().getValues();
  const signatures = [];
  
  for (let i = 1; i < data.length; i++) {
    signatures.push({
      name: data[i][0],
      content: data[i][1]
    });
  }

  return ContentService.createTextOutput(JSON.stringify({ signatures: signatures }))
    .setMimeType(ContentService.MimeType.JSON);
}

function getLinkings() {
  const ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName('紐付けマスター');
  
  if (!sheet) {
    sheet = ss.insertSheet('紐付けマスター');
    sheet.appendRow(['宛先ID', 'テンプレートID']);
    sheet.appendRow(['1', '2']);  // Sample: Recipient ID 1 linked to Template ID 2
  }

  const data = sheet.getDataRange().getValues();
  const linkings = [];
  
  for (let i = 1; i < data.length; i++) {
    linkings.push({
      recipient_id: String(data[i][0]),
      template_id: String(data[i][1])
    });
  }

  return ContentService.createTextOutput(JSON.stringify({ linkings: linkings }))
    .setMimeType(ContentService.MimeType.JSON);
}

function doPost(e) {
  let payload;
  try {
     payload = JSON.parse(e.postData.contents);
  } catch (err) {
     return ContentService.createTextOutput(JSON.stringify({ success: false, error: "Invalid JSON" }))
       .setMimeType(ContentService.MimeType.JSON);
  }

  const action = payload.action;

  if (action === 'sendMail') {
    return sendMail(payload);
  } else if (action === 'sendBatchMail') {
    return sendBatchMail(payload);
  }

  return ContentService.createTextOutput(JSON.stringify({ error: 'Unknown action' }))
    .setMimeType(ContentService.MimeType.JSON);
}

function sendMail(payload) {
  try {
    GmailApp.sendEmail(payload.to, payload.subject, payload.body);
    return ContentService.createTextOutput(JSON.stringify({ success: true }))
      .setMimeType(ContentService.MimeType.JSON);
  } catch (error) {
    return ContentService.createTextOutput(JSON.stringify({
      success: false,
      error: error.toString()
    }))
      .setMimeType(ContentService.MimeType.JSON);
  }
}

function sendBatchMail(payload) {
  const results = [];
  const emails = payload.emails; // Array of {to, subject, body}
  
  if (!Array.isArray(emails)) {
    return ContentService.createTextOutput(JSON.stringify({ success: false, error: "Emails should be an array" }))
      .setMimeType(ContentService.MimeType.JSON);
  }

  for (const email of emails) {
    try {
      if (email.to && email.subject && email.body) {
        GmailApp.sendEmail(email.to, email.subject, email.body);
        results.push({ to: email.to, success: true });
      }
    } catch (error) {
      results.push({ to: email.to, success: false, error: error.toString() });
    }
  }

  return ContentService.createTextOutput(JSON.stringify({
    success: true,
    results: results
  }))
    .setMimeType(ContentService.MimeType.JSON);
}
