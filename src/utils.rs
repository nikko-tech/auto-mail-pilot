use crate::models::{RecipientData, RecipientInfo, Attachment};

pub fn apply_variables(mut text: String, recipient: &RecipientData) -> String {
    text = text.replace("{{name}}", &recipient.name);
    text = text.replace("{{company}}", &recipient.company);
    text = text.replace("{{email}}", &recipient.email);
    text = text.replace("{{id}}", &recipient.id);
    text
}

/// 文字列を正規化（スペース除去、小文字化）
fn normalize_string(s: &str) -> String {
    s.replace(" ", "")
        .replace("　", "")
        .to_lowercase()
}

/// 添付ファイル名と宛先の整合性をチェック
/// 添付ファイルに紐付けられた会社名と、宛先の会社名が一致するか検証
pub fn validate_attachment_recipient_match(
    attachments: &[Attachment],
    _recipient: &RecipientInfo,
    recipient_data: Option<&RecipientData>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    let recipient_company = recipient_data
        .map(|r| normalize_string(&r.company))
        .unwrap_or_default();
    let recipient_name = recipient_data
        .map(|r| normalize_string(&r.name))
        .unwrap_or_default();

    for att in attachments.iter().filter(|a| a.enabled) {
        if let Some(ref linked_company) = att.linked_company {
            let linked_normalized = normalize_string(linked_company);

            // 会社名または氏名が含まれているかチェック
            let company_match = !recipient_company.is_empty()
                && (linked_normalized.contains(&recipient_company)
                    || recipient_company.contains(&linked_normalized));
            let name_match = !recipient_name.is_empty()
                && (linked_normalized.contains(&recipient_name)
                    || recipient_name.contains(&linked_normalized));

            if !company_match && !name_match {
                errors.push(format!(
                    "⚠️ 添付ファイル「{}」は「{}」宛ですが、選択された宛先と一致しません",
                    att.file_name,
                    linked_company
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// 本文中の会社名・氏名と宛先の照合
pub fn validate_body_recipient_match(
    body: &str,
    recipient_data: Option<&RecipientData>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Some(rec) = recipient_data {
        let body_normalized = normalize_string(body);
        let company_normalized = normalize_string(&rec.company);
        let name_normalized = normalize_string(&rec.name);

        // 本文に「御中」や「様」の前に別の会社名がないかチェック
        // 本文に選択された宛先の会社名または氏名が含まれているか確認
        let has_company = company_normalized.is_empty()
            || body_normalized.contains(&company_normalized);
        let has_name = name_normalized.is_empty()
            || body_normalized.contains(&name_normalized);

        if !has_company && !has_name {
            errors.push(format!(
                "⚠️ 本文に「{}」または「{}」が見つかりません。宛先が正しいか確認してください",
                rec.company,
                rec.name
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// 重複チェック：同じ添付ファイルが複数の宛先に有効になっていないか
pub fn validate_no_cross_recipient_attachments(
    _recipients: &[RecipientInfo],
    attachments: &[Attachment],
    active_recipient_index: usize,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for att in attachments.iter().filter(|a| a.enabled) {
        if let Some(linked_idx) = att.linked_recipient_index {
            if linked_idx != active_recipient_index {
                errors.push(format!(
                    "⚠️ 添付ファイル「{}」は宛先{}に紐付けられています",
                    att.file_name,
                    linked_idx + 1
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// 全ての検証を実行
pub fn validate_send_safety(
    recipients: &[RecipientInfo],
    recipients_master: &[RecipientData],
    attachments: &[Attachment],
) -> Vec<String> {
    let mut all_errors = Vec::new();

    for (idx, recipient) in recipients.iter().enumerate() {
        if recipient.email.is_empty() {
            continue;
        }

        // 宛先マスターから対応するデータを取得
        let recipient_data = recipient.locked_recipient_id.as_ref()
            .and_then(|id| recipients_master.iter().find(|r| &r.id == id));

        // 1. 添付ファイルと宛先の整合性チェック
        // この宛先に対して有効な添付ファイルをフィルタ
        let recipient_attachments: Vec<_> = attachments.iter()
            .filter(|a| a.enabled && a.linked_recipient_index == Some(idx))
            .cloned()
            .collect();

        if let Err(errs) = validate_attachment_recipient_match(&recipient_attachments, recipient, recipient_data) {
            all_errors.extend(errs.into_iter().map(|e| format!("[宛先{}] {}", idx + 1, e)));
        }

        // 2. 本文と宛先の照合
        if let Err(errs) = validate_body_recipient_match(&recipient.body, recipient_data) {
            all_errors.extend(errs.into_iter().map(|e| format!("[宛先{}] {}", idx + 1, e)));
        }

        // 3. ロックされた宛先IDと現在の宛先が一致しているか
        if let Some(ref locked_id) = recipient.locked_recipient_id {
            let current_matches = recipient_data.map(|r| &r.id == locked_id).unwrap_or(false);
            if !current_matches && recipient_data.is_some() {
                all_errors.push(format!(
                    "[宛先{}] ⚠️ ロックされた宛先と選択された宛先が一致しません",
                    idx + 1
                ));
            }
        }
    }

    // 4. 同じメールアドレスが複数の宛先に設定されていないか
    let valid_emails: Vec<_> = recipients.iter()
        .filter(|r| !r.email.is_empty())
        .map(|r| &r.email)
        .collect();

    for (i, email) in valid_emails.iter().enumerate() {
        for (j, other) in valid_emails.iter().enumerate() {
            if i < j && email == other {
                all_errors.push(format!(
                    "⚠️ メールアドレス「{}」が複数の宛先に設定されています",
                    email
                ));
                break;
            }
        }
    }

    all_errors
}
