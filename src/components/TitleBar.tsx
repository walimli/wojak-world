import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "preact/hooks";

export function TitleBar() {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let alive = true;
    let unlisten: (() => void) | undefined;
    const win = getCurrentWindow();
    win
      .isMaximized()
      .then((value) => {
        if (alive) setMaximized(value);
      })
      .catch(() => {});
    win
      .onResized(() => {
        win
          .isMaximized()
          .then((value) => {
            if (alive) setMaximized(value);
          })
          .catch(() => {});
      })
      .then((stop) => {
        unlisten = stop;
      })
      .catch(() => {});
    return () => {
      alive = false;
      unlisten?.();
    };
  }, []);

  function toggleMaximize() {
    void getCurrentWindow().toggleMaximize();
  }

  return (
    <div class="title-bar">
      <div class="title-bar-text" data-tauri-drag-region onDblClick={toggleMaximize}>
        Wojak World
      </div>
      <div class="title-drag" data-tauri-drag-region onDblClick={toggleMaximize} />
      <div class="title-bar-controls">
        <button
          type="button"
          aria-label="Minimize"
          onClick={() => {
            void getCurrentWindow().minimize();
          }}
        />
        <button
          type="button"
          aria-label={maximized ? "Restore" : "Maximize"}
          onClick={toggleMaximize}
        />
        <button
          type="button"
          aria-label="Close"
          onClick={() => {
            void getCurrentWindow().close();
          }}
        />
      </div>
    </div>
  );
}
