export function Toolbar(props: {
  query: string;
  favoritesOnly: boolean;
  hideMissing: boolean;
  busy: boolean;
  onQuery: (query: string) => void;
  onFavoritesOnly: (value: boolean) => void;
  onHideMissing: (value: boolean) => void;
  onRefresh: () => void;
  onSettings: () => void;
}) {
  return (
    <div class="field-row toolbar">
      <label for="search">Search</label>
      <input
        id="search"
        class="search-input"
        type="text"
        value={props.query}
        disabled={props.busy}
        onInput={(event) => props.onQuery(event.currentTarget.value)}
      />
      <input
        id="favorites-only"
        type="checkbox"
        checked={props.favoritesOnly}
        disabled={props.busy}
        onChange={(event) => props.onFavoritesOnly(event.currentTarget.checked)}
      />
      <label for="favorites-only">Favorites only</label>
      <input
        id="hide-missing"
        type="checkbox"
        checked={props.hideMissing}
        disabled={props.busy}
        onChange={(event) => props.onHideMissing(event.currentTarget.checked)}
      />
      <label for="hide-missing">Hide missing</label>
      <button type="button" disabled={props.busy} onClick={props.onRefresh}>
        Refresh
      </button>
      <button type="button" disabled={props.busy} onClick={props.onSettings}>
        Settings
      </button>
    </div>
  );
}
