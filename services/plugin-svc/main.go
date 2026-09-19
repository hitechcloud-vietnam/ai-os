package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/cors"
	"github.com/jackc/pgx/v5/pgxpool"
)

type Plugin struct {
	ID          string  `json:"id"`
	OwnerOrgID  *string `json:"owner_org_id,omitempty"`
	Name        string  `json:"name"`
	Description string  `json:"description"`
}

type PluginVersion struct {
	PluginID     string          `json:"plugin_id"`
	Version      string          `json:"version"`
	Manifest     json.RawMessage `json:"manifest"`
	PackageHash  string          `json:"package_hash"`
	Signature    *string         `json:"signature,omitempty"`
	Status       string          `json:"status"`
}

func main() {
	port := getEnv("PLUGIN_SVC_PORT", "8083")
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
	log.Println("Connected to PostgreSQL")

	r := chi.NewRouter()
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)
	r.Use(middleware.Timeout(30 * time.Second))
	r.Use(cors.Handler(cors.Options{
		AllowedOrigins: []string{"*"},
		AllowedMethods: []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders: []string{"Accept", "Authorization", "Content-Type"},
		AllowCredentials: false,
		MaxAge: 300,
	}))

	r.Get("/health", func(w http.ResponseWriter, r *http.Request) {
		json.NewEncoder(w).Encode(map[string]string{
			"status":  "ok",
			"version": "0.1.0",
			"service": "hitechcloud-plugin-svc",
		})
	})

	r.Get("/plugins", func(w http.ResponseWriter, r *http.Request) {
		rows, err := pool.Query(ctx,
			`SELECT id, owner_org_id, name, description FROM plugins ORDER BY name LIMIT 50`)
		if err != nil {
			http.Error(w, err.Error(), 500)
			return
		}
		defer rows.Close()

		var plugins []Plugin
		for rows.Next() {
			var p Plugin
			rows.Scan(&p.ID, &p.OwnerOrgID, &p.Name, &p.Description)
			plugins = append(plugins, p)
		}
		if plugins == nil {
			plugins = []Plugin{}
		}
		json.NewEncoder(w).Encode(map[string]any{"data": plugins, "total": len(plugins)})
	})

	r.Get("/plugins/{id}", func(w http.ResponseWriter, r *http.Request) {
		id := chi.URLParam(r, "id")
		var p Plugin
		err := pool.QueryRow(ctx,
			`SELECT id, owner_org_id, name, description FROM plugins WHERE id = $1`, id).
			Scan(&p.ID, &p.OwnerOrgID, &p.Name, &p.Description)
		if err != nil {
			http.Error(w, `{"error":{"code":"not_found","message":"Plugin not found"}}`, 404)
			return
		}
		json.NewEncoder(w).Encode(p)
	})

	r.Get("/plugins/{id}/versions", func(w http.ResponseWriter, r *http.Request) {
		id := chi.URLParam(r, "id")
		rows, err := pool.Query(ctx,
			`SELECT plugin_id, version, manifest, package_hash, signature, status
			 FROM plugin_versions WHERE plugin_id = $1 ORDER BY version DESC`, id)
		if err != nil {
			http.Error(w, err.Error(), 500)
			return
		}
		defer rows.Close()

		var versions []PluginVersion
		for rows.Next() {
			var v PluginVersion
			rows.Scan(&v.PluginID, &v.Version, &v.Manifest, &v.PackageHash, &v.Signature, &v.Status)
			versions = append(versions, v)
		}
		if versions == nil {
			versions = []PluginVersion{}
		}
		json.NewEncoder(w).Encode(versions)
	})

	r.Post("/plugins", func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			Name        string `json:"name"`
			Description string `json:"description"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, `{"error":{"code":"validation_error","message":"Invalid JSON"}}`, 422)
			return
		}
		id := req.Name
		_, err := pool.Exec(ctx,
			`INSERT INTO plugins (id, name, description) VALUES ($1, $2, $3)`,
			id, req.Name, req.Description)
		if err != nil {
			http.Error(w, fmt.Sprintf(`{"error":{"code":"internal_error","message":"%s"}}`, err.Error()), 500)
			return
		}
		w.WriteHeader(201)
		json.NewEncoder(w).Encode(Plugin{ID: id, Name: req.Name, Description: req.Description})
	})

	log.Printf("Plugin Service listening on :%s", port)
	http.ListenAndServe(":"+port, r)
}

func getEnv(key, fallback string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return fallback
}
