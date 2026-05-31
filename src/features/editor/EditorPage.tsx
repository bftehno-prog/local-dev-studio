import { useCallback, useEffect, useMemo, useState } from 'react';
import { FileText, Folder, RefreshCcw, Save } from 'lucide-react';
import type { TFunction } from '../../app/types';
import type {
  Project,
  ProjectFileContent,
  ProjectFileEntry,
  RecentProjectFile,
} from '../../lib/types';
import { api } from '../../shared/lib/api';
import { Panel } from '../../components/ui/Panel';

type EditorPageProps = {
  projects: Project[];
  selectedProjectId?: string;
  onSelectProject: (id: string) => void;
  onRun: (action: () => Promise<unknown>, success: string) => Promise<void>;
  showError: (error: unknown) => void;
  t: TFunction;
};

export function EditorPage({
  projects,
  selectedProjectId,
  onSelectProject,
  onRun,
  showError,
  t,
}: EditorPageProps) {
  const [files, setFiles] = useState<ProjectFileEntry[]>([]);
  const [recentFiles, setRecentFiles] = useState<RecentProjectFile[]>([]);
  const [activeFile, setActiveFile] = useState<ProjectFileContent | null>(null);
  const [draft, setDraft] = useState('');
  const [loading, setLoading] = useState(false);
  const selectedProject = projects.find((project) => project.id === selectedProjectId);
  const dirty = activeFile ? draft !== activeFile.content : false;

  const visibleFiles = useMemo(
    () => files.filter((file) => file.is_dir || isEditableName(file.name)),
    [files],
  );

  const loadFiles = useCallback(
    async (projectId = selectedProjectId) => {
      if (!projectId) {
        setFiles([]);
        setRecentFiles([]);
        setActiveFile(null);
        setDraft('');
        return;
      }
      setLoading(true);
      try {
        const [nextFiles, nextRecentFiles] = await Promise.all([
          api.listProjectFiles(projectId),
          api.listRecentFiles(projectId),
        ]);
        setFiles(nextFiles);
        setRecentFiles(nextRecentFiles);
      } finally {
        setLoading(false);
      }
    },
    [selectedProjectId],
  );

  async function openFile(file: ProjectFileEntry) {
    if (!selectedProjectId || file.is_dir) return;
    await openFilePath(file.path);
  }

  async function openFilePath(path: string) {
    if (!selectedProjectId) return;
    const content = await api.readProjectFile(selectedProjectId, path);
    setActiveFile(content);
    setDraft(content.content);
    setRecentFiles(await api.listRecentFiles(selectedProjectId));
  }

  async function saveFile() {
    if (!selectedProjectId || !activeFile) return;
    await onRun(async () => {
      const saved = await api.writeProjectFile(selectedProjectId, activeFile.path, draft);
      setActiveFile(saved);
      setDraft(saved.content);
    }, t('message.fileSaved'));
  }

  useEffect(() => {
    void loadFiles().catch(showError);
  }, [loadFiles, showError]);

  return (
    <div className="content">
      <Panel title={t('editor.title')}>
        <div className="toolbar">
          <select
            value={selectedProjectId || ''}
            onChange={(event) => {
              onSelectProject(event.target.value);
              setActiveFile(null);
              setDraft('');
            }}
          >
            <option value="">{t('editor.selectProject')}</option>
            {projects.map((project) => (
              <option key={project.id} value={project.id}>
                {project.name}
              </option>
            ))}
          </select>
          <button disabled={!selectedProjectId || loading} onClick={() => void loadFiles()}>
            <RefreshCcw size={15} /> {t('action.refresh')}
          </button>
          <button disabled={!dirty} onClick={() => void saveFile()}>
            <Save size={15} /> {t('action.saveFile')}
          </button>
        </div>
      </Panel>
      <div className="editor-grid">
        <Panel title={selectedProject?.name || t('editor.files')}>
          {recentFiles.length > 0 && (
            <div className="recent-files">
              <span className="muted">{t('editor.recent')}</span>
              {recentFiles.map((file) => (
                <button
                  key={file.path}
                  onClick={() => void openFilePath(file.path).catch(showError)}
                >
                  <FileText size={14} />
                  <span>{file.path}</span>
                </button>
              ))}
            </div>
          )}
          <div className="file-tree">
            {visibleFiles.length === 0 && <p className="muted">{t('editor.noFiles')}</p>}
            {visibleFiles.map((file) => (
              <button
                className={activeFile?.path === file.path ? 'active' : ''}
                disabled={file.is_dir}
                key={file.path}
                onClick={() => void openFile(file).catch(showError)}
                style={{ paddingLeft: 8 + file.path.split('/').length * 10 }}
              >
                {file.is_dir ? <Folder size={14} /> : <FileText size={14} />}
                <span>{file.name}</span>
              </button>
            ))}
          </div>
        </Panel>
        <Panel title={activeFile?.path || t('editor.empty')}>
          <div className="editor-toolbar">
            <span className="muted">
              {activeFile
                ? `${activeFile.size} B${dirty ? ` · ${t('editor.unsaved')}` : ''}`
                : t('editor.chooseFile')}
            </span>
          </div>
          <textarea
            className="code-editor"
            disabled={!activeFile}
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            spellCheck={false}
          />
        </Panel>
      </div>
    </div>
  );
}

function isEditableName(name: string) {
  return /\.(css|html|js|jsx|json|md|mjs|php|ts|tsx|txt|yml|yaml)$/i.test(name);
}
