/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  tutorialSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      items: ['getting-started/overview', 'getting-started/quick-start'],
    },
    {
      type: 'category',
      label: 'Modules',
      items: [
        'modules/automation-circuit',
        'modules/system-monitor',
        'modules/market-intelligence',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: [
        'api/tauri-commands',
        'api/websocket-events',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/workflows',
        'guides/scripts',
      ],
    },
  ],
};

module.exports = sidebars;
