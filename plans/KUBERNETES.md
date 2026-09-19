# Kubernetes

## Namespace gợi ý

```
hitechcloud-production
hitechcloud-staging
hitechcloud-dev
```

## Deployment mẫu (MCP Gateway)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-gateway
  namespace: hitechcloud-production
spec:
  replicas: 3
  selector:
    matchLabels: { app: mcp-gateway }
  template:
    metadata:
      labels: { app: mcp-gateway }
    spec:
      containers:
        - name: mcp-gateway
          image: registry-mcp.hitechcloud.vn/mcp-gateway:1.4.2
          ports: [{ containerPort: 8080 }]
          envFrom:
            - secretRef: { name: mcp-gateway-secrets }
          resources:
            requests: { cpu: "250m", memory: "256Mi" }
            limits: { cpu: "1", memory: "512Mi" }
          readinessProbe:
            httpGet: { path: /healthz, port: 8080 }
            initialDelaySeconds: 5
          livenessProbe:
            httpGet: { path: /healthz, port: 8080 }
            initialDelaySeconds: 15
---
apiVersion: v1
kind: Service
metadata:
  name: mcp-gateway
  namespace: hitechcloud-production
spec:
  selector: { app: mcp-gateway }
  ports: [{ port: 80, targetPort: 8080 }]
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: mcp-gateway-hpa
  namespace: hitechcloud-production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: mcp-gateway
  minReplicas: 3
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target: { type: Utilization, averageUtilization: 70 }
```

## Sandbox runner cho MCP server chưa verified

Chạy như Job/Pod riêng với:
- `securityContext.runAsNonRoot: true`
- `NetworkPolicy` chỉ cho phép egress tới domain trong allowlist (qua egress gateway/proxy nội bộ).
- Giới hạn `resources.limits` chặt để tránh 1 MCP server lỗi ảnh hưởng node dùng chung.

## Ingress & TLS

- Ingress controller (nginx/Traefik) + cert-manager để tự động renew TLS cho `gw.hitechcloud.vn`, `api-mcp.hitechcloud.vn`.
- Bật mTLS nội bộ giữa Gateway ↔ Registry qua service mesh (Linkerd/Istio) nếu cần compliance cao hơn (Enterprise).

## Observability trong cluster

Xem [OBSERVABILITY.md](OBSERVABILITY.md) cho Prometheus/Grafana/OpenTelemetry setup tương ứng.
