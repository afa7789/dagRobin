# Release Process

## How it works

Push a tag `v*` to trigger the release workflow automatically.

The CI will:
1. Build binaries for **Linux x86/ARM** and **macOS x86/ARM**
2. Create a **GitHub Release** with `.tar.gz` + SHA256 checksums
3. Generate release notes from commits since the last tag

## Creating a release

### Option 1: make tag

```bash
make tag VERSION=0.2.0
git push origin main --tags
```

This updates `Cargo.toml` version, commits, and creates the tag.

### Option 2: manual

```bash
# 1. Update version in Cargo.toml
# 2. Commit
git add Cargo.toml
git commit -m "bump version to 0.2.0"

# 3. Tag and push
git tag v0.2.0
git push origin main --tags
```

## Workflow details

**File:** `.github/workflows/release.yml`

**Targets built:**

| Artifact | OS | Arch |
|---|---|---|
| `dagRobin-linux-amd64.tar.gz` | Linux | x86_64 |
| `dagRobin-linux-arm64.tar.gz` | Linux | aarch64 |
| `dagRobin-macos-amd64.tar.gz` | macOS | x86_64 |
| `dagRobin-macos-arm64.tar.gz` | macOS | aarch64 |

Each artifact includes a `.sha256` checksum file.

## Installing from a release

```bash
# Example: macOS ARM (Apple Silicon)
curl -L https://github.com/afa7789/dagRobin/releases/latest/download/dagRobin-macos-arm64.tar.gz | tar xz
sudo mv dagRobin /usr/local/bin/
```

---

## npm distribution

The `npm/` folder publishes a thin wrapper package (`dagrobin`) whose
`postinstall` downloads the binary for the user's platform from the GitHub
release of the same version and verifies its SHA256.

**Order matters:** publish to npm only *after* the release workflow has finished
and the `.tar.gz` + `.sha256` assets exist for that tag — otherwise every
`npm install` fails at postinstall.

### Release checklist

```bash
# 1. Bump both versions (make tag does Cargo.toml + npm/package.json)
make tag VERSION=0.2.0
git push origin main --tags

# 2. Wait for the Release workflow to attach the binaries
gh run watch
gh release view v0.2.0

# 3. Publish to npm
cd npm
npm publish --access public      # needs npm login, or NODE_AUTH_TOKEN
```

Or run the **npm publish** workflow manually from the Actions tab with the tag
as input — it requires an `NPM_TOKEN` repository secret.

### Verifying a published package

```bash
npm install -g dagrobin
dagRobin --version
dagRobin which-db
```

---

## Documentation site

`docs/` is deployed to GitHub Pages by `.github/workflows/pages.yml` on every
push to `main` that touches `docs/`. Enable it once under
**Settings → Pages → Source: GitHub Actions**.

Live at <https://afa7789.github.io/dagrobin/>.
