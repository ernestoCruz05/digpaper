# Phase 3: Communication Enhancements (The Collaborative Edge)

## Objective
Focus on features that actively replace standard messaging apps (like WhatsApp) by enabling rich, context-aware communication directly on top of the documents stored in the system.

---

## Feature 1: Deep Linking / Direct Sharing

### Description
Every uploaded document receives a unique, stable URL that can be directly shared via a messaging app.

### Requirements for Completion (Definition of Done)
- [ ] The PWA router supports dynamic routes for specific documents (e.g., `/projects/123/documents/456`).
- [ ] A "Share" button exists on the document viewing interface.
- [ ] Clicking "Share" utilizes the native Web Share API (if available on mobile) to pop open the system share sheet, or copies the URL to the clipboard on desktop.
- [ ] Following a shared link opens the exact document directly (assuming the user is logged in/authorized).

### Testing Plan
1. **Link Generation:** Open a document, click Share, and copy the link.
2. **Direct Access:** Open an Incognito/Private window (or a different browser), paste the link, log in if prompted, and verify it routes directly to the correct image, not the homepage.

---

## Feature 2: Image Annotations (Canvas Drawing)

### Description
A simple drawing layer over the photo before uploading to highlight specific areas or write quick corrections.

### Requirements for Completion (Definition of Done)
- [ ] UI provides a basic drawing interface (e.g., an HTML5 Canvas over the image).
- [ ] Essential tools are available: Pen/Highlighter, color choice (e.g., Red/Yellow), and Undo.
- [ ] The resulting image sent to the backend is a flattened composition of the original photo and the annotation layer.

### Testing Plan
1. **Drawing Sandbox:** Select an image. Draw a red circle and a green line.
2. **Flatten & Upload:** Submit the image. View it on the backend or on another device and confirm the red circle and green line are permanently burnt into the saved image.

---

## Feature 3: Voice Memos tied to Images

### Description
Allow workers to attach short audio recordings to images to provide context quickly without typing.

### Requirements for Completion (Definition of Done)
- [ ] The upload interface includes a microphone button ("Hold to Record").
- [ ] Utilizing the Web Audio API / MediaRecorder API to capture audio.
- [ ] Audio files are compressed (e.g., standard `.ogg` or `.webm` format) and sent alongside the main photo payload.
- [ ] The document viewing screen includes an inline audio player for playback.

### Testing Plan
1. **Hardware Access:** Click the microphone button; verify the browser prompts for microphone permissions.
2. **Record & Review:** Record a 5-second memo clearly speaking. Upload the document.
3. **Cross-Device Playback:** Open the document on a separate device and press play. Ensure the audio is clear and audible.
