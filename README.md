# cloud-shell
Google cloud shell再現プロジェクト

## 環境変数
- `RESOURCE_LIMIT_MEMORY` - シェルのメモリー

## Testing
```
kubectl port-forward svc/cloud-shell-service -n shell 8000:8000
```