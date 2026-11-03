import { useEffect, useRef, useState } from "preact/hooks";
import { displayLabel, thumbUrl } from "../media";
import type { ImageSummary } from "../types";

const CELL_W = 112;
const CELL_H = 140;
const GAP = 8;
const PAD = 6;

export function Grid(props: {
  items: ImageSummary[];
  loading: boolean;
  selectedPath: string | null;
  onSelect: (path: string) => void;
}) {
  const scroller = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState({ width: 0, height: 0, scrollTop: 0 });

  useEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const update = () => {
      setBox({
        width: el.clientWidth,
        height: el.clientHeight,
        scrollTop: el.scrollTop,
      });
    };
    update();
    const observer = new ResizeObserver(update);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const cols = Math.max(1, Math.floor((box.width - PAD * 2 + GAP) / (CELL_W + GAP)));
  const rowH = CELL_H + GAP;
  const rows = Math.ceil(props.items.length / cols);
  const totalH = rows === 0 ? 0 : PAD * 2 + rows * rowH - GAP;
  const firstRow = Math.max(0, Math.floor(box.scrollTop / rowH) - 2);
  const lastRow = Math.min(rows, firstRow + Math.ceil(box.height / rowH) + 5);
  const start = firstRow * cols;
  const end = Math.min(props.items.length, lastRow * cols);

  return (
    <div
      class="grid-scroller"
      ref={scroller}
      onScroll={(event) => {
        const scrollTop = event.currentTarget.scrollTop;
        setBox((prev) => ({ ...prev, scrollTop }));
      }}
    >
      {props.loading ? (
        <p class="empty">Loading…</p>
      ) : props.items.length === 0 ? (
        <p class="empty">No images.</p>
      ) : (
        <div class="grid-canvas" style={{ height: `${totalH}px` }}>
          {props.items.slice(start, end).map((item, offset) => {
            const index = start + offset;
            const col = index % cols;
            const row = Math.floor(index / cols);
            const selected = item.path === props.selectedPath;
            return (
              <div
                key={item.path}
                role="button"
                tabIndex={0}
                aria-pressed={selected}
                class={selected ? "cell selected" : "cell"}
                style={{
                  left: `${PAD + col * (CELL_W + GAP)}px`,
                  top: `${PAD + row * rowH}px`,
                  width: `${CELL_W}px`,
                  height: `${CELL_H}px`,
                }}
                onClick={() => props.onSelect(item.path)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    props.onSelect(item.path);
                  }
                }}
              >
                <Thumb item={item} />
                <span class="cell-label">{displayLabel(item)}</span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function Thumb(props: { item: ImageSummary }) {
  const [failed, setFailed] = useState(false);
  if (props.item.missing || failed) {
    return (
      <div class="thumb missing-thumb">{props.item.missing ? "Missing" : "No preview"}</div>
    );
  }
  return (
    <img
      class="thumb"
      alt=""
      src={thumbUrl(props.item.path, props.item.mtime)}
      onError={() => setFailed(true)}
    />
  );
}
