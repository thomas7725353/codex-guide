const OWNER = "thomas7725353";
const REPO = "codex-guide";
const UPSTREAM_REPO = `${OWNER}/${REPO}`;
const CC_SWITCH_REPO = "SaladDay/cc-switch-cli";
const GITHUB_RAW_BASE = `https://raw.githubusercontent.com/${UPSTREAM_REPO}/main`;
const GITHUB_RELEASE_LATEST = `https://github.com/${UPSTREAM_REPO}/releases/latest/download`;
const CODEX_APP_WINDOWS =
  "https://get.microsoft.com/installer/download/9PLM9XGG6VKS?cid=website_cta_psi";
const CODEX_INSTALL_PS1 = "https://chatgpt.com/codex/install.ps1";
const CODEX_INSTALL_SH = "https://chatgpt.com/codex/install.sh";
const CC_SWITCH_INSTALL_SH =
  "https://github.com/SaladDay/cc-switch-cli/releases/latest/download/install.sh";
const CC_SWITCH_WINDOWS_ZIP =
  "https://github.com/SaladDay/cc-switch-cli/releases/latest/download/cc-switch-cli-windows-x64.zip";

const RELEASE_ASSETS = new Set([
  "codex-guide-windows-x64.zip",
  "codex-guide-windows-x64.exe",
  "codex-guide-macos-arm64",
  "codex-guide-macos-x64",
]);

const SECURITY_HEADERS = {
  "content-security-policy":
    "default-src 'self'; style-src 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'",
  "referrer-policy": "no-referrer",
  "x-content-type-options": "nosniff",
};

export default {
  async fetch(request) {
    const url = new URL(request.url);
    if (request.method !== "GET" && request.method !== "HEAD") {
      return new Response("Method Not Allowed", { status: 405 });
    }

    switch (url.pathname) {
      case "/":
      case "/index.html":
        return htmlResponse(renderHome(url.origin));
      case "/install.ps1":
        return proxyText(`${GITHUB_RAW_BASE}/scripts/install.ps1`, "text/plain; charset=utf-8");
      case "/install.sh":
        return proxyText(`${GITHUB_RAW_BASE}/scripts/install.sh`, "text/plain; charset=utf-8");
      case "/download/windows":
      case "/download/windows.zip":
        return proxyBinary(`${GITHUB_RELEASE_LATEST}/codex-guide-windows-x64.zip`);
      case "/download/windows.exe":
        return proxyBinary(`${GITHUB_RELEASE_LATEST}/codex-guide-windows-x64.exe`);
      case "/download/macos-arm64":
        return proxyBinary(`${GITHUB_RELEASE_LATEST}/codex-guide-macos-arm64`);
      case "/download/macos-x64":
        return proxyBinary(`${GITHUB_RELEASE_LATEST}/codex-guide-macos-x64`);
      case "/codex/windows-app":
        return Response.redirect(CODEX_APP_WINDOWS, 302);
      case "/codex/install.ps1":
        return proxyText(CODEX_INSTALL_PS1, "text/plain; charset=utf-8");
      case "/codex/install.sh":
        return proxyText(CODEX_INSTALL_SH, "text/plain; charset=utf-8");
      case "/cc-switch/windows-x64.zip":
        return proxyBinary(CC_SWITCH_WINDOWS_ZIP);
      case "/cc-switch/install.sh":
        return proxyText(CC_SWITCH_INSTALL_SH, "text/plain; charset=utf-8");
      case "/api/health":
        return jsonResponse({
          ok: true,
          service: "codex-guide",
          domain: "guide.gorustai.com",
        });
      default:
        if (url.pathname.startsWith("/release/")) {
          return releaseAsset(url.pathname.slice("/release/".length));
        }
        return new Response("Not Found", { status: 404, headers: SECURITY_HEADERS });
    }
  },
};

async function releaseAsset(assetName) {
  if (!RELEASE_ASSETS.has(assetName)) {
    return new Response("Forbidden", { status: 403, headers: SECURITY_HEADERS });
  }
  return proxyBinary(`${GITHUB_RELEASE_LATEST}/${assetName}`);
}

async function proxyText(target, contentType) {
  const response = await fetch(target, cfFetchInit());
  return passThrough(response, {
    "content-type": contentType,
    "cache-control": "public, max-age=120",
  });
}

async function proxyBinary(target) {
  const response = await fetch(target, cfFetchInit());
  return passThrough(response, {
    "cache-control": "public, max-age=600",
  });
}

function cfFetchInit() {
  return {
    headers: {
      "user-agent": "codex-guide-worker/0.1",
      accept: "*/*",
    },
    cf: {
      cacheEverything: true,
      cacheTtl: 600,
    },
  };
}

function passThrough(response, extraHeaders) {
  const headers = new Headers(response.headers);
  for (const [key, value] of Object.entries(extraHeaders)) {
    headers.set(key, value);
  }
  headers.set("access-control-allow-origin", "*");
  headers.set("x-content-type-options", "nosniff");
  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

function htmlResponse(body) {
  return new Response(body, {
    headers: {
      ...SECURITY_HEADERS,
      "content-type": "text/html; charset=utf-8",
      "cache-control": "public, max-age=120",
    },
  });
}

function jsonResponse(data) {
  return new Response(JSON.stringify(data), {
    headers: {
      ...SECURITY_HEADERS,
      "content-type": "application/json; charset=utf-8",
      "cache-control": "no-store",
    },
  });
}

function renderHome(origin) {
  const winZip = `${origin}/download/windows.zip`;
  const winScript = `powershell -NoProfile -ExecutionPolicy Bypass -Command "irm ${origin}/install.ps1 | iex"`;
  const macScript = `curl -fsSL ${origin}/install.sh | bash`;
  return `<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Codex Guide 一键安装</title>
  <style>
    :root {
      color-scheme: light;
      --bg: #f6f8fb;
      --panel: #ffffff;
      --text: #17202a;
      --muted: #5b6673;
      --line: #d9e0e8;
      --primary: #0f766e;
      --primary-dark: #115e59;
      --code: #111827;
      --accent: #b45309;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.6;
    }
    header {
      background: #12343b;
      color: #fff;
      padding: 44px 20px 36px;
    }
    .wrap { max-width: 980px; margin: 0 auto; }
    h1 { margin: 0 0 10px; font-size: clamp(30px, 5vw, 46px); line-height: 1.15; letter-spacing: 0; }
    h2 { margin: 0 0 14px; font-size: 24px; letter-spacing: 0; }
    h3 { margin: 18px 0 8px; font-size: 18px; letter-spacing: 0; }
    p { margin: 8px 0; }
    .lead { color: #dce7ea; font-size: 18px; max-width: 760px; }
    .main { padding: 28px 20px 44px; }
    .section {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 22px;
      margin: 18px 0;
    }
    .grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
    .step {
      border-left: 4px solid var(--primary);
      padding-left: 14px;
      margin: 14px 0;
    }
    .button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 44px;
      padding: 10px 16px;
      border-radius: 6px;
      background: var(--primary);
      color: #fff;
      text-decoration: none;
      font-weight: 700;
      margin: 8px 8px 8px 0;
    }
    .button.secondary { background: #334155; }
    .button:hover { background: var(--primary-dark); }
    code, pre {
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace;
      letter-spacing: 0;
    }
    pre {
      background: var(--code);
      color: #f8fafc;
      border-radius: 8px;
      padding: 14px;
      overflow-x: auto;
      white-space: pre-wrap;
      word-break: break-word;
    }
    .note {
      border: 1px solid #f2d399;
      background: #fff7ed;
      color: #653b09;
      border-radius: 8px;
      padding: 12px 14px;
    }
    .muted { color: var(--muted); }
    ul { padding-left: 22px; }
    footer { color: var(--muted); font-size: 14px; padding: 18px 0 0; }
    @media (max-width: 760px) {
      .grid { grid-template-columns: 1fr; }
      header { padding-top: 32px; }
      .section { padding: 18px; }
    }
  </style>
</head>
<body>
  <header>
    <div class="wrap">
      <h1>Codex Guide 一键安装</h1>
      <p class="lead">给 Windows 用户优先准备的中文入口。下载、安装、配置 Codex App、Codex CLI、cc-switch-cli 和 gorustai provider。</p>
    </div>
  </header>
  <main class="main">
    <div class="wrap">
      <section class="section">
        <h2>Windows 最简单：下载后双击</h2>
        <p>不会用 PowerShell 的用户直接用这个方式。</p>
        <a class="button" href="${winZip}">下载 Windows 安装包</a>
        <a class="button secondary" href="${origin}/codex/windows-app">Codex App 官方下载</a>
        <div class="step"><strong>第 1 步：</strong>下载 zip，然后解压。</div>
        <div class="step"><strong>第 2 步：</strong>双击 <code>安装Codex.bat</code>。</div>
        <div class="step"><strong>第 3 步：</strong>按窗口提示粘贴自己的 <code>OPENAI_API_KEY</code>。</div>
        <div class="note">不要把 API key 发给别人。安装器不会内置或上传 key，只写到用户自己电脑。</div>
      </section>

      <section class="section">
        <h2>Windows 一行安装</h2>
        <pre>${escapeHtml(winScript)}</pre>
        <p class="muted">这个命令会下载本项目安装器，再自动检查 Codex App、Codex CLI、cc-switch-cli 和配置文件。</p>
      </section>

      <section class="section">
        <h2>macOS 一行安装</h2>
        <pre>${escapeHtml(macScript)}</pre>
        <p class="muted">Apple Silicon 和 Intel Mac 会自动选择对应二进制。</p>
      </section>

      <section class="section grid">
        <div>
          <h2>安装完成后</h2>
          <pre>codex-guide launch-codex
codex-guide launch-cc-switch
codex doctor
cc-switch --app codex</pre>
        </div>
        <div>
          <h2>远程运维</h2>
          <p>电脑环境出问题时，让用户选择双击菜单里的“启动 Codex CLI”，或者运行：</p>
          <pre>codex-guide launch-codex</pre>
          <p class="muted">技术人员可以看这个窗口里的报错继续处理。</p>
        </div>
      </section>

      <section class="section">
        <h2>镜像下载地址</h2>
        <ul>
          <li>Windows 安装包：<a href="${origin}/download/windows.zip">${origin}/download/windows.zip</a></li>
          <li>Windows exe：<a href="${origin}/download/windows.exe">${origin}/download/windows.exe</a></li>
          <li>macOS ARM64：<a href="${origin}/download/macos-arm64">${origin}/download/macos-arm64</a></li>
          <li>macOS x64：<a href="${origin}/download/macos-x64">${origin}/download/macos-x64</a></li>
          <li>Codex CLI Windows：<a href="${origin}/codex/install.ps1">${origin}/codex/install.ps1</a></li>
          <li>Codex CLI macOS/Linux：<a href="${origin}/codex/install.sh">${origin}/codex/install.sh</a></li>
          <li>cc-switch Windows：<a href="${origin}/cc-switch/windows-x64.zip">${origin}/cc-switch/windows-x64.zip</a></li>
        </ul>
      </section>

      <section class="section">
        <h2>会写入的 Codex 配置</h2>
        <pre>model_provider = "gorustai"
model = "gpt-5.5"
review_model = "gpt-5.5"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = true
windows_wsl_setup_acknowledged = true

[model_providers.gorustai]
name = "OpenAI"
base_url = "https://gorustai.com"
wire_api = "responses"
requires_openai_auth = false
env_key = "OPENAI_API_KEY"

[features]
goals = true</pre>
      </section>

      <footer>
        <p>本页由 Cloudflare Worker 提供，用于中国用户更容易打开教程和安装入口。源码仓库保存在 GitHub：${UPSTREAM_REPO}。</p>
      </footer>
    </div>
  </main>
</body>
</html>`;
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
