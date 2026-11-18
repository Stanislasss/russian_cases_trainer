// Writes latest.json, the auto-update manifest, for a release whose installers
// and update signatures are already uploaded. Run once after all platform builds:
// had each build edit latest.json itself, parallel builds would overwrite each
// other's entries.
//
// node updater-manifest.mjs <version> <download base URL> <notes file> <sig dir> <out file>
// `sig dir` holds the release's .sig files (Name.ext.sig signs the asset Name.ext).

import { readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

/** Update platforms by asset name; the first entry of each is what apps use. */
const PLATFORMS = [
  [/_amd64\.AppImage$/, ['linux-x86_64', 'linux-x86_64-appimage']],
  [/_amd64\.deb$/, ['linux-x86_64-deb']],
  // NSIS installer: apps installed from it update with it.
  [/_x64-setup\.exe$/, ['windows-x86_64', 'windows-x86_64-nsis']],
  [/_aarch64\.app\.tar\.gz$/, ['darwin-aarch64', 'darwin-aarch64-app']],
  [/_x64\.app\.tar\.gz$/, ['darwin-x86_64', 'darwin-x86_64-app']],
]

const [version, baseUrl, notesFile, sigDir, outFile] = process.argv.slice(2)
if (!outFile) {
  console.error('usage: updater-manifest.mjs <version> <base URL> <notes file> <sig dir> <out file>')
  process.exit(2)
}

const platforms = {}
for (const sig of readdirSync(sigDir).filter((f) => f.endsWith('.sig')).sort()) {
  const asset = sig.slice(0, -'.sig'.length)
  const match = PLATFORMS.find(([pattern]) => pattern.test(asset))
  if (!match) continue
  const entry = {
    signature: readFileSync(join(sigDir, sig), 'utf8').trim(),
    url: `${baseUrl.replace(/\/$/, '')}/${encodeURIComponent(asset)}`,
  }
  for (const key of match[1]) platforms[key] = entry
}

// Every platform has to be there: a missing one means its apps never update.
const missing = PLATFORMS.flatMap(([, keys]) => keys).filter((k) => !platforms[k])
if (missing.length > 0) {
  console.error(`no signed update for: ${missing.join(', ')}`)
  process.exit(1)
}

const manifest = {
  version,
  notes: readFileSync(notesFile, 'utf8').trim(),
  pub_date: new Date().toISOString(),
  platforms,
}
writeFileSync(outFile, JSON.stringify(manifest, null, 2) + '\n')
console.log(`latest.json for ${version}: ${Object.keys(platforms).join(', ')}`)
