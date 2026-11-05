# Yo website

Website sources for [yo.lurenyang.com](https://yo.lurenyang.com). React and Tailwind, prerendered to static HTML with Bun.

```sh
bun install
bun run dev      # local server with live rebuilds
bun run build    # prerender to dist/
bun run deploy   # build and push dist/ to the gh-pages branch
```

The default website origin and GitHub Pages custom domain are `https://yo.lurenyang.com`. Set `SITE_URL` to override the origin for local previews or another deployment. DNS and the GitHub Pages domain must be configured separately. Subpath hosting is not supported by the current asset URLs.
