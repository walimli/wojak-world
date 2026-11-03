export function StatusBar(props: {
  visible: number;
  total: number;
  favorites: number;
  selected: string;
  busy: boolean;
}) {
  return (
    <div class="status-bar">
      <p class="status-bar-field">Visible: {props.visible}</p>
      <p class="status-bar-field">Indexed: {props.total}</p>
      <p class="status-bar-field">Favorites: {props.favorites}</p>
      <p class="status-bar-field status-path">
        {props.busy ? "Indexing…" : props.selected || "Ready"}
      </p>
    </div>
  );
}
