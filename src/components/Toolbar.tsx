export function Toolbar(props: {
  query: string;
  favoritesOnly: boolean;
  taggedOnly: boolean;
  hideMissing: boolean;
  busy: boolean;
  onQuery: (query: string) => void;
  onSearch: () => void;
  onFavoritesOnly: (value: boolean) => void;
  onTaggedOnly: (value: boolean) => void;
  onHideMissing: (value: boolean) => void;
  onRefresh: () => void;
  onSettings: () => void;
  onHelp: () => void;
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
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            props.onSearch();
          }
        }}
      />
      <button type="button" disabled={props.busy} onClick={props.onSearch}>
        Search
      </button>
      <button
        type="button"
        aria-label="Search help"
        title="Search help"
        disabled={props.busy}
        onClick={props.onHelp}
      >
        ?
      </button>
      <input
        id="favorites-only"
        type="checkbox"
        checked={props.favoritesOnly}
        disabled={props.busy}
        onChange={(event) => props.onFavoritesOnly(event.currentTarget.checked)}
      />
      <label for="favorites-only">Favorites only</label>
      <input
        id="tagged-only"
        type="checkbox"
        checked={props.taggedOnly}
        disabled={props.busy}
        onChange={(event) => props.onTaggedOnly(event.currentTarget.checked)}
      />
      <label for="tagged-only">Tagged only</label>
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
