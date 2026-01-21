use crate::models::RecipientData;

pub fn apply_variables(mut text: String, recipient: &RecipientData) -> String {
    text = text.replace("{{name}}", &recipient.name);
    text = text.replace("{{company}}", &recipient.company);
    text = text.replace("{{email}}", &recipient.email);
    text = text.replace("{{id}}", &recipient.id);
    text
}
