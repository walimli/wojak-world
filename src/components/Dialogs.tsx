import { useState } from "preact/hooks";

export function MessageDialog(props: { message: string; onClose: () => void }) {
  return (
    <div class="modal-backdrop">
      <div class="window dialog" role="dialog" aria-label="Wojak World">
        <div class="title-bar">
          <div class="title-bar-text">Wojak World</div>
        </div>
        <div class="window-body">
          <p class="dialog-message">{props.message}</p>
          <section class="field-row dialog-actions">
            <button type="button" onClick={props.onClose}>
              OK
            </button>
          </section>
        </div>
      </div>
    </div>
  );
}

export function SettingsDialog(props: {
  initial: string;
  defaults: string;
  onCancel: () => void;
  onSave: (rootsText: string) => void;
}) {
  const [text, setText] = useState(props.initial);
  return (
    <div class="modal-backdrop">
      <div class="window dialog" role="dialog" aria-label="Folders">
        <div class="title-bar">
          <div class="title-bar-text">Folders</div>
        </div>
        <div class="window-body">
          <p>
            Saving replaces the indexed folders. It does not scan. Press Refresh to index them.
          </p>
          <div class="stack">
            <label for="settings-roots">Folders, one path per line</label>
            <textarea
              id="settings-roots"
              class="roots-input"
              value={text}
              onInput={(event) => setText(event.currentTarget.value)}
            />
          </div>
          <section class="field-row dialog-actions">
            <button type="button" onClick={() => setText(props.defaults)}>
              Use defaults
            </button>
            <button type="button" onClick={props.onCancel}>
              Cancel
            </button>
            <button type="button" onClick={() => props.onSave(text)}>
              Save
            </button>
          </section>
        </div>
      </div>
    </div>
  );
}
