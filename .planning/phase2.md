# Phase 2: Organization & Workflow (The Manager's Dream)

## Objective
Once Phase 1 establishes rock-solid reliability, Phase 2 focuses on minimizing the manual labor required to organize the uploaded photos into specific "Obras" (Construction Sites).

---

## Feature 1: Bulk Assignment UI

### Description
Allow users to select multiple documents from the unassigned "Inbox" at once and assign them to an Obra in a single action.

### Requirements for Completion (Definition of Done)
- [ ] The Inbox UI includes checkboxes (or distinct selection states) for each document.
- [ ] A "Select All" button is present.
- [ ] Once items are selected, a batch action toolbar appears with an "Assign to Obra" dropdown.
- [ ] The backend endpoint handles batch assignment efficiently.
- [ ] After successful assignment, selected items disappear from the Inbox view.

### Testing Plan
1. **Selection Limits:** Upload 10 items. Select 5. Assign them to Project A. Verify exactly those 5 moved.
2. **Select All:** Use "Select All" on a page of 20 items. Assign them to Project B. Verify Inbox is now empty.

---

## Feature 2: Contextual Auto-Tagging & Grouping

### Description
Leverage EXIF data to suggest grouping. If a worker uploads several photos taken within a narrow time window, the system suggests they belong together.

### Requirements for Completion (Definition of Done)
- [ ] The client extracts timestamp metadata from newly selected photos.
- [ ] Photos taken within a configurable timeframe (e.g., 15 minutes) are visually grouped in the Inbox as a "Suggested Batch".
- [ ] A single click can assign the entire "Suggested Batch" to an Obra.

### Testing Plan
1. **Metadata Reading:** Take 3 photos in rapid succession, wait 30 minutes, and take 1 more. Upload all 4. 
2. **Grouping Logic:** Verify the first 3 are visually grouped "Taken recently together" while the 4th is separate.

---

## Feature 3: Status Flags

### Description
Add simple color-coded flags to documents to quickly signal their state to other team members.

### Requirements for Completion (Definition of Done)
- [ ] Each document has a `status` field in the database (e.g., Default, Doubt, In Progress, Completed).
- [ ] The UI displays a clear visual indicator (color dot or icon) for the status.
- [ ] Users can easily toggle the status from the document viewing screen.

### Testing Plan
1. **State Persistence:** Change a document state to "Doubt". Refresh the page. Verify it remains "Doubt".
2. **Visual Filtering:** (Optional/If implemented) Ensure documents marked "Doubt" stand out visually in lists.

---

## Feature 4: Obra Sub-folders (Rooms)

### Description
Allow logical sub-categorization within a single project (e.g., "Kitchen", "Main Bathroom") to prevent larger projects from becoming cluttered.

### Requirements for Completion (Definition of Done)
- [ ] Database schema supports optional `category` or `room` fields for documents within a project.
- [ ] The UI allows creating predefined or custom categories when viewing a project.
- [ ] Users can map documents to these categories upon assignment or later.
- [ ] Project view supports filtering or grouping by these categories.

### Testing Plan
1. **Hierarchy Check:** Assign a photo to "Obra A" -> "Kitchen". Assign another to "Obra A" -> "Living Room". Verify they appear separate within the specific Obra view.
