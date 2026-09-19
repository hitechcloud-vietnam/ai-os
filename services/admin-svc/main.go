package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"math/rand"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/cors"
	"github.com/jackc/pgx/v5/pgxpool"
)

// ═══════════════════════════════════════════════════════════════
//  Models
// ═══════════════════════════════════════════════════════════════

type Organization struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	Plan      string    `json:"plan"`
	CreatedAt time.Time `json:"created_at"`
}

type User struct {
	ID        string    `json:"id"`
	Email     string    `json:"email"`
	OrgID     *string   `json:"org_id,omitempty"`
	CreatedAt time.Time `json:"created_at"`
}

type AuditLog struct {
	ID           int64            `json:"id"`
	OrgID        *string          `json:"org_id,omitempty"`
	ActorID      *string          `json:"actor_id,omitempty"`
	Action       string           `json:"action"`
	ResourceType *string          `json:"resource_type,omitempty"`
	ResourceID   *string          `json:"resource_id,omitempty"`
	ResultStatus string           `json:"result_status"`
	RequestMeta  *json.RawMessage `json:"request_meta,omitempty"`
	CreatedAt    time.Time        `json:"created_at"`
}

type MCPServer struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Transport   string    `json:"transport"`
	Namespace   string    `json:"namespace"`
	Visibility  string    `json:"visibility"`
	OwnerOrgID  *string   `json:"owner_org_id,omitempty"`
	CreatedAt   time.Time `json:"created_at"`
}

type Skill struct {
	ID          string  `json:"id"`
	Scope       string  `json:"scope"`
	OwnerID     *string `json:"owner_id,omitempty"`
	Name        string  `json:"name"`
	Description string  `json:"description"`
}

type Plugin struct {
	ID          string  `json:"id"`
	OwnerOrgID  *string `json:"owner_org_id,omitempty"`
	Name        string  `json:"name"`
	Description string  `json:"description"`
}

// ═══════════════════════════════════════════════════════════════
//  Main
// ═══════════════════════════════════════════════════════════════

func main() {
	port := getEnv("ADMIN_SVC_PORT", "8085")
	dbURL := getEnv("DATABASE_URL", "postgres://hitechcloud:hitechcloud_dev_2026@127.0.0.1:5432/hitechcloud")

	ctx := context.Background()
	pool, err := pgxpool.New(ctx, dbURL)
	if err != nil {
		log.Fatalf("Unable to connect to database: %v", err)
	}
	defer pool.Close()

	if err := pool.Ping(ctx); err != nil {
		log.Fatalf("Unable to ping database: %v", err)
	}
	log.Println("Admin Service: Connected to PostgreSQL")

	r := chi.NewRouter()
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)
	r.Use(middleware.Timeout(30 * time.Second))
	r.Use(cors.Handler(cors.Options{
		AllowedOrigins:   []string{"*"},
		AllowedMethods:   []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders:   []string{"Accept", "Authorization", "Content-Type"},
		AllowCredentials: false,
		MaxAge:           300,
	}))

	// ── Health ──
	r.Get("/health", func(w http.ResponseWriter, r *http.Request) {
		json.NewEncoder(w).Encode(map[string]string{
			"status":  "ok",
			"version": "0.1.0",
			"service": "hitechcloud-admin-svc",
		})
	})

	// ═══════════════════════════════════════════════════════════
	//  Organizations
	// ═══════════════════════════════════════════════════════════
	r.Get("/orgs", func(w http.ResponseWriter, r *http.Request) {
		rows, err := pool.Query(ctx,
			`SELECT id::text, name, plan, created_at FROM organizations ORDER BY created_at DESC LIMIT 100`)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		defer rows.Close()
		var orgs []Organization
		for rows.Next() {
			var o Organization
			rows.Scan(&o.ID, &o.Name, &o.Plan, &o.CreatedAt)
			orgs = append(orgs, o)
		}
		writeJSON(w, 200, map[string]any{"data": orEmpty(orgs), "total": len(orgs)})
	})

	r.Post("/orgs", func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			Name string `json:"name"`
			Plan string `json:"plan"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.Name == "" {
			writeError(w, 422, "validation_error", "name is required")
			return
		}
		if req.Plan == "" {
			req.Plan = "free"
		}
		var org Organization
		err := pool.QueryRow(ctx,
			`INSERT INTO organizations (name, plan) VALUES ($1, $2)
			 RETURNING id::text, name, plan, created_at`,
			req.Name, req.Plan,
		).Scan(&org.ID, &org.Name, &org.Plan, &org.CreatedAt)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		writeJSON(w, 201, org)
	})

	// ═══════════════════════════════════════════════════════════
	//  Users
	// ═══════════════════════════════════════════════════════════
	r.Get("/users", func(w http.ResponseWriter, r *http.Request) {
		rows, err := pool.Query(ctx,
			`SELECT id::text, email, org_id::text, created_at FROM users ORDER BY created_at DESC LIMIT 100`)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		defer rows.Close()
		var users []User
		for rows.Next() {
			var u User
			rows.Scan(&u.ID, &u.Email, &u.OrgID, &u.CreatedAt)
			users = append(users, u)
		}
		writeJSON(w, 200, map[string]any{"data": orEmpty(users), "total": len(users)})
	})

	r.Post("/users", func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			Email string `json:"email"`
			OrgID string `json:"org_id"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.Email == "" {
			writeError(w, 422, "validation_error", "email is required")
			return
		}
		var user User
		if req.OrgID != "" {
			err := pool.QueryRow(ctx,
				`INSERT INTO users (email, org_id) VALUES ($1, $2)
				 RETURNING id::text, email, org_id::text, created_at`,
				req.Email, req.OrgID,
			).Scan(&user.ID, &user.Email, &user.OrgID, &user.CreatedAt)
			if err != nil {
				writeError(w, 500, "internal_error", err.Error())
				return
			}
		} else {
			err := pool.QueryRow(ctx,
				`INSERT INTO users (email) VALUES ($1)
				 RETURNING id::text, email, org_id::text, created_at`,
				req.Email,
			).Scan(&user.ID, &user.Email, &user.OrgID, &user.CreatedAt)
			if err != nil {
				writeError(w, 500, "internal_error", err.Error())
				return
			}
		}
		writeJSON(w, 201, user)
	})

	// ═══════════════════════════════════════════════════════════
	//  API Keys
	// ═══════════════════════════════════════════════════════════
	r.Post("/api-keys", func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			Email string `json:"email"`
			Scope string `json:"scope"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.Email == "" {
			writeError(w, 422, "validation_error", "email is required")
			return
		}
		if req.Scope == "" {
			req.Scope = "user"
		}

		var userID string
		err := pool.QueryRow(ctx, `SELECT id::text FROM users WHERE email = $1`, req.Email).Scan(&userID)
		if err != nil {
			writeError(w, 404, "not_found", "User not found. Create user first via POST /users")
			return
		}

		apiKey := "hitechcloud-sk_live_" + randomString(32)
		keyHash := sha256Hex(apiKey)

		_, err = pool.Exec(ctx,
			`INSERT INTO api_keys (user_id, key_hash, scope) VALUES ($1, $2, $3)`,
			userID, keyHash, req.Scope)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}

		writeJSON(w, 201, map[string]string{
			"key":   apiKey,
			"scope": req.Scope,
			"note":  "Save this key now — it will NOT be shown again",
		})
	})

	// ═══════════════════════════════════════════════════════════
	//  MCP Servers (Admin CRUD)
	// ═══════════════════════════════════════════════════════════
	r.Get("/mcp-servers", func(w http.ResponseWriter, r *http.Request) {
		rows, err := pool.Query(ctx,
			`SELECT id, name, transport, namespace, visibility, owner_org_id::text, created_at
			 FROM mcp_servers ORDER BY created_at DESC LIMIT 100`)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		defer rows.Close()
		var servers []MCPServer
		for rows.Next() {
			var s MCPServer
			rows.Scan(&s.ID, &s.Name, &s.Transport, &s.Namespace, &s.Visibility, &s.OwnerOrgID, &s.CreatedAt)
			servers = append(servers, s)
		}
		writeJSON(w, 200, map[string]any{"data": orEmpty(servers), "total": len(servers)})
	})

	r.Post("/mcp-servers", func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			Name       string `json:"name"`
			Transport  string `json:"transport"`
			Namespace  string `json:"namespace"`
			Visibility string `json:"visibility"`
			OwnerOrgID string `json:"owner_org_id"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.Name == "" {
			writeError(w, 422, "validation_error", "name, transport, namespace required")
			return
		}
		if req.Visibility == "" {
			req.Visibility = "public"
		}
		id := slugify(req.Name)
		var server MCPServer
		if req.OwnerOrgID != "" {
			err := pool.QueryRow(ctx,
				`INSERT INTO mcp_servers (id, name, transport, namespace, visibility, owner_org_id)
				 VALUES ($1,$2,$3,$4,$5,$6)
				 RETURNING id, name, transport, namespace, visibility, owner_org_id::text, created_at`,
				id, req.Name, req.Transport, req.Namespace, req.Visibility, req.OwnerOrgID,
			).Scan(&server.ID, &server.Name, &server.Transport, &server.Namespace, &server.Visibility, &server.OwnerOrgID, &server.CreatedAt)
			if err != nil {
				writeError(w, 500, "internal_error", err.Error())
				return
			}
		} else {
			err := pool.QueryRow(ctx,
				`INSERT INTO mcp_servers (id, name, transport, namespace, visibility)
				 VALUES ($1,$2,$3,$4,$5)
				 RETURNING id, name, transport, namespace, visibility, owner_org_id::text, created_at`,
				id, req.Name, req.Transport, req.Namespace, req.Visibility,
			).Scan(&server.ID, &server.Name, &server.Transport, &server.Namespace, &server.Visibility, &server.OwnerOrgID, &server.CreatedAt)
			if err != nil {
				writeError(w, 500, "internal_error", err.Error())
				return
			}
		}
		writeJSON(w, 201, server)
	})

	r.Delete("/mcp-servers/{id}", func(w http.ResponseWriter, r *http.Request) {
		id := chi.URLParam(r, "id")
		ct, err := pool.Exec(ctx, `DELETE FROM mcp_servers WHERE id = $1`, id)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		if ct.RowsAffected() == 0 {
			writeError(w, 404, "not_found", fmt.Sprintf("MCP server '%s' not found", id))
			return
		}
		writeJSON(w, 200, map[string]any{"deleted": true, "id": id})
	})

	// ═══════════════════════════════════════════════════════════
	//  Skills (Admin CRUD)
	// ═══════════════════════════════════════════════════════════
	r.Get("/skills", func(w http.ResponseWriter, r *http.Request) {
		rows, err := pool.Query(ctx,
			`SELECT id, scope, owner_id::text, name, description FROM skills ORDER BY name LIMIT 100`)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		defer rows.Close()
		var skills []Skill
		for rows.Next() {
			var s Skill
			rows.Scan(&s.ID, &s.Scope, &s.OwnerID, &s.Name, &s.Description)
			skills = append(skills, s)
		}
		writeJSON(w, 200, map[string]any{"data": orEmpty(skills), "total": len(skills)})
	})

	r.Post("/skills", func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			Name        string `json:"name"`
			Scope       string `json:"scope"`
			Description string `json:"description"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.Name == "" {
			writeError(w, 422, "validation_error", "name and scope required")
			return
		}
		if req.Scope == "" {
			req.Scope = "public"
		}
		id := slugify(req.Name)
		_, err := pool.Exec(ctx,
			`INSERT INTO skills (id, scope, name, description) VALUES ($1,$2,$3,$4)`,
			id, req.Scope, req.Name, req.Description)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		writeJSON(w, 201, Skill{ID: id, Scope: req.Scope, Name: req.Name, Description: req.Description})
	})

	r.Delete("/skills/{id}", func(w http.ResponseWriter, r *http.Request) {
		id := chi.URLParam(r, "id")
		ct, err := pool.Exec(ctx, `DELETE FROM skills WHERE id = $1`, id)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		if ct.RowsAffected() == 0 {
			writeError(w, 404, "not_found", fmt.Sprintf("Skill '%s' not found", id))
			return
		}
		writeJSON(w, 200, map[string]any{"deleted": true, "id": id})
	})

	// ═══════════════════════════════════════════════════════════
	//  Plugins (Admin CRUD)
	// ═══════════════════════════════════════════════════════════
	r.Get("/plugins", func(w http.ResponseWriter, r *http.Request) {
		rows, err := pool.Query(ctx,
			`SELECT id, owner_org_id::text, name, description FROM plugins ORDER BY name LIMIT 100`)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		defer rows.Close()
		var plugins []Plugin
		for rows.Next() {
			var p Plugin
			rows.Scan(&p.ID, &p.OwnerOrgID, &p.Name, &p.Description)
			plugins = append(plugins, p)
		}
		writeJSON(w, 200, map[string]any{"data": orEmpty(plugins), "total": len(plugins)})
	})

	r.Post("/plugins", func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			Name        string `json:"name"`
			Description string `json:"description"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.Name == "" {
			writeError(w, 422, "validation_error", "name is required")
			return
		}
		id := slugify(req.Name)
		_, err := pool.Exec(ctx,
			`INSERT INTO plugins (id, name, description) VALUES ($1,$2,$3)`,
			id, req.Name, req.Description)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		writeJSON(w, 201, Plugin{ID: id, Name: req.Name, Description: req.Description})
	})

	r.Delete("/plugins/{id}", func(w http.ResponseWriter, r *http.Request) {
		id := chi.URLParam(r, "id")
		ct, err := pool.Exec(ctx, `DELETE FROM plugins WHERE id = $1`, id)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		if ct.RowsAffected() == 0 {
			writeError(w, 404, "not_found", fmt.Sprintf("Plugin '%s' not found", id))
			return
		}
		writeJSON(w, 200, map[string]any{"deleted": true, "id": id})
	})

	// ═══════════════════════════════════════════════════════════
	//  Audit Logs
	// ═══════════════════════════════════════════════════════════
	r.Get("/audit-logs", func(w http.ResponseWriter, r *http.Request) {
		orgId := r.URL.Query().Get("org_id")
		var rows_query string
		var args []any
		if orgId != "" {
			rows_query = `SELECT id, org_id::text, actor_id::text, action, resource_type, resource_id,
				result_status, request_meta, created_at
				FROM audit_logs WHERE org_id::text = $1 ORDER BY created_at DESC LIMIT 200`
			args = append(args, orgId)
		} else {
			rows_query = `SELECT id, org_id::text, actor_id::text, action, resource_type, resource_id,
				result_status, request_meta, created_at
				FROM audit_logs ORDER BY created_at DESC LIMIT 200`
		}
		rows, err := pool.Query(ctx, rows_query, args...)
		if err != nil {
			writeError(w, 500, "internal_error", err.Error())
			return
		}
		defer rows.Close()
		var logs []AuditLog
		for rows.Next() {
			var l AuditLog
			rows.Scan(&l.ID, &l.OrgID, &l.ActorID, &l.Action, &l.ResourceType, &l.ResourceID,
				&l.ResultStatus, &l.RequestMeta, &l.CreatedAt)
			logs = append(logs, l)
		}
		writeJSON(w, 200, map[string]any{"data": orEmpty(logs), "total": len(logs)})
	})

	log.Printf("Admin Service (SaaS) listening on :%s", port)
	log.Fatal(http.ListenAndServe(":"+port, r))
}

// ═══════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════

func getEnv(key, fallback string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return fallback
}

func writeJSON(w http.ResponseWriter, status int, data any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

func writeError(w http.ResponseWriter, status int, code, message string) {
	writeJSON(w, status, map[string]any{
		"error": map[string]string{"code": code, "message": message},
	})
}

func orEmpty[T any](s []T) []T {
	if s == nil {
		return []T{}
	}
	return s
}

func slugify(s string) string {
	result := make([]byte, 0, len(s))
	for i := 0; i < len(s); i++ {
		c := s[i]
		switch {
		case c >= 'a' && c <= 'z':
			result = append(result, c)
		case c >= 'A' && c <= 'Z':
			result = append(result, c+32)
		case c >= '0' && c <= '9':
			result = append(result, c)
		case c == ' ' || c == '_':
			result = append(result, '-')
		case c == '-':
			result = append(result, '-')
		}
	}
	return string(result)
}

func randomString(n int) string {
	const letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
	b := make([]byte, n)
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	for i := range b {
		b[i] = letters[r.Intn(len(letters))]
	}
	return string(b)
}

func sha256Hex(s string) string {
	h := sha256.Sum256([]byte(s))
	return hex.EncodeToString(h[:])
}
