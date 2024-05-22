# cloud-shell
Google cloud shell再現プロジェクト

## Todo
- [ ] - アカウント必須にする
- [ ] - 帯域制限

## 環境変数
- `MEMORY_LIMIT` - Shell memory

## Testing
```
kubectl port-forward svc/cloud-shell-service -n shell 8000:8000
```
or
```
kubectl patch svc cloud-shell-service -n shell -p '{"spec": {"type": "LoadBalancer"}}'
```
