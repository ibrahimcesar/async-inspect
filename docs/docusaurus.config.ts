import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'async-inspect',
  tagline: 'X-ray vision for async Rust 🔍',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  url: 'https://ibrahimcesar.github.io',
  baseUrl: '/async-inspect/',

  organizationName: 'ibrahimcesar',
  projectName: 'async-inspect',

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  markdown: {
    mermaid: true,
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },

  themes: ['@docusaurus/theme-mermaid'],

  presets: [
    [
      'classic',
      {
        docs: {
          path: 'content',
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/ibrahimcesar/async-inspect/tree/main/docs/',
          routeBasePath: '/', // Docs at root instead of /docs
        },
        blog: false, // Disable blog
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/async-inspect-social-card.png',
    navbar: {
      title: 'async-inspect',
      logo: {
        alt: 'async-inspect Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          href: 'https://docs.rs/async-inspect',
          label: 'API Docs',
          position: 'left',
        },
        {
          href: 'https://github.com/ibrahimcesar/async-inspect',
          label: 'GitHub',
          position: 'right',
        },
        {
          href: 'https://crates.io/crates/async-inspect',
          label: 'crates.io',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Getting Started',
              to: '/intro',
            },
            {
              label: 'API Reference',
              href: 'https://docs.rs/async-inspect',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub Discussions',
              href: 'https://github.com/ibrahimcesar/async-inspect/discussions',
            },
            {
              label: 'Issues',
              href: 'https://github.com/ibrahimcesar/async-inspect/issues',
            },
          ],
        },
        {
          title: 'Resources',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/ibrahimcesar/async-inspect',
            },
            {
              label: 'crates.io',
              href: 'https://crates.io/crates/async-inspect',
            },
            {
              label: 'Examples',
              href: 'https://github.com/ibrahimcesar/async-inspect/tree/main/examples',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} async-inspect. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'bash'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
