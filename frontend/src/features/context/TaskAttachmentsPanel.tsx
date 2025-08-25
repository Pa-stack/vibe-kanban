import { useEffect, useRef, useState } from 'react';
import { contextApi, type UploadMeta } from './api';

export function TaskAttachmentsPanel({ taskId, projectDocs }: { taskId: string; projectDocs?: UploadMeta[] }) {
  const [items, setItems] = useState<UploadMeta[]>([]);
  const [busy, setBusy] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const refresh = async () => setItems(await contextApi.listTask(taskId));
  useEffect(() => { refresh(); }, [taskId]);

  const onFiles = async (files: FileList | null) => {
    if (!files?.length) return;
    setBusy(true);
    try {
      await contextApi.uploadTask(taskId, files[0]);
      await refresh();
    } finally { setBusy(false); }
  };

  return (
    <div className="space-y-4">
      <div className="border rounded p-3 flex items-center justify-between text-sm text-muted-foreground">
        <span>Attach a file to this task.</span>
        <input ref={inputRef} type="file" onChange={(e) => onFiles(e.target.files)} />
      </div>

      {!!projectDocs?.length && (
        <div>
          <div className="text-xs uppercase text-muted-foreground mb-1">Project Context (read-only)</div>
          <ul className="text-sm list-disc pl-4">
            {projectDocs.map((d) => (
              <li key={d.filename + d.content_hash}>{d.filename} — {d.mime}</li>
            ))}
          </ul>
        </div>
      )}

      <div>
        <div className="text-xs uppercase text-muted-foreground mb-1">Task Attachments</div>
        <ul className="text-sm list-disc pl-4">
          {items.map((d) => (
            <li key={d.filename + d.content_hash}>{d.filename} — {d.mime}</li>
          ))}
          {!items.length && <li className="text-muted-foreground">No attachments yet.</li>}
        </ul>
      </div>
      {busy && <div className="text-xs text-muted-foreground">Uploading…</div>}
    </div>
  );
}
