import { useEffect, useRef, useState } from 'react';
import { contextApi, type UploadMeta } from './api';

export function ProjectContextPanel({ projectId }: { projectId: string }) {
  const [items, setItems] = useState<UploadMeta[]>([]);
  const [busy, setBusy] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const refresh = async () => {
    const data = await contextApi.listProject(projectId);
    setItems(data);
  };
  useEffect(() => { refresh(); }, [projectId]);

  const onFiles = async (files: FileList | null) => {
    if (!files || !files.length) return;
    setBusy(true);
    try {
      await contextApi.uploadProject(projectId, files[0]);
      await refresh();
    } finally {
      setBusy(false);
    }
  };

  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    onFiles(e.dataTransfer.files);
  };

  return (
    <div className="space-y-3">
      <div
        onDragOver={(e) => e.preventDefault()}
        onDrop={onDrop}
        className="border rounded p-4 text-sm text-muted-foreground flex items-center gap-2 justify-between"
      >
        <span>Drag a file here or choose a file to add to Context Library.</span>
        <div className="flex items-center gap-2">
          <input ref={inputRef} type="file" onChange={(e) => onFiles(e.target.files)} />
        </div>
      </div>
      <table className="w-full text-sm">
        <thead>
          <tr className="text-left text-muted-foreground">
            <th className="py-1">Name</th>
            <th>Size</th>
            <th>Mime</th>
            <th>Checksum</th>
            <th>Download</th>
          </tr>
        </thead>
        <tbody>
          {items.map((it) => (
            <tr key={it.filename + it.content_hash} className="border-t">
              <td className="py-1">{it.filename}</td>
              <td>{(it.size / 1024).toFixed(1)} KB</td>
              <td>{it.mime}</td>
              <td className="font-mono text-xs">{it.content_hash.slice(0, 12)}…</td>
              <td>
                <a
                  className="text-blue-600 hover:underline"
                  href={`/api/uploads/${it.content_hash}/${encodeURIComponent(it.filename)}`}
                  download
                >
                  Download
                </a>
              </td>
            </tr>
          ))}
          {!items.length && (
            <tr>
              <td colSpan={5} className="text-center py-4 text-muted-foreground">
                No context files yet.
              </td>
            </tr>
          )}
        </tbody>
      </table>
      {busy && <div className="text-xs text-muted-foreground">Uploading…</div>}
    </div>
  );
}
