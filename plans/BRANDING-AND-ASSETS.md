# Đặc tả Tài nguyên Nhận diện Thương hiệu & Biểu tượng (Branding & Assets Specification)

Tài liệu này đặc tả quy chuẩn sử dụng bộ tài nguyên biểu tượng (icons), favicon và logo chính thức của **HiTechCloud** trên toàn bộ hệ sinh thái **HiTechCloud Agent Platform** (Web Dashboard, VS Code Extension, Rust CLI, Microsoft 365 Copilot / Teams Apps, Universal Package Registry, và Marketplace).

---

## 1. Danh mục URL Logo & Biểu tượng Chính thức (Official Brand Assets)

| STT | Tài nguyên | Kích thước / Định dạng | URL Chính Thức | Mục đích Sử dụng |
|---|---|---|---|---|
| 1 | **Logo Chính thức (Vector SVG)** | Vector SVG | `https://hitechcloud.vn/wp-content/uploads/2025/01/hitechcloudvn.svg` | Header / Sidebar Logo trên Web Dashboard (`mcp.hitechcloud.vn`), Marketplace (`marketplace-mcp.hitechcloud.vn`), Webview VS Code, Tài liệu Documentation & README |
| 2 | **Favicon ICO** | Multi-size `.ico` | `https://hitechcloud.vn/wp-content/themes/hitechcloud-news-2026/favicon/icon-v2.ico?v=2` | Favicon cho Web Dashboard (`mcp.hitechcloud.vn`, `marketplace-mcp.hitechcloud.vn`, `registry-mcp.hitechcloud.vn`), Windows Executable Icon (`.exe` resource) |
| 3 | **Icon Small (32x32)** | `32x32` PNG | `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-32x32.png?v=2&v=3` | Web Favicon 32x32, VS Code Activity Bar Container Icon, Microsoft Teams / Copilot Outline Icon (`icons.outline`), Status Bar Brand Badge |
| 4 | **Icon Medium (192x192)** | `192x192` PNG | `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-192x192.png?v=2&v=3` | PWA Web App Manifest Icon (`192x192`), Microsoft 365 Copilot / Teams App Color Icon (`icons.color`), Android Touch Icon, Notification Badge |
| 5 | **Icon High-Res (270x270)** | `270x270` PNG | `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-270x270.png` | VS Code Extension Icon (`package.json` -> `icon: "media/icon.png"`), Apple Touch Icon, Windows Metro Tile (`msapplication-TileImage`), Marketplace & Registry Package Default Badge |

---

## 2. Quy chuẩn Ứng dụng theo Từng Phân hệ

### 2.1. Web Dashboard & Web Portals (`mcp.hitechcloud.vn`, `marketplace-mcp.hitechcloud.vn`)
Trong `index.html` và Web App Manifest của Vite SPA:

```html
<!-- Official Vector Logo in Navbar/Sidebar Header -->
<img src="https://hitechcloud.vn/wp-content/uploads/2025/01/hitechcloudvn.svg" alt="HiTechCloud" class="h-8 w-auto">

<!-- Favicons & Browser Tab Icons -->
<link rel="icon" href="https://hitechcloud.vn/wp-content/themes/hitechcloud-news-2026/favicon/icon-v2.ico?v=2" sizes="any">
<link rel="icon" type="image/png" sizes="32x32" href="https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-32x32.png?v=2&v=3">
<link rel="apple-touch-icon" sizes="270x270" href="https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-270x270.png">
<meta name="msapplication-TileImage" content="https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-270x270.png">

<!-- PWA Manifest -->
<link rel="manifest" href="/manifest.json">
```

Tệp `manifest.json`:
```json
{
  "name": "HiTechCloud Agent Platform",
  "short_name": "HiTechCloud MCP",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#090d16",
  "theme_color": "#0f172a",
  "icons": [
    {
      "src": "https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-192x192.png?v=2&v=3",
      "sizes": "192x192",
      "type": "image/png"
    },
    {
      "src": "https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-270x270.png",
      "sizes": "270x270",
      "type": "image/png"
    }
  ]
}
```

---

### 2.2. VS Code Extension Riêng biệt (`hitechcloud-agent-platform`)
- **Package Icon**: Tệp `media/icon.png` (sử dụng độ phân giải cao `270x270` từ `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-270x270.png`).
- **Activity Bar Icon**: Tệp `resources/icon.svg` / `resources/icon-32.png` (sử dụng từ `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-32x32.png?v=2&v=3`).
- **Khởi tạo trong `package.json`**:
```json
{
  "name": "hitechcloud-agent-platform",
  "displayName": "HiTechCloud Agent Platform",
  "icon": "media/icon.png",
  "contributes": {
    "viewsContainers": {
      "activitybar": [
        {
          "id": "hitechcloud-explorer",
          "title": "HiTechCloud MCP",
          "icon": "resources/icon.svg"
        }
      ]
    }
  }
}
```

---

### 2.3. Microsoft 365 Copilot & Teams App Manifest (`manifest.json`)
Theo chuẩn Teams App / M365 Copilot Declarative Agent:
- `icons.color`: Kích thước `192x192` PNG (`https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-192x192.png?v=2&v=3`).
- `icons.outline`: Kích thước `32x32` PNG (`https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-32x32.png?v=2&v=3`).

```json
{
  "$schema": "https://developer.microsoft.com/en-us/json-schemas/teams/v1.16/MicrosoftTeams.schema.json",
  "manifestVersion": "1.16",
  "version": "1.0.0",
  "id": "00000000-0000-0000-0000-000000000000",
  "packageName": "vn.hitechcloud.agent.m365",
  "developer": {
    "name": "HiTechCloud Enterprise",
    "websiteUrl": "https://hitechcloud.vn",
    "privacyUrl": "https://hitechcloud.vn/privacy",
    "termsOfUseUrl": "https://hitechcloud.vn/terms"
  },
  "icons": {
    "color": "color.png",
    "outline": "outline.png"
  },
  "name": {
    "short": "HiTechCloud",
    "full": "HiTechCloud AI Agent Platform & Tools"
  }
}
```

---

### 2.4. Rust CLI Binary (`hitechcloud.exe` Windows Resource)
Khi build nhị phân Rust cho Windows (`x86_64-pc-windows-msvc`), nhúng tệp `.ico` qua `winres`:

Tệp `crates/hitechcloud-cli/build.rs`:
```rust
#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon-v2.ico");
    res.set("ProductName", "HiTechCloud Agent CLI");
    res.set("CompanyName", "HiTechCloud");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
```

---

### 2.5. Universal Package Registry (`registry-mcp.hitechcloud.vn`) & AI Marketplace
Mỗi gói Package HCP (`.hcp`) hoặc MCP Server/Skill khi publish lên Registry nếu không đính kèm icon riêng sẽ mặc định sử dụng fallback brand asset:
- Logo mặc định của Provider HiTechCloud: `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-270x270.png`
- Verified Publisher Badge: Icon tích xanh kèm biểu tượng HiTechCloud 32x32.
