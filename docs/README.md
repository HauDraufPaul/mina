# MINA Documentation

This directory contains the Docusaurus-based documentation for MINA.

## Development

### Start Docusaurus Dev Server

```bash
npm run docs:dev
```

This starts Docusaurus on `http://localhost:3000`. The MINA app will connect to this in development mode.

### Build Documentation

```bash
npm run docs:build
```

This builds the static documentation site to `../dist/docs/`, which is then served by the MINA app in production.

### Preview Build

```bash
npm run docs:serve
```

Preview the built documentation locally.

## Integration with MINA

The documentation is integrated into MINA in two ways:

1. **Documentation Viewer**: Navigate to `/docs` in the MINA app to view documentation
2. **Help Buttons**: Many modules have help buttons (?) that link to specific documentation pages

## Adding Documentation

1. Create markdown files in the `docs/` directory
2. Update `sidebars.js` to include new pages
3. Use proper frontmatter for metadata
4. Link between pages using relative paths

## Structure

- `docs/` - Markdown documentation files
- `src/pages/` - React pages (like the landing page)
- `src/css/custom.css` - MINA theme customizations
- `static/` - Static assets (images, etc.)
- `docusaurus.config.js` - Main configuration
- `sidebars.js` - Sidebar navigation structure

## Theme

The documentation uses a custom theme matching MINA's dark neon aesthetic:
- Dark background (#0a0a0a)
- Neon cyan accents (#00ffff)
- JetBrains Mono font
- Glass morphism effects

## Features

- Full-text search
- Code syntax highlighting
- MDX support
- Sidebar navigation
- Breadcrumbs
- Table of contents
- Dark theme (default)
