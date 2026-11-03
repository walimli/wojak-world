import { useEffect, useState } from "preact/hooks";
import { fileUrl } from "../media";
import type { ImageDetail, ImageSummary } from "../types";

export function Detail(props: {
  image: ImageSummary | null;
  detail: ImageDetail | null;
  onFavorite: (path: string, favorite: boolean) => void;
  onDisplayName: (path: string, displayName: string) => void;
  onTags: (path: string, tags: string[]) => void;
}) {
  const image = props.image;
  const [name, setName] = useState("");
  const [tagDraft, setTagDraft] = useState("");

  useEffect(() => {
    setName(image?.displayName ?? "");
    setTagDraft("");
  }, [image?.path, image?.displayName]);

  if (!image) {
    return (
      <aside class="detail">
        <p class="hint">Select an image.</p>
      </aside>
    );
  }

  const tags = props.detail?.path === image.path ? props.detail.tags : [];
  const categories = props.detail?.path === image.path ? props.detail.categories : [];

  function commitName() {
    const current = image?.displayName ?? "";
    if (name.trim() !== current.trim()) {
      props.onDisplayName(image!.path, name);
    }
  }

  function addTag() {
    const next = tagDraft.trim();
    if (!next) return;
    props.onTags(image!.path, [...tags, next]);
    setTagDraft("");
  }

  return (
    <aside class="detail">
      {image.missing ? (
        <p class="missing-copy">
          This file is missing. Favorites, tags, and the display name stay on this path.
        </p>
      ) : (
        <img class="preview" alt="" src={fileUrl(image.path, image.mtime)} />
      )}
      <div class="stack">
        <label for="display-name">Name</label>
        <input
          id="display-name"
          type="text"
          value={name}
          placeholder={image.basename}
          onInput={(event) => setName(event.currentTarget.value)}
          onBlur={commitName}
          onKeyDown={(event) => {
            if (event.key === "Enter") event.currentTarget.blur();
          }}
        />
      </div>
      <div class="field-row">
        <input
          id="favorite"
          type="checkbox"
          checked={image.favorite}
          onChange={(event) => props.onFavorite(image.path, event.currentTarget.checked)}
        />
        <label for="favorite">Favorite</label>
      </div>
      <fieldset>
        <legend>Tags</legend>
        {tags.map((tag) => (
          <div class="tag-row" key={tag}>
            <span>{tag}</span>
            <button
              type="button"
              onClick={() => props.onTags(image.path, tags.filter((item) => item !== tag))}
            >
              Remove
            </button>
          </div>
        ))}
        <div class="tag-add">
          <input
            type="text"
            aria-label="Add tag"
            value={tagDraft}
            onInput={(event) => setTagDraft(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                addTag();
              }
            }}
          />
          <button type="button" onClick={addTag}>
            Add
          </button>
        </div>
      </fieldset>
      {categories.length > 0 && (
        <p class="suggested">Suggested: {categories.join(", ")}</p>
      )}
      <div class="stack">
        <label for="image-path">Path</label>
        <input id="image-path" type="text" readonly value={image.path} />
      </div>
    </aside>
  );
}
