# macOS Code Signing & Notarization Setup Guide

To ensure that non-engineer users can download and run `Claude Desktop Switcher` without encountering the macOS Gatekeeper "App is damaged" error, the application must be properly signed and notarized by Apple.

This document outlines the steps the repository administrator must take to configure GitHub Actions for automated Code Signing and Notarization via Tauri.

## Prerequisites
1. An active [Apple Developer Program](https://developer.apple.com/programs/) membership ($99/year).
2. A macOS machine with Xcode installed to generate certificates.

## 1. Generate Certificates

You need an **Developer ID Application** certificate to distribute the app outside the Mac App Store.

1. Open **Xcode** > Settings > Accounts.
2. Select your Apple ID and click **Manage Certificates...**
3. Click the **+** button and select **Developer ID Application**.
4. Once created, right-click the certificate in the list and select **Export Certificate...**
5. Save it as `Certificates.p12` and set a strong password.

## 2. Generate Base64 String from the Certificate

GitHub Actions requires the `.p12` file to be encoded in Base64 format.
Run the following command in your terminal:

```bash
base64 -i Certificates.p12 -o certificate.b64
```
Open `certificate.b64` in a text editor. You will copy its contents to GitHub Secrets in Step 4.

## 3. Generate App Store Connect API Key (for Notarization)

Notarization requires an App Store Connect API Key.

1. Log in to [App Store Connect](https://appstoreconnect.apple.com/).
2. Go to **Users and Access** > **Keys** > **App Store Connect API**.
3. Click **+** to generate a new key.
   * **Name**: `GitHub Actions Notarization`
   * **Access**: `App Manager`
4. Download the `.p8` key file. (You can only download it once).
5. Note down your **Issuer ID** and the **Key ID**.

## 4. Add Secrets to GitHub

Go to your repository on GitHub > **Settings** > **Secrets and variables** > **Actions**.
Click **New repository secret** and add the following secrets exactly as named:

| Secret Name | Value |
|-------------|-------|
| `APPLE_CERTIFICATE` | The contents of `certificate.b64` (the Base64 string from Step 2). |
| `APPLE_CERTIFICATE_PASSWORD` | The password you set when exporting the `.p12` file in Step 1. |
| `APPLE_SIGNING_IDENTITY` | Your Developer ID Application name (e.g., `Developer ID Application: Your Name (TEAM_ID)`). You can find this in your Keychain Access. |
| `APPLE_ID` | Your App Store Connect API **Key ID** (from Step 3). |
| `APPLE_PASSWORD` | The contents of the `.p8` key file (from Step 3). |
| `APPLE_TEAM_ID` | Your App Store Connect API **Issuer ID** (from Step 3). |

## 5. Verify the Build

Once these secrets are configured, the next time a release is triggered (via `main` branch push and Release Please), the `build-mac-dmg` job in GitHub Actions will automatically:
1. Decode the certificate and import it to the runner's keychain.
2. Code Sign the `.app` bundle.
3. Submit the `.app` to Apple's notarization service (`notarytool`).
4. Staple the notarization ticket to the `.app`.
5. Package the `.app` into a `.dmg` and upload it to the GitHub Release.

End users downloading the DMG will be able to double-click and open the app natively, without using the terminal or encountering the "App is damaged" quarantine error.
