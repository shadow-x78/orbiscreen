# Cloudflare Workers Build Setup

إعدادات **Cloudflare Workers Builds** الصحيحة:

## Dashboard Settings

افتح Cloudflare dashboard → Workers & Pages → orbiscreen-site → Settings → Builds

### Root Directory
```
/
```
(أو اتركه فارغ - يعني repository root)

### Build Command
```
npm run build
```
(أو اتركه فارغ - الـbuild block في wrangler.jsonc سيعمل)

### Deploy Command  
```
npm run deploy
```

### Node Version
```
22
```

## التحقق المحلي

```bash
# Test build
npm run build

# Test deploy (dry-run)
npx wrangler deploy --dry-run

# Deploy manually (if needed)
npm run deploy
```

## إذا لا يزال Build يفشل

1. تحقق من logs في Cloudflare dashboard
2. جرّب Manual Deployment:
   ```bash
   npm run build
   npm run deploy
   ```
3. تحقق من Git Integration settings (Connected Repository)

## الملفات المهمة

- `wrangler.jsonc` - root config مع build block
- `site/wrangler.jsonc` - site root config (backup)
- `.nvmrc` - Node 22
- `worker/index.mjs` - Worker code
- `site/dist/` - Built assets (auto-generated)
