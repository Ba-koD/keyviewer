// Cloudflare Worker for KeyViewer OAuth (GitHub + Google) — Secured
//
// Required env vars (wrangler.toml 또는 Dashboard):
//   GITHUB_CLIENT_ID
//   GITHUB_CLIENT_SECRET
//   GOOGLE_CLIENT_ID
//   GOOGLE_CLIENT_SECRET
//   STATE_SECRET        ← 신규: HMAC 서명용 (32자 이상 랜덤 문자열)
//
// STATE_SECRET 생성 예시:
//   node -e "console.log(require('crypto').randomBytes(32).toString('hex'))"
//
// 배포: wrangler deploy

// ============= Origin 허용 목록 =============
const ALLOWED_ORIGIN_RE = /^http:\/\/(localhost|127\.0\.0\.1)(:\d{1,5})?$/;

function getCorsHeaders(request) {
  const origin = request.headers.get('Origin') || '';
  if (ALLOWED_ORIGIN_RE.test(origin)) {
    return {
      'Access-Control-Allow-Origin': origin,
      'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    };
  }
  return {};
}

// ============= HMAC 서명 State =============
async function hmacSign(secret, message) {
  const key = await crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const sig = await crypto.subtle.sign('HMAC', key, new TextEncoder().encode(message));
  return [...new Uint8Array(sig)].map(b => b.toString(16).padStart(2, '0')).join('');
}

async function createSignedState(data, secret) {
  data.ts = Date.now();
  const b64 = btoa(JSON.stringify(data));
  const sig = await hmacSign(secret, b64);
  return b64 + '.' + sig;
}

async function verifyState(state, secret) {
  if (!state || typeof state !== 'string') return null;

  const dot = state.lastIndexOf('.');
  if (dot === -1) return null;

  const b64 = state.substring(0, dot);
  const sig = state.substring(dot + 1);

  // HMAC 검증
  const expected = await hmacSign(secret, b64);
  if (sig.length !== expected.length) return null;

  // 타이밍 공격 방지 (constant-time 비교)
  let diff = 0;
  for (let i = 0; i < sig.length; i++) {
    diff |= sig.charCodeAt(i) ^ expected.charCodeAt(i);
  }
  if (diff !== 0) return null;

  try {
    const data = JSON.parse(atob(b64));
    // 10분 만료
    if (!data.ts || Date.now() - data.ts > 10 * 60 * 1000) return null;
    return data;
  } catch {
    return null;
  }
}

// ============= 입력 검증 =============
function validatePort(port) {
  if (!/^\d{1,5}$/.test(port)) return false;
  const n = parseInt(port, 10);
  return n >= 1 && n <= 65535;
}

function validatePath(path) {
  return /^\/[a-zA-Z0-9\/_.-]*$/.test(path) && path.length < 200;
}

// ============= 안전한 리다이렉트 페이지 =============
// 보안 포인트:
//   1) port, path 화이트리스트 검증 → XSS 차단
//   2) JSON.stringify로 JS 문자열 이스케이핑 (defense-in-depth)
//   3) Fragment(#) 사용 → 토큰이 Referer 헤더, 서버 로그에 미노출
function redirectPage(port, path, tokenKey, tokenValue) {
  if (!validatePort(port)) return new Response('Invalid port', { status: 400 });
  if (!validatePath(path)) return new Response('Invalid path', { status: 400 });

  const providerName = tokenKey.includes('github') ? 'GitHub' : 'Google';
  const jsPort = JSON.stringify(String(port));
  const jsPath = JSON.stringify(path);
  const jsKey = JSON.stringify(tokenKey);
  const jsValue = JSON.stringify(encodeURIComponent(tokenValue));

  const html = `<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>로그인 성공</title></head>
<body style="background:#0d1117;color:#fff;font-family:system-ui;display:flex;align-items:center;justify-content:center;height:100vh;margin:0">
<div style="text-align:center">
  <h2>\u2705 ${providerName} 로그인 성공!</h2>
  <p>KeyViewer로 이동 중...</p>
</div>
<script>
location.href="http://localhost:"+${jsPort}+${jsPath}+"#"+${jsKey}+"="+${jsValue};
</script>
</body></html>`;

  return new Response(html, {
    headers: { 'Content-Type': 'text/html; charset=utf-8' }
  });
}

// ============= GitHub OAuth =============
async function startGitHubAuth(url, env, callbackPath) {
  const port = url.searchParams.get('port') || '8000';
  const path = url.searchParams.get('path') || '/control';

  if (!validatePort(port) || !validatePath(path)) {
    return new Response('Invalid parameters', { status: 400 });
  }

  const state = await createSignedState(
    { provider: 'github', port, path },
    env.STATE_SECRET
  );

  const githubUrl = 'https://github.com/login/oauth/authorize?' + new URLSearchParams({
    client_id: env.GITHUB_CLIENT_ID,
    redirect_uri: `${url.origin}${callbackPath}`,
    scope: 'gist',
    state
  });

  return Response.redirect(githubUrl, 302);
}

async function handleGitHubCallback(url, env) {
  const code = url.searchParams.get('code');
  const stateRaw = url.searchParams.get('state');

  if (!code || !stateRaw) return new Response('Missing params', { status: 400 });

  const stateData = await verifyState(stateRaw, env.STATE_SECRET);
  if (!stateData) return new Response('Invalid or expired state', { status: 403 });

  const { port, path } = stateData;

  const tokenRes = await fetch('https://github.com/login/oauth/access_token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    body: JSON.stringify({
      client_id: env.GITHUB_CLIENT_ID,
      client_secret: env.GITHUB_CLIENT_SECRET,
      code
    })
  });

  const tokenData = await tokenRes.json();
  if (tokenData.error) return new Response('Auth error: ' + tokenData.error, { status: 400 });

  return redirectPage(port, path, 'github_token', tokenData.access_token);
}

// ============= Google OAuth =============
async function startGoogleAuth(url, env) {
  const port = url.searchParams.get('port') || '8000';
  const path = url.searchParams.get('path') || '/control';

  if (!validatePort(port) || !validatePath(path)) {
    return new Response('Invalid parameters', { status: 400 });
  }

  const state = await createSignedState(
    { provider: 'google', port, path },
    env.STATE_SECRET
  );

  const scopes = [
    'https://www.googleapis.com/auth/drive.file',
    'https://www.googleapis.com/auth/userinfo.email'
  ].join(' ');

  const googleUrl = 'https://accounts.google.com/o/oauth2/v2/auth?' + new URLSearchParams({
    client_id: env.GOOGLE_CLIENT_ID,
    redirect_uri: `${url.origin}/callback/google`,
    response_type: 'code',
    scope: scopes,
    state,
    access_type: 'offline',
    prompt: 'consent'
  });

  return Response.redirect(googleUrl, 302);
}

async function handleGoogleCallback(url, env) {
  const code = url.searchParams.get('code');
  const stateRaw = url.searchParams.get('state');
  const error = url.searchParams.get('error');

  if (error) return new Response('Auth denied: ' + error, { status: 400 });
  if (!code || !stateRaw) return new Response('Missing params', { status: 400 });

  const stateData = await verifyState(stateRaw, env.STATE_SECRET);
  if (!stateData) return new Response('Invalid or expired state', { status: 403 });

  const { port, path } = stateData;

  const tokenRes = await fetch('https://oauth2.googleapis.com/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      client_id: env.GOOGLE_CLIENT_ID,
      client_secret: env.GOOGLE_CLIENT_SECRET,
      code,
      grant_type: 'authorization_code',
      redirect_uri: `${url.origin}/callback/google`
    })
  });

  const tokenData = await tokenRes.json();
  if (tokenData.error) {
    return new Response('Token error: ' + (tokenData.error_description || tokenData.error), { status: 400 });
  }

  const userRes = await fetch('https://www.googleapis.com/oauth2/v2/userinfo', {
    headers: { Authorization: `Bearer ${tokenData.access_token}` }
  });
  const userData = await userRes.json();

  const payload = btoa(JSON.stringify({
    access_token: tokenData.access_token,
    refresh_token: tokenData.refresh_token,
    email: userData.email,
    picture: userData.picture
  }));

  return redirectPage(port, path, 'google_token', payload);
}

// ============= Google Token Refresh =============
async function handleGoogleRefresh(request, env, corsHeaders) {
  // Origin 이중 검증 (CORS 헤더와 별도)
  const origin = request.headers.get('Origin') || '';
  if (!ALLOWED_ORIGIN_RE.test(origin)) {
    return new Response(JSON.stringify({ error: 'Forbidden origin' }), {
      status: 403,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  try {
    const body = await request.json();
    const refreshToken = body?.refresh_token;

    if (!refreshToken || typeof refreshToken !== 'string') {
      return new Response(JSON.stringify({ error: 'Missing refresh_token' }), {
        status: 400,
        headers: { ...corsHeaders, 'Content-Type': 'application/json' }
      });
    }

    const tokenRes = await fetch('https://oauth2.googleapis.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        client_id: env.GOOGLE_CLIENT_ID,
        client_secret: env.GOOGLE_CLIENT_SECRET,
        refresh_token: refreshToken,
        grant_type: 'refresh_token'
      })
    });

    const tokenData = await tokenRes.json();
    return new Response(JSON.stringify(tokenData), {
      headers: { ...corsHeaders, 'Content-Type': 'application/json' }
    });
  } catch {
    return new Response(JSON.stringify({ error: 'Invalid request' }), {
      status: 400,
      headers: { ...corsHeaders, 'Content-Type': 'application/json' }
    });
  }
}

// ============= Main Handler =============
export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: getCorsHeaders(request) });
    }

    try {
      // --- Legacy GitHub OAuth (하위호환) ---
      if (url.pathname === '/auth') return startGitHubAuth(url, env, '/callback');
      if (url.pathname === '/callback') return handleGitHubCallback(url, env);

      // --- GitHub OAuth ---
      if (url.pathname === '/auth/github') return startGitHubAuth(url, env, '/callback/github');
      if (url.pathname === '/callback/github') return handleGitHubCallback(url, env);

      // --- Google OAuth ---
      if (url.pathname === '/auth/google') return startGoogleAuth(url, env);
      if (url.pathname === '/callback/google') return handleGoogleCallback(url, env);

      // --- Google Token Refresh ---
      if (url.pathname === '/refresh/google' && request.method === 'POST') {
        return handleGoogleRefresh(request, env, getCorsHeaders(request));
      }

      return new Response(`KeyViewer OAuth Proxy (secured)

Endpoints:
  GET  /auth            - GitHub OAuth (legacy)
  GET  /auth/github     - GitHub OAuth
  GET  /auth/google     - Google OAuth
  POST /refresh/google  - Refresh Google token

Query params: ?port=8000&path=/control
`, { headers: { 'Content-Type': 'text/plain' } });
    } catch (e) {
      console.error('Worker error:', e);
      return new Response('Internal Server Error', { status: 500 });
    }
  }
};
