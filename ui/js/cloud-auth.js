// Cloud Authentication Module (GitHub Gist + Google Drive)
// Usage: CloudAuth.showModal() to open login modal

const CloudAuth = (function() {
  const OAUTH_PROXY = 'https://keyviewer-oauth.rudghrnt.workers.dev';
  const COOKIE_KEY_GITHUB = 'kv_github_token';
  const COOKIE_KEY_GOOGLE = 'kv_google_auth';
  const COOKIE_MAX_AGE = 31536000; // 1 year in seconds
  
  let currentProvider = null;
  let modalEl = null;
  
  // ============= Cookie Helpers =============
  function getCookie(name) {
    const match = document.cookie.match(new RegExp('(^| )' + name + '=([^;]+)'));
    return match ? decodeURIComponent(match[2]) : null;
  }
  
  function setCookie(name, value, maxAge = COOKIE_MAX_AGE) {
    document.cookie = `${name}=${encodeURIComponent(value)}; path=/; max-age=${maxAge}; SameSite=Strict`;
  }
  
  function deleteCookie(name) {
    document.cookie = `${name}=; path=/; max-age=0; SameSite=Strict`;
  }
  
  // ============= Token Management =============
  function getGitHubToken() {
    return getCookie(COOKIE_KEY_GITHUB);
  }
  
  function setGitHubToken(token) {
    setCookie(COOKIE_KEY_GITHUB, token);
  }
  
  function getGoogleAuth() {
    try {
      const value = getCookie(COOKIE_KEY_GOOGLE);
      return value ? JSON.parse(value) : null;
    } catch { return null; }
  }
  
  function setGoogleAuth(auth) {
    setCookie(COOKIE_KEY_GOOGLE, JSON.stringify(auth));
  }
  
  function clearAuth() {
    deleteCookie(COOKIE_KEY_GITHUB);
    deleteCookie(COOKIE_KEY_GOOGLE);
    currentProvider = null;
  }
  
  function getCurrentProvider() {
    if (getGitHubToken()) return 'github';
    if (getGoogleAuth()) return 'google';
    return null;
  }
  
  // ============= Google Token Refresh =============
  async function refreshGoogleToken() {
    const auth = getGoogleAuth();
    if (!auth || !auth.refresh_token) return null;
    
    try {
      const res = await fetch(`${OAUTH_PROXY}/refresh/google`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refresh_token: auth.refresh_token })
      });
      const data = await res.json();
      if (data.access_token) {
        auth.access_token = data.access_token;
        setGoogleAuth(auth);
        return auth.access_token;
      }
    } catch (e) {
      console.error('Token refresh failed:', e);
    }
    return null;
  }
  
  // ============= OAuth Flow =============
  async function startGitHubAuth() {
    // Check if we have a saved valid token
    const token = getGitHubToken();
    if (token) {
      console.log('[OAuth] Found saved GitHub token, validating...');
      hideModal();
      
      try {
        const res = await fetch('https://api.github.com/user', {
          headers: {
            'Authorization': `token ${token}`,
            'Accept': 'application/vnd.github.v3+json'
          }
        });
        
        if (res.ok) {
          console.log('[OAuth] GitHub token valid, using existing session');
          currentProvider = 'github';
          const user = await res.json();
          window.dispatchEvent(new CustomEvent('github-auth-success', { detail: user }));
          return;
        }
      } catch (e) {
        console.log('[OAuth] GitHub token validation failed:', e);
      }
      console.log('[OAuth] Token invalid, redirecting to OAuth...');
    }
    
    // No token or validation failed - redirect to OAuth
    const port = location.port || '8000';
    const path = location.pathname;
    window.location.href = `${OAUTH_PROXY}/auth/github?port=${port}&path=${path}`;
  }
  
  async function startGoogleAuth() {
    // Check if we have a saved refresh token
    const auth = getGoogleAuth();
    if (auth && auth.refresh_token) {
      console.log('[OAuth] Found saved Google refresh token, attempting refresh...');
      hideModal();
      
      // Try to refresh the token
      const newToken = await refreshGoogleToken();
      if (newToken) {
        console.log('[OAuth] Google token refreshed successfully');
        currentProvider = 'google';
        // Dispatch event to notify the page
        window.dispatchEvent(new CustomEvent('google-auth-success', { detail: getGoogleAuth() }));
        return;
      }
      console.log('[OAuth] Refresh failed, redirecting to OAuth...');
    }
    
    // No refresh token or refresh failed - redirect to OAuth
    const port = location.port || '8000';
    const path = location.pathname;
    window.location.href = `${OAUTH_PROXY}/auth/google?port=${port}&path=${path}`;
  }
  
  function handleCallback() {
    const params = new URLSearchParams(location.search);
    
    // GitHub callback
    const githubToken = params.get('github_token');
    if (githubToken) {
      setGitHubToken(githubToken);
      currentProvider = 'github';
      // Clean URL
      history.replaceState({}, '', location.pathname);
      return true;
    }
    
    // Google callback
    const googleToken = params.get('google_token');
    if (googleToken) {
      try {
        const auth = JSON.parse(atob(googleToken));
        setGoogleAuth(auth);
        currentProvider = 'google';
        history.replaceState({}, '', location.pathname);
        return true;
      } catch (e) {
        console.error('Failed to parse Google token:', e);
      }
    }
    
    return false;
  }
  
  // ============= Login Modal =============
  function createModal() {
    if (modalEl) return modalEl;
    
    modalEl = document.createElement('div');
    modalEl.id = 'cloud-auth-modal';
    modalEl.innerHTML = `
      <div class="cam-overlay"></div>
      <div class="cam-dialog">
        <div class="cam-header">
          <h2>☁️ 클라우드 로그인</h2>
          <button class="cam-close">&times;</button>
        </div>
        <div class="cam-body">
          <p class="cam-desc">설정과 프리셋을 클라우드에 저장하고 동기화하세요</p>
          
          <button class="cam-btn cam-btn-github" data-provider="github">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
              <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
            </svg>
            GitHub로 로그인
          </button>
          
          <button class="cam-btn cam-btn-google" data-provider="google">
            <svg viewBox="0 0 24 24" width="20" height="20">
              <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
              <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
              <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
              <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
            </svg>
            Google로 로그인
          </button>
          
          <div class="cam-divider"><span>저장 위치</span></div>
          
          <div class="cam-info">
            <div class="cam-info-item">
              <strong>GitHub</strong>
              <span>Gist (비공개)</span>
            </div>
            <div class="cam-info-item">
              <strong>Google</strong>
              <span>Drive (앱 폴더)</span>
            </div>
          </div>
        </div>
      </div>
    `;
    
    // Add styles
    const style = document.createElement('style');
    style.textContent = `
      #cloud-auth-modal { display:none; position:fixed; inset:0; z-index:99999; }
      #cloud-auth-modal.show { display:flex; align-items:center; justify-content:center; }
      .cam-overlay { position:absolute; inset:0; background:rgba(0,0,0,0.7); backdrop-filter:blur(4px); }
      .cam-dialog { position:relative; background:#1a1d24; border-radius:16px; width:380px; max-width:90vw; box-shadow:0 20px 60px rgba(0,0,0,0.5); border:1px solid rgba(255,255,255,0.1); overflow:hidden; }
      .cam-header { display:flex; align-items:center; justify-content:space-between; padding:20px 24px; border-bottom:1px solid rgba(255,255,255,0.1); }
      .cam-header h2 { margin:0; font-size:18px; color:#fff; font-weight:600; }
      .cam-close { background:none; border:none; color:#888; font-size:24px; cursor:pointer; padding:0; line-height:1; }
      .cam-close:hover { color:#fff; }
      .cam-body { padding:24px; }
      .cam-desc { color:#888; font-size:14px; margin:0 0 24px; text-align:center; }
      .cam-btn { display:flex; align-items:center; justify-content:center; gap:12px; width:100%; padding:14px; border-radius:10px; border:none; font-size:15px; font-weight:500; cursor:pointer; transition:all 0.2s; margin-bottom:12px; }
      .cam-btn-github { background:#24292e; color:#fff; }
      .cam-btn-github:hover { background:#2f363d; }
      .cam-btn-google { background:#fff; color:#333; }
      .cam-btn-google:hover { background:#f1f1f1; }
      .cam-divider { display:flex; align-items:center; gap:12px; margin:24px 0 16px; color:#555; font-size:12px; }
      .cam-divider::before, .cam-divider::after { content:''; flex:1; height:1px; background:rgba(255,255,255,0.1); }
      .cam-info { display:grid; grid-template-columns:1fr 1fr; gap:12px; }
      .cam-info-item { background:rgba(255,255,255,0.05); padding:12px; border-radius:8px; text-align:center; }
      .cam-info-item strong { display:block; color:#fff; font-size:13px; margin-bottom:4px; }
      .cam-info-item span { color:#666; font-size:12px; }
    `;
    document.head.appendChild(style);
    document.body.appendChild(modalEl);
    
    // Event listeners
    modalEl.querySelector('.cam-overlay').addEventListener('click', hideModal);
    modalEl.querySelector('.cam-close').addEventListener('click', hideModal);
    modalEl.querySelector('.cam-btn-github').addEventListener('click', startGitHubAuth);
    modalEl.querySelector('.cam-btn-google').addEventListener('click', startGoogleAuth);
    
    return modalEl;
  }
  
  function showModal() {
    createModal();
    modalEl.classList.add('show');
  }
  
  function hideModal() {
    if (modalEl) modalEl.classList.remove('show');
  }
  
  // ============= Google Drive API =============
  const GoogleDrive = {
    FOLDER_NAME: 'KeyViewer',
    CONFIG_FILENAME: 'config.json',
    _folderId: null,
    
    async getHeaders() {
      const auth = getGoogleAuth();
      if (!auth) throw new Error('Not authenticated');
      return { 'Authorization': `Bearer ${auth.access_token}` };
    },
    
    // Get or create KeyViewer folder
    async getFolderId() {
      if (this._folderId) return this._folderId;
      
      const headers = await this.getHeaders();
      
      // Search for existing folder
      const searchRes = await fetch(
        `https://www.googleapis.com/drive/v3/files?q=name='${this.FOLDER_NAME}' and mimeType='application/vnd.google-apps.folder' and 'root' in parents and trashed=false`,
        { headers }
      );
      const data = await searchRes.json();
      
      if (data.files?.length > 0) {
        this._folderId = data.files[0].id;
        return this._folderId;
      }
      
      // Create folder
      const createRes = await fetch('https://www.googleapis.com/drive/v3/files', {
        method: 'POST',
        headers: { ...headers, 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: this.FOLDER_NAME,
          mimeType: 'application/vnd.google-apps.folder',
          parents: ['root']
        })
      });
      const folder = await createRes.json();
      this._folderId = folder.id;
      return this._folderId;
    },
    
    // Find file by name in KeyViewer folder
    async findFile(filename) {
      const headers = await this.getHeaders();
      const folderId = await this.getFolderId();
      
      const searchRes = await fetch(
        `https://www.googleapis.com/drive/v3/files?q=name='${filename}' and '${folderId}' in parents and trashed=false`,
        { headers }
      );
      const data = await searchRes.json();
      return data.files?.[0] || null;
    },
    
    // Save JSON file (create or update)
    async saveFile(filename, content) {
      const headers = await this.getHeaders();
      const folderId = await this.getFolderId();
      const existing = await this.findFile(filename);
      
      const metadata = { 
        name: filename, 
        mimeType: 'application/json',
        parents: existing ? undefined : [folderId]
      };
      const body = JSON.stringify(content, null, 2);
      
      // Multipart upload
      const boundary = '-------KeyViewerBoundary';
      const multipartBody = 
        `--${boundary}\r\n` +
        `Content-Type: application/json; charset=UTF-8\r\n\r\n` +
        `${JSON.stringify(metadata)}\r\n` +
        `--${boundary}\r\n` +
        `Content-Type: application/json\r\n\r\n` +
        `${body}\r\n` +
        `--${boundary}--`;
      
      const url = existing 
        ? `https://www.googleapis.com/upload/drive/v3/files/${existing.id}?uploadType=multipart`
        : 'https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart';
      
      const res = await fetch(url, {
        method: existing ? 'PATCH' : 'POST',
        headers: { 
          ...headers, 
          'Content-Type': `multipart/related; boundary=${boundary}` 
        },
        body: multipartBody
      });
      return res.json();
    },
    
    // Load JSON file
    async loadFile(filename) {
      const headers = await this.getHeaders();
      const file = await this.findFile(filename);
      if (!file) return null;
      
      const res = await fetch(
        `https://www.googleapis.com/drive/v3/files/${file.id}?alt=media`,
        { headers }
      );
      return res.json();
    },
    
    // Delete file
    async deleteFile(filename) {
      const headers = await this.getHeaders();
      const file = await this.findFile(filename);
      if (!file) return false;
      
      await fetch(`https://www.googleapis.com/drive/v3/files/${file.id}`, {
        method: 'DELETE',
        headers
      });
      return true;
    },
    
    // List all files in KeyViewer folder
    async listFiles() {
      const headers = await this.getHeaders();
      const folderId = await this.getFolderId();
      
      const res = await fetch(
        `https://www.googleapis.com/drive/v3/files?q='${folderId}' in parents and trashed=false&fields=files(id,name,modifiedTime)`,
        { headers }
      );
      const data = await res.json();
      return data.files || [];
    },
    
    // Convenience methods
    async saveConfig(config) {
      return this.saveFile(this.CONFIG_FILENAME, config);
    },
    
    async loadConfig() {
      return this.loadFile(this.CONFIG_FILENAME);
    }
  };
  
  // ============= Public API =============
  return {
    showModal,
    hideModal,
    handleCallback,
    getCurrentProvider,
    getGitHubToken,
    getGoogleAuth,
    refreshGoogleToken,
    clearAuth,
    GoogleDrive,
    
    // Check auth status on load
    init() {
      handleCallback();
      currentProvider = getCurrentProvider();
      return currentProvider;
    }
  };
})();

// Auto-init
if (typeof window !== 'undefined') {
  window.CloudAuth = CloudAuth;
}

