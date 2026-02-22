# Phase 1: Core Reliability (The Foundation)

## Objective
The primary goal of Phase 1 is to solve the most critical pain points for field workers: uploading large images over slow or non-existent internet connections (common inside concrete shells or basements). The app must be trusted to "never lose a photo."

---

## Feature 1: Client-Side Image Compression

### Description
Automatically compress high-resolution smartphone photos down to a manageable size (~200-500KB) before transmitting them to the backend server to ensure faster uploads and prevent timeouts.

### Requirements for Completion (Definition of Done)
- [ ] Integration of a client-side compression library (e.g., `browser-image-compression` or native Canvas API).
- [ ] Compression occurs automatically when the user selects a photo.
- [ ] Visual indicator (spinner or progress bar) is present while compressing and uploading.
- [ ] The final uploaded file clearly maintains readability for handwritten notes/sketches.
- [ ] The backend correctly receives and stores the smaller payload.

### Testing Plan
1. **Large File Test:** Take a raw 10MB+ photo with a phone camera and upload it. Verify the resulting file on the backend is < 500KB.
2. **Clarity Test:** Upload a photo of handwritten numbers (e.g., "55cm"). Verify the numbers are perfectly legible on the compressed version.
3. **Speed Test:** Throttle browser network to "Fast 3G" in DevTools and upload; ensure it succeeds without timing out.

---

## Feature 2: Offline-First Upload Queue

### Description
Allow workers to queue uploads when there is zero internet connection. The app will store the data locally and automatically sync it with the server when a stable connection is restored.

### Requirements for Completion (Definition of Done)
- [ ] The PWA is registered with a functional Service Worker.
- [ ] The app uses IndexedDB to store pending uploads (image blob + metadata).
- [ ] The UI allows the user to click "Upload" even while completely offline, showing a "Saved to queue" success message.
- [ ] A visual badge/indicator displays the number of currently pending uploads.
- [ ] Background Sync API (or a fallback polling mechanism when the app is reopened) automatically attempts to push the queued items when the network status changes to "online".
- [ ] Upon successful sync, items are removed from IndexedDB and the UI badge clears.
- [ ] Error handling for failed syncs (e.g., server down) to keep items in the queue.

### Testing Plan
1. **Offline Capture:** Turn off Wi-Fi and Mobile Data (airplane mode). Open the app, take a picture, and click upload. Verify it says "Queued".
2. **App Restart:** While still offline, close the app completely and reopen it. Verify the UI still shows "1 pending upload".
3. **Sync Recovery:** Turn Wi-Fi/Data back on. Verify the app detects the connection, uploads the file in the background, and clears the pending queue. Verify the image appears on the backend.
