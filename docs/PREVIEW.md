# Preview

## Local Preview

Local preview uses `http://localhost:<port>` and is rendered in an iframe.

## External Browser

Use the browser button to open the same URL in the default browser.

## Network Preview

When enabled, the app derives a LAN URL from Windows network configuration and shows a QR code. Fallback is `127.0.0.1`.

## Ports

If a configured port is occupied, Local Dev Studio logs a warning and chooses a free port from the configured range.

## CSP Notes

The CSP allows local iframe origins only. LAN iframe origins are intentionally not broadly enabled until a dedicated trusted-LAN policy is implemented.
