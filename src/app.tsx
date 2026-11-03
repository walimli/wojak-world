import { useEffect, useRef, useState } from "preact/hooks";
import {
  errorText,
  imageDetail,
  libraryState,
  refresh,
  runSetup,
  search,
  setDisplayName,
  setFavorite,
  setRoots,
  setTags,
} from "./api";
import { Detail } from "./components/Detail";
import { MessageDialog, SettingsDialog } from "./components/Dialogs";
import { Grid } from "./components/Grid";
import { SetupPanel } from "./components/SetupPanel";
import { StatusBar } from "./components/StatusBar";
import { TitleBar } from "./components/TitleBar";
import { Toolbar } from "./components/Toolbar";
import { displayLabel } from "./media";
import type { ImageDetail, SearchResponse } from "./types";

type Phase = { name: "loading" } | { name: "setup"; rootsText: string } | { name: "ready" };

function folderLines(text: string): string[] {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

export function App() {
  const [phase, setPhase] = useState<Phase>({ name: "loading" });
  const [roots, setRootList] = useState<string[]>([]);
  const [defaultRoots, setDefaultRoots] = useState<string[]>([]);
  const [query, setQuery] = useState("");
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [hideMissing, setHideMissing] = useState(true);
  const [result, setResult] = useState<SearchResponse | null>(null);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [detail, setDetail] = useState<ImageDetail | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const request = useRef(0);

  useEffect(() => {
    let alive = true;
    libraryState()
      .then((state) => {
        if (!alive) return;
        setRootList(state.roots);
        setDefaultRoots(state.defaultRoots);
        if (state.setupComplete) setPhase({ name: "ready" });
        else setPhase({ name: "setup", rootsText: state.roots.join("\n") });
      })
      .catch((error) => {
        if (alive) setMessage(errorText(error));
      });
    return () => {
      alive = false;
    };
  }, []);

  useEffect(() => {
    if (phase.name !== "ready") return;
    const id = ++request.current;
    const handle = window.setTimeout(() => {
      search(query, favoritesOnly, hideMissing)
        .then((response) => {
          if (request.current !== id) return;
          setResult(response);
          setSelectedPath((current) =>
            current && response.items.some((item) => item.path === current) ? current : null,
          );
        })
        .catch((error) => {
          if (request.current === id) setMessage(errorText(error));
        });
    }, query.trim() ? 150 : 0);
    return () => window.clearTimeout(handle);
  }, [phase, query, favoritesOnly, hideMissing]);

  useEffect(() => {
    if (!selectedPath) {
      setDetail(null);
      return;
    }
    let alive = true;
    imageDetail(selectedPath)
      .then((next) => {
        if (alive) setDetail(next);
      })
      .catch((error) => {
        if (alive) setMessage(errorText(error));
      });
    return () => {
      alive = false;
    };
  }, [selectedPath]);

  async function onIndex(rootsText: string) {
    setBusy(true);
    try {
      const report = await runSetup(folderLines(rootsText));
      const state = await libraryState();
      setRootList(state.roots);
      setDefaultRoots(state.defaultRoots);
      setPhase({ name: "ready" });
      if (report.errors.length > 0) setMessage(report.errors.join("\n"));
    } catch (error) {
      setMessage(errorText(error));
    } finally {
      setBusy(false);
    }
  }

  async function onRefresh() {
    setBusy(true);
    try {
      const report = await refresh();
      const response = await search(query, favoritesOnly, hideMissing);
      request.current += 1;
      setResult(response);
      setSelectedPath((current) =>
        current && response.items.some((item) => item.path === current) ? current : null,
      );
      if (report.errors.length > 0) setMessage(report.errors.join("\n"));
    } catch (error) {
      setMessage(errorText(error));
    } finally {
      setBusy(false);
    }
  }

  async function onSaveRoots(rootsText: string) {
    try {
      const saved = await setRoots(folderLines(rootsText));
      setRootList(saved);
      setSettingsOpen(false);
      setMessage("Folders saved. Press Refresh to index them.");
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function onFavorite(path: string, favorite: boolean) {
    try {
      await setFavorite(path, favorite);
      setResult((current) => {
        if (!current) return current;
        const items = current.items
          .map((item) => (item.path === path ? { ...item, favorite } : item))
          .filter((item) => !(favoritesOnly && item.path === path && !favorite));
        const removed = favoritesOnly && !favorite;
        return {
          ...current,
          items,
          visible: removed ? Math.max(0, current.visible - 1) : current.visible,
          favorites: Math.max(0, current.favorites + (favorite ? 1 : -1)),
        };
      });
      if (favoritesOnly && !favorite) setSelectedPath((current) => (current === path ? null : current));
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function onDisplayName(path: string, displayName: string) {
    try {
      const updated = await setDisplayName(path, displayName);
      setResult((current) => {
        if (!current) return current;
        return {
          ...current,
          items: current.items.map((item) =>
            item.path === path ? { ...item, displayName: updated } : item,
          ),
        };
      });
      setDetail((current) =>
        current && current.path === path ? { ...current, displayName: updated } : current,
      );
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function onTags(path: string, tags: string[]) {
    try {
      const stored = await setTags(path, tags);
      setDetail((current) => (current && current.path === path ? { ...current, tags: stored } : current));
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  const selected = result?.items.find((item) => item.path === selectedPath) ?? null;
  const selectedText = selected ? (selected.displayName?.trim() ? displayLabel(selected) : selected.path) : "";

  return (
    <div class="window app-frame">
      <TitleBar />
      <div class="window-body">
        {phase.name === "loading" && <p class="hint">Starting…</p>}
        {phase.name === "setup" && (
          <SetupPanel rootsText={phase.rootsText} busy={busy} onIndex={onIndex} />
        )}
        {phase.name === "ready" && (
          <>
            <Toolbar
              query={query}
              favoritesOnly={favoritesOnly}
              hideMissing={hideMissing}
              busy={busy}
              onQuery={setQuery}
              onFavoritesOnly={setFavoritesOnly}
              onHideMissing={setHideMissing}
              onRefresh={() => {
                void onRefresh();
              }}
              onSettings={() => setSettingsOpen(true)}
            />
            <div class="browser">
              <Grid
                items={result?.items ?? []}
                loading={result === null}
                selectedPath={selectedPath}
                onSelect={setSelectedPath}
              />
              <Detail
                image={selected}
                detail={detail}
                onFavorite={(path, favorite) => {
                  void onFavorite(path, favorite);
                }}
                onDisplayName={(path, displayName) => {
                  void onDisplayName(path, displayName);
                }}
                onTags={(path, tags) => {
                  void onTags(path, tags);
                }}
              />
            </div>
          </>
        )}
      </div>
      <StatusBar
        visible={result?.visible ?? 0}
        total={result?.total ?? 0}
        favorites={result?.favorites ?? 0}
        selected={selectedText}
        busy={busy}
      />
      {settingsOpen && (
        <SettingsDialog
          initial={roots.join("\n")}
          defaults={defaultRoots.join("\n")}
          onCancel={() => setSettingsOpen(false)}
          onSave={(text) => {
            void onSaveRoots(text);
          }}
        />
      )}
      {message && <MessageDialog message={message} onClose={() => setMessage(null)} />}
    </div>
  );
}
