-- Seed data for HiTechCloud AI OS

-- Default organization
INSERT INTO organizations (id, name, plan) VALUES
  ('a0000000-0000-0000-0000-000000000001', 'HiTechCloud', 'enterprise')
ON CONFLICT DO NOTHING;

-- Default admin user
INSERT INTO users (id, org_id, email) VALUES
  ('a0000000-0000-0000-0000-000000000002', 'a0000000-0000-0000-0000-000000000001', 'admin@hitechcloud.vn')
ON CONFLICT DO NOTHING;

-- Sample MCP servers
INSERT INTO mcp_servers (id, name, transport, namespace, visibility) VALUES
  ('github-mcp', 'GitHub MCP Server', 'stdio', 'github', 'public'),
  ('postgres-mcp', 'PostgreSQL MCP Server', 'stdio', 'postgres', 'public'),
  ('filesystem-mcp', 'Filesystem MCP Server', 'stdio', 'fs', 'public'),
  ('docker-mcp', 'Docker MCP Server', 'stdio', 'docker', 'public')
ON CONFLICT DO NOTHING;

-- Sample skills
INSERT INTO skills (id, scope, name, description) VALUES
  ('review-pr', 'public', 'Review PR', 'Dùng khi user nhắc review PR, kiểm tra pull request, hoặc yêu cầu áp chuẩn coding công ty.'),
  ('generate-tests', 'public', 'Generate Tests', 'Dùng khi user yêu cầu tạo unit test, integration test, hoặc test coverage cho code.'),
  ('refactor-code', 'public', 'Refactor Code', 'Dùng khi user yêu cầu refactor, tái cấu trúc, hoặc cải thiện chất lượng code.'),
  ('security-audit', 'public', 'Security Audit', 'Dùng khi user yêu cầu kiểm tra bảo mật, phát hiện lỗ hổng, hoặc audit code security.')
ON CONFLICT DO NOTHING;

-- Sample plugins
INSERT INTO plugins (id, name, description) VALUES
  ('hitechcloud-devtools', 'HiTechCloud DevTools', 'Bộ công cụ phát triển HiTechCloud - MCP tools, skills, và CLI integration.'),
  ('hitechcloud-analytics', 'HiTechCloud Analytics', 'Plugin theo dõi và phân tích usage, billing, và performance metrics.')
ON CONFLICT DO NOTHING;
