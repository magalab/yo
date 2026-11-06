export const SITE_URL = (process.env.SITE_URL ?? 'https://yo.lurenyang.com').replace(/\/$/, '');
export const INSTALL_COMMAND = 'curl -fsSL https://raw.githubusercontent.com/magalab/yo/develop/script/install.sh | sh';
export const SITE_TITLE = 'Yo';
export const SITE_DESCRIPTION =
  'Yo is the fast, native code editor, written in Rust. Batteries included, with coding agents built in. No account, no telemetry.';

export const REPO_URL = 'https://github.com/magalab/yo';
export const RELEASES_URL = `${REPO_URL}/releases`;
export const ISSUES_URL = `${REPO_URL}/issues`;
export const INSTALL_GUIDE_URL = `${REPO_URL}/blob/main/docs/INSTALL.md`;
export const MEMORY_BENCHMARK_URL = `${REPO_URL}/blob/main/docs/memory-benchmark.md`;
export const OPEN_COLLECTIVE_URL = 'https://opencollective.com/REPLACE_WITH_YOUR_SLUG';
export const RAYCAST_URL = 'https://www.raycast.com/farshed/yo-editor';
export const ZED_URL = 'https://github.com/zed-industries/zed';
export const ZED_DOCS_URL = 'https://zed.dev/docs';
