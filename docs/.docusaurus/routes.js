import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/docs/docs',
    component: ComponentCreator('/docs/docs', 'be6'),
    routes: [
      {
        path: '/docs/docs',
        component: ComponentCreator('/docs/docs', '0e5'),
        routes: [
          {
            path: '/docs/docs',
            component: ComponentCreator('/docs/docs', 'bf4'),
            routes: [
              {
                path: '/docs/docs/api/tauri-commands',
                component: ComponentCreator('/docs/docs/api/tauri-commands', '7a4'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docs/api/websocket-events',
                component: ComponentCreator('/docs/docs/api/websocket-events', '65c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docs/getting-started/overview',
                component: ComponentCreator('/docs/docs/getting-started/overview', 'dcb'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docs/getting-started/quick-start',
                component: ComponentCreator('/docs/docs/getting-started/quick-start', '638'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docs/guides/scripts',
                component: ComponentCreator('/docs/docs/guides/scripts', '9f2'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docs/guides/workflows',
                component: ComponentCreator('/docs/docs/guides/workflows', '0e7'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docs/intro',
                component: ComponentCreator('/docs/docs/intro', '2ea'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docs/modules/automation-circuit',
                component: ComponentCreator('/docs/docs/modules/automation-circuit', '58c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docs/modules/market-intelligence',
                component: ComponentCreator('/docs/docs/modules/market-intelligence', '51c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docs/modules/system-monitor',
                component: ComponentCreator('/docs/docs/modules/system-monitor', '6ec'),
                exact: true,
                sidebar: "tutorialSidebar"
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '/docs/',
    component: ComponentCreator('/docs/', '2a6'),
    exact: true
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
