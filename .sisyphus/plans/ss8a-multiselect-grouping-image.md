# SS8a: Multi-Select + Grouping + Image Upload

**Sprint:** SS8a (before SS8b PPTX Full Roundtrip)
**Target:** Presentation Editor (`presentationeditor-react`)
**Goal:** Enable multi-shape selection, shape grouping, and image insertion on slides.

---

## Phase 1: Types + Store (Foundation)

### 1.1 Types (`presentation.ts`)

- Add `"image"` to `ShapeType` union
- Add `ImageData` interface: `{ src: string; alt?: string }`
- Add `imageData?: ImageData` to `ShapeData`
- Add `groupId?: string` to `ShapeData` (for grouping membership)

### 1.2 Store (`PresentationStore.ts`)

**Multi-Select:**
- Replace `selectedShapeId: string | null` with `selectedShapeIds: string[]`
- `selectShape(id)`: single-select (clears others)
- `toggleShapeSelection(id)`: Shift+click behavior (add/remove from selection)
- `deselectAllShapes()`: clear all
- `selectAllShapes()`: select all shapes on current slide
- `isSelected(id)`: helper to check if shape is in selection
- `selectedShapes`: computed getter for selected shape objects
- Update every method that used `selectedShapeId`:
  - `copyShape()` → copy all selected shapes
  - `cutShape()` → cut all selected shapes
  - `pasteShape()` → paste all clipboard shapes
  - `removeShape()` → also remove from selectedShapeIds
  - `alignShape()` → support multi-shape alignment
  - `bringForward/sendBackward/bringToFront/sendToBack` → apply to all selected

**Multi-Drag:**
- `moveShape()` signature stays but update multiple shapes when multiple are selected
- In `handleMouseMove` canvas handler, move all selected shapes

**Grouping:**
- Add `groups: Map<string, string[]>` to store (groupId → member shapeIds)
- `groupSelected()`: create group from current selection
- `ungroupSelected()`: dissolve group
- When a grouped shape is dragged/resized, all members move proportionally
- `getGroupBounds(groupId)`: computed bounding box

**Image Upload:**
- `addImageToSlide(slideIndex, file: File)`: read file as data URL via FileReader, create image shape
- Image shape defaults: ~200×200, centered-ish

## Phase 2: Canvas + UI (Rendering)

### 2.1 Canvas (`SlideCanvas.tsx`)

**Multi-Selection Visual:**
- Change `shape.id === selectedShapeId` check to `selectedShapeIds.includes(shape.id)`
- Each selected shape gets `outline: "2px solid var(--wo-prese-accent)"`
- Resize handles shown on ALL selected shapes

**Multi-Drag:**
- `onDragStart` must capture positions of ALL selected shapes
- `handleMouseMove` updates all captured shapes
- Store `dragState: { shapeIds: string[], startX, startY, origPositions: Map<id, {x,y}> }`

**Image Rendering:**
- Add `case "image":` to renderShape switch
- Render `<img>` tag inside a `<div>` with proper sizing
- Support resize/rotation handles same as other shapes

### 2.2 Keyboard (`useKeyboardShortcuts.ts`)

- `Ctrl+A` → select all shapes on current slide
- `Delete/Backspace` → remove all selected shapes
- Keep existing shortcuts

### 2.3 Toolbar (`InsertTab.tsx`)

- Wire "Pictures" button to hidden `<input type="file" accept="image/*">`
- On file select → `presentationStore.addImageToSlide()`
- Use a hidden ref + programmatic click

### 2.4 HomeTab Arrange Dropdown

- Add "Group" / "Ungroup" buttons (enabled when 2+ shapes selected for group, or group selected for ungroup)
- Add "Select All" button

## Phase 3: Verification

- `pnpm typecheck` passes (20/20)
- Manual test: Shift+click multiple shapes → all show selection
- Manual test: Drag multi-selection → all move together
- Manual test: Group shapes → they move as a unit
- Manual test: Insert image from file → image shape appears on canvas
- Manual test: Ctrl+A → all shapes selected
- Manual test: Delete with multi-selection → all removed
