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

type Skill struct {
	ID          string  `json:"id"`
	Scope       string  `json:"scope"`
	OwnerID     *string `json:"owner_id,omitempty"`
	Name        string  `json:"name"`
	Description string  `json:"description"`
}

type SkillVersion struct {
	SkillID    string `json:"skill_id"`
	Version    string `json:"version"`
	ContentRef string `json:"content_ref"`
	Status     string `json:"status"`
}

type SearchQuery struct {
	Q       string
	Page    int
	PerPage int
}

func main() {
	port := getEnv("SKILLS_SVC_PORT", "8082")
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
		AllowedOrigins:   []string{"*"},
		AllowedMethods:   []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders:   []string{"Accept", "Authorization", "Content-Type"},
		ExposedHeaders:   []string{"Link"},
		AllowCredentials: false,
		MaxAge:           300,
	}))

	r.Get("/health", func(w http.ResponseWriter, r *http.Request) {
		json.NewEncoder(w).Encode(map[string]string{
			"status":  "ok",
			"version": "0.1.0",
			"service": "hitechcloud-skills-svc",
		})
	})

	r.Get("/skills", func(w http.ResponseWriter, r *http.Request) {
		q := r.URL.Query().Get("q")
		var skills []Skill
		var err error

		if q != "" {
			rows, err := pool.Query(ctx,
				`SELECT id, scope, owner_id, name, description FROM skills
				 WHERE to_tsvector('simple', name || ' ' || description) @@ plainto_tsquery('simple', $1)
				 ORDER BY name LIMIT 50`, q)
			if err != nil {
				http.Error(w, err.Error(), 500)
				return
			}
			defer rows.Close()
			for rows.Next() {
				var s Skill
				rows.Scan(&s.ID, &s.Scope, &s.OwnerID, &s.Name, &s.Description)
				skills = append(skills, s)
			}
		} else {
			rows, err := pool.Query(ctx,
				`SELECT id, scope, owner_id, name, description FROM skills ORDER BY name LIMIT 50`)
			if err != nil {
				http.Error(w, err.Error(), 500)
				return
			}
			defer rows.Close()
			for rows.Next() {
				var s Skill
				rows.Scan(&s.ID, &s.Scope, &s.OwnerID, &s.Name, &s.Description)
				skills = append(skills, s)
			}
		}

		if skills == nil {
			skills = []Skill{}
		}
		_ = err
		json.NewEncoder(w).Encode(map[string]any{
			"data":  skills,
			"total": len(skills),
		})
	})

	r.Get("/skills/{id}", func(w http.ResponseWriter, r *http.Request) {
		id := chi.URLParam(r, "id")
		var s Skill
		err := pool.QueryRow(ctx,
			`SELECT id, scope, owner_id, name, description FROM skills WHERE id = $1`, id).
			Scan(&s.ID, &s.Scope, &s.OwnerID, &s.Name, &s.Description)
		if err != nil {
			http.Error(w, `{"error":{"code":"not_found","message":"Skill not found"}}`, 404)
			return
		}
		json.NewEncoder(w).Encode(s)
	})

	r.Get("/skills/{id}/versions", func(w http.ResponseWriter, r *http.Request) {
		id := chi.URLParam(r, "id")
		rows, err := pool.Query(ctx,
			`SELECT skill_id, version, content_ref, status FROM skill_versions WHERE skill_id = $1 ORDER BY version DESC`, id)
		if err != nil {
			http.Error(w, err.Error(), 500)
			return
		}
		defer rows.Close()

		var versions []SkillVersion
		for rows.Next() {
			var v SkillVersion
			rows.Scan(&v.SkillID, &v.Version, &v.ContentRef, &v.Status)
			versions = append(versions, v)
		}
		if versions == nil {
			versions = []SkillVersion{}
		}
		json.NewEncoder(w).Encode(versions)
	})

	r.Post("/skills", func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			Name        string `json:"name"`
			Scope       string `json:"scope"`
			Description string `json:"description"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, `{"error":{"code":"validation_error","message":"Invalid JSON"}}`, 422)
			return
		}
		id := req.Name // simplified
		_, err := pool.Exec(ctx,
			`INSERT INTO skills (id, scope, name, description) VALUES ($1, $2, $3, $4)`,
			id, req.Scope, req.Name, req.Description)
		if err != nil {
			http.Error(w, fmt.Sprintf(`{"error":{"code":"internal_error","message":"%s"}}`, err.Error()), 500)
			return
		}
		w.WriteHeader(201)
		json.NewEncoder(w).Encode(Skill{ID: id, Scope: req.Scope, Name: req.Name, Description: req.Description})
	})

	log.Printf("Skills Service listening on :%s", port)
	http.ListenAndServe(":"+port, r)
}

func getEnv(key, fallback string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return fallback
}
