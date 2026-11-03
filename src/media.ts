/** Windows WebView2 origin for the registered `wojak` scheme. */
export function thumbUrl(path: string, mtime: number): string {
  return `http://wojak.localhost/thumb?p=${encodeURIComponent(path)}&v=${mtime}`;
}

export function fileUrl(path: string, mtime: number): string {
  return `http://wojak.localhost/file?p=${encodeURIComponent(path)}&v=${mtime}`;
}

export function displayLabel(item: {
  displayName: string | null;
  basename: string;
}): string {
  const name = item.displayName?.trim();
  return name ? name : item.basename;
}
