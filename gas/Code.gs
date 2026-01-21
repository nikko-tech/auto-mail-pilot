function doGet(e) {
  const action = e.parameter.action || 'getTemplates';

  if (action === 'getTemplates') {
    return getTemplates();
  }

  return ContentService.createTextOutput(JSON.stringify({ error: 'Unknown action' }))
    .setMimeType(ContentService.MimeType.JSON);
}

function getTemplates() {
  const ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName('テンプレート');
  
  if (!sheet) {
    // Create sheet if not exists (for first run ease)
    sheet = ss.insertSheet('テンプレート');
    sheet.appendRow(['Name', 'Subject', 'Body']);
    sheet.appendRow(['Greeting', 'Hello', 'Hi {{name}},\n\nHow are you?']);
  }

  const data = sheet.getDataRange().getValues();
  const templates = [];
  
  // Skip header
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
