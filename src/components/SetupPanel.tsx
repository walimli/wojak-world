import { useState } from "preact/hooks";

export function SetupPanel(props: {
  rootsText: string;
  busy: boolean;
  onIndex: (rootsText: string) => void;
}) {
  const [text, setText] = useState(props.rootsText);
  return (
    <fieldset class="setup">
      <legend>Index library</legend>
      <p>
        These folders are indexed in place. The app does not copy or move the images, and it
        does not scan again until you press Refresh.
      </p>
      <div class="stack">
        <label for="setup-roots">Folders, one path per line</label>
        <textarea
          id="setup-roots"
          class="roots-input"
          value={text}
          disabled={props.busy}
          onInput={(event) => setText(event.currentTarget.value)}
        />
      </div>
      <section class="field-row dialog-actions">
        <button type="button" disabled={props.busy} onClick={() => props.onIndex(text)}>
          Index library
        </button>
      </section>
    </fieldset>
  );
}
