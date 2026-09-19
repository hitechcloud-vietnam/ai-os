# HiTechCloud AI OS — Systemd Service Files
# ═══════════════════════════════════════════════════════════════
# Copy to /etc/systemd/system/ and run:
#   sudo systemctl daemon-reload
#   sudo systemctl enable hitechcloud-*
#   sudo systemctl start hitechcloud-*

# ── AI Gateway ────────────────────────────────────────────────
# [Unit]
# Description=HiTechCloud AI Gateway
# After=network.target postgresql.service redis.service
# 
# [Service]
# Type=simple
# User=hitechcloud
# Group=hitechcloud
# WorkingDirectory=/opt/hitechcloud
# ExecStart=/opt/hitechcloud/bin/hitechcloud-ai-gateway
# EnvironmentFile=/opt/hitechcloud/.env
# Restart=always
# RestartSec=5
# 
# [Install]
# WantedBy=multi-user.target

# ── MCP Gateway ───────────────────────────────────────────────
# [Unit]
# Description=HiTechCloud MCP Gateway
# After=network.target postgresql.service redis.service
# 
# [Service]
# Type=simple
# User=hitechcloud
# Group=hitechcloud
# WorkingDirectory=/opt/hitechcloud
# ExecStart=/opt/hitechcloud/bin/hitechcloud-mcp-gateway
# EnvironmentFile=/opt/hitechcloud/.env
# Restart=always
# RestartSec=5
# 
# [Install]
# WantedBy=multi-user.target

# ── A2A Gateway ───────────────────────────────────────────────
# [Unit]
# Description=HiTechCloud A2A Gateway
# After=network.target postgresql.service
# 
# [Service]
# Type=simple
# User=hitechcloud
# Group=hitechcloud
# WorkingDirectory=/opt/hitechcloud
# ExecStart=/opt/hitechcloud/bin/hitechcloud-a2a-gateway
# EnvironmentFile=/opt/hitechcloud/.env
# Restart=always
# RestartSec=5
# 
# [Install]
# WantedBy=multi-user.target