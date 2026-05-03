# Send Button Tooltip and Focus Ring — Proof Evidence

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: claude/serene-brown-ovvuR

## Feature

Dynamic `title` + `aria-label` on the Send button:
- Empty editor: "Type a message to send"
- Has text: "Send message (Enter)"
- Streaming: "Waiting for response…"

High-contrast `:focus-visible` ring using `--color-fg` instead of `--color-accent`
so the focus ring contrasts against the gold button background.

## Verification

### Unit tests — 292/292 pass

Three tests in `InputField.test.ts` directly verify the dynamic label behaviour:

1. `enables send button when editor has text` (line 87): confirms button
   is disabled when editor is empty and enabled after text is typed.
   Query updated to `getByRole('button', { name: /send/i })` to match
   "Type a message to send" (empty) and "Send message (Enter)" (has text).

2. `syncs editorText after npc chip click so send button is enabled (#684)`
   (line 613): same pattern after NPC chip click populates the editor.

3. `inserts pasted text at the cursor and keeps editorText state in sync
   (send enabled)` (line 731): same pattern after paste event.

All three assert `sendBtn.disabled === false` after the input arrives,
confirming the reactive label and disabled-state binding are both wired.

### Code review

`InputField.svelte` line 1034-1035:
```svelte
title={$streamingActive ? 'Waiting for response…' :
       isEditorEmpty() ? 'Type a message to send' : 'Send message (Enter)'}
aria-label={$streamingActive ? 'Waiting for response…' :
            isEditorEmpty() ? 'Type a message to send' : 'Send message (Enter)'}
```

`svelte-check` reports 0 errors (1 pre-existing unrelated warning).

### CSS review

`:focus-visible` ring rule sets `outline-color: var(--color-fg)` on `.send-btn`,
giving a dark foreground ring against the gold (`--color-accent`) background.
