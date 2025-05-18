# Testing the Notification System

This document explains how to test the notification system implementation.

## Overview

The notification system allows an external MCP server to send keywords to the MCP Link application, which will display a notification. When the user clicks on the notification, the app will be brought to the foreground and search for the keyword in the MCP-list page.

## Prerequisites

- Node.js installed on your system
- MCP Link application built and running

## Test Script Usage

1. Start the MCP Link application first
2. Open a terminal and navigate to the app directory
3. Run the test script with one or more keywords:

```bash
node test-notification.js "Docker" "Kubernetes"
```

4. This will send a POST request to the MCP Link app's HTTP server (port 8082) with the specified keywords
5. You should see a notification appear on your system with the keywords
6. Clicking the notification should bring the MCP Link app to the foreground and initiate a search for the first keyword in the MCP-list page

## Understanding The Flow

1. External service sends keywords to the HTTP server (port 8082)
2. The app receives the keywords and displays a platform-specific notification
3. When clicked, the notification activates the app via URI scheme
4. The URI scheme handler extracts the keyword and navigates to the MCP-list page
5. The MCP-list page performs a search using the keyword

## Troubleshooting

- If notifications don't appear:
  - Check the console for error messages
  - Verify that the HTTP server is running (port 8082)
  - Make sure your system allows notifications from the app

- If clicking notifications doesn't work:
  - Verify that the URI scheme is registered correctly
  - Check for errors in the console related to URI scheme handling

## Implementation Details

- Windows notifications use `winrt-notification`
- macOS notifications use `notify-rust`
- Linux notifications use `notify-rust`
- URI scheme: `mcplink://notification?keyword=value`
- HTTP server endpoint: `POST /recommendations` with payload `{ "keywords": ["keyword1", "keyword2", ...] }`