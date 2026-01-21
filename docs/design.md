# Auto Mail Pilot Design Document

## Architecture Overview
The system consists of two main components:
1.  **Desktop App (Rust + egui)**: The user interface for composing and managing emails.
2.  **Backend (Google Apps Script)**: A middleman that receives instructions from the desktop app and interacts with Gmail and Google Sheets.

## Data Flow
1.  User inputs recipient details and email bodies in the Desktop App.
2.  Desktop App sends a JSON payload to the GAS Web App URL.
3.  GAS Web App parses the payload and uses `GmailApp.sendEmail()` to send messages.
4.  (Future) GAS reads signature and recipient templates from Google Sheets.

## Key Features
- **Multi-recipient support**: Send to up to 3 people simultaneously.
- **Custom Bodies**: Individualized content for each recipient.
- **GAS Integration**: Leverages Gmail's API via a simple web app.
