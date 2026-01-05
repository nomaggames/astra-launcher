import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Check if running inside Tauri
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

interface UpdateInfo {
  latest_version: string;
  download_url: string;
  release_notes: string;
  is_update_available: boolean;
  installed_version: string | null;
}

interface DownloadProgress {
  downloaded_bytes: number;
  total_bytes: number;
  percentage: number;
  status: string;
}

interface LauncherConfig {
  fullscreen: boolean;
}

type AppState = 'checking' | 'ready' | 'update-available' | 'downloading' | 'error';
type View = 'main' | 'settings';

function App() {
  const [state, setState] = useState<AppState>('checking');
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<View>('main');
  const [config, setConfig] = useState<LauncherConfig>({ fullscreen: true });

  useEffect(() => {
    loadConfig();
    checkForUpdates();

    const unlisten = listen<DownloadProgress>('download-progress', (event) => {
      setProgress(event.payload);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  async function checkForUpdates() {
    setState('checking');
    setError(null);
    try {
      const info = await invoke<UpdateInfo>('check_updates');
      setUpdateInfo(info);

      if (info.is_update_available) {
        setState('update-available');
      } else {
        setState('ready');
      }
    } catch (e) {
      setError(String(e));
      setState('error');
    }
  }

  async function startDownload() {
    if (!updateInfo) return;

    setState('downloading');
    try {
      await invoke('download_game_update', {
        version: updateInfo.latest_version,
        downloadUrl: updateInfo.download_url,
      });
      setState('ready');
      setUpdateInfo({
        ...updateInfo,
        is_update_available: false,
        installed_version: updateInfo.latest_version,
      });
    } catch (e) {
      setError(String(e));
      setState('error');
    }
  }

  async function launchGame() {
    try {
      await invoke('launch_astra');
    } catch (e) {
      setError(String(e));
    }
  }

  async function loadConfig() {
    try {
      const cfg = await invoke<LauncherConfig>('get_config');
      setConfig(cfg);
    } catch (e) {
      console.error('Failed to load config:', e);
    }
  }

  async function saveConfig(newConfig: LauncherConfig) {
    try {
      await invoke('update_config', { config: newConfig });
      setConfig(newConfig);
    } catch (e) {
      setError(String(e));
    }
  }

  // Show message if not running in Tauri
  if (!isTauri) {
    return (
      <div className="launcher">
        <header className="launcher-header">
          <h1>ASTRA</h1>
          <span className="tagline">Mine. Fight. Trade. Explore.</span>
        </header>
        <main className="launcher-content">
          <section className="status-section" style={{ width: '100%', textAlign: 'center' }}>
            <div className="status error">
              <p>This launcher must be run as a desktop application.</p>
              <p style={{ fontSize: '0.9rem', opacity: 0.7, marginTop: '1rem' }}>
                Download the launcher from the releases page or run with:<br />
                <code style={{ background: '#333', padding: '0.5rem', borderRadius: '4px', display: 'inline-block', marginTop: '0.5rem' }}>
                  npm run dev:launcher
                </code>
              </p>
            </div>
          </section>
        </main>
        <footer className="launcher-footer">
          <span>2025 NoMag Games</span>
        </footer>
      </div>
    );
  }

  if (view === 'settings') {
    return (
      <div className="launcher">
        <header className="launcher-header">
          <h1>ASTRA</h1>
          <span className="tagline">Mine. Fight. Trade. Explore.</span>
        </header>

        <main className="launcher-content">
          <section className="settings-section">
            <div className="settings-header">
              <h2>Settings</h2>
              <button onClick={() => setView('main')} className="back-btn">
                ← Back
              </button>
            </div>

            <div className="settings-options">
              <div className="setting-item">
                <label htmlFor="fullscreen-toggle">
                  <span className="setting-label">Fullscreen Mode</span>
                  <span className="setting-description">
                    Launch the game in fullscreen by default
                  </span>
                </label>
                <input
                  id="fullscreen-toggle"
                  type="checkbox"
                  className="toggle"
                  checked={config.fullscreen}
                  onChange={(e) => saveConfig({ fullscreen: e.target.checked })}
                />
              </div>
            </div>
          </section>
        </main>

        <footer className="launcher-footer">
          <span>2025 NoMag Games</span>
          <div className="footer-links">
            <a href="https://wiki.astragame.online" target="_blank" rel="noopener noreferrer">Wiki</a>
            <a href="https://client.astragame.online" target="_blank" rel="noopener noreferrer">Play in Browser</a>
          </div>
        </footer>
      </div>
    );
  }

  return (
    <div className="launcher">
      <header className="launcher-header">
        <button onClick={() => setView('settings')} className="settings-btn" title="Settings">
          ⚙️
        </button>
        <h1>ASTRA</h1>
        <span className="tagline">Mine. Fight. Trade. Explore.</span>
      </header>

      <main className="launcher-content">
        <section className="status-section">
          {state === 'checking' && (
            <div className="status">
              <div className="spinner"></div>
              <p>Checking for updates...</p>
            </div>
          )}

          {state === 'error' && (
            <div className="status error">
              <p>Error: {error}</p>
              <button onClick={checkForUpdates}>Retry</button>
            </div>
          )}

          {state === 'update-available' && (
            <div className="status update">
              <p>Update available: v{updateInfo?.latest_version}</p>
              <p className="current">
                Current: {updateInfo?.installed_version || 'Not installed'}
              </p>
              <button onClick={startDownload} className="download-btn">
                {updateInfo?.installed_version ? 'Update' : 'Install'}
              </button>
            </div>
          )}

          {state === 'downloading' && progress && (
            <div className="status downloading">
              <p>{progress.status}</p>
              <div className="progress-bar">
                <div
                  className="progress-fill"
                  style={{ width: `${progress.percentage}%` }}
                />
              </div>
              <p className="progress-text">
                {Math.round(progress.percentage)}% -
                {formatBytes(progress.downloaded_bytes)} / {formatBytes(progress.total_bytes)}
              </p>
            </div>
          )}

          {state === 'ready' && (
            <div className="status ready">
              <p>Version: v{updateInfo?.installed_version}</p>
              <button onClick={launchGame} className="launch-btn">
                LAUNCH
              </button>
            </div>
          )}
        </section>
      </main>

      <footer className="launcher-footer">
        <span>2025 NoMag Games</span>
        <div className="footer-links">
          <a href="https://github.com/nomaggames/astra/releases" target="_blank" rel="noopener noreferrer">Patch Notes</a>
          <a href="https://wiki.astragame.online" target="_blank" rel="noopener noreferrer">Wiki</a>
          <a href="https://client.astragame.online" target="_blank" rel="noopener noreferrer">Play in Browser</a>
        </div>
      </footer>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export default App;
