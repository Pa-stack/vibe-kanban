export type UploadMeta = {
  filename: string;
  mime: string;
  size: number;
  content_hash: string;
  stored_at: string;
  extracted_text: boolean;
  dedup?: boolean;
};

async function api<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, init);
  const json = await res.json();
  return json.data as T;
}

export const contextApi = {
  listProject: (projectId: string) =>
    api<UploadMeta[]>(`/api/projects/${projectId}/uploads`),
  uploadProject: async (projectId: string, file: File) => {
    const fd = new FormData();
    fd.append('file', file);
    const res = await fetch(`/api/projects/${projectId}/uploads`, { method: 'POST', body: fd });
    const json = await res.json();
    return json.data as UploadMeta;
  },
  listTask: (taskId: string) => api<UploadMeta[]>(`/api/tasks/${taskId}/uploads`),
  uploadTask: async (taskId: string, file: File) => {
    const fd = new FormData();
    fd.append('file', file);
    const res = await fetch(`/api/tasks/${taskId}/uploads`, { method: 'POST', body: fd });
    const json = await res.json();
    return json.data as UploadMeta;
  },
};
