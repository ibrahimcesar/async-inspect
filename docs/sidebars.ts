import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    'intro',
    'async-stack-traces',
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'installation',
        'quickstart',
        'async-state-machines',
      ],
    },
    {
      type: 'category',
      label: 'Usage',
      items: [
        'cli-usage',
        'examples',
      ],
    },
    {
      type: 'category',
      label: 'Deployment',
      items: [
        'production',
        'troubleshooting',
      ],
    },
    {
      type: 'category',
      label: 'Integrations',
      items: [
        'integrations/prometheus',
        'integrations/opentelemetry',
        'integrations/tracing',
      ],
    },
  ],
};

export default sidebars;
